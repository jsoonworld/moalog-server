use crate::domain::{
    member::entity::{assistant_usage, member, member_response, member_retro, member_retro_room},
    payment::entity::member_subscription,
    retrospect::entity::{
        response, response_comment, response_like, retro_reference, retro_room, retrospect,
    },
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema, Statement};
use std::env;
use tracing::info;

pub async fn establish_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    info!("Successfully connected to the database.");

    // Check if schema update is enabled
    let should_update_schema = env::var("DB_SCHEMA_UPDATE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or_else(|_| {
            tracing::warn!(
                "Invalid DB_SCHEMA_UPDATE value, defaulting to false. Use 'true' or 'false'."
            );
            false
        });

    if should_update_schema {
        // Auto-create tables (Schema Sync)
        create_tables(&db).await?;
    } else {
        info!("Skipping database schema synchronization (DB_SCHEMA_UPDATE is not true).");
    }

    Ok(db)
}

async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    info!("Starting database schema synchronization...");

    // List of entities to create
    // Order matters for foreign keys! (Parent first, then Child)

    // 1. Independent Entities
    create_table_if_not_exists(db, &schema, member::Entity).await?;
    create_table_if_not_exists(db, &schema, retro_room::Entity).await?;

    // 2. Dependent Entities (Level 1)
    create_table_if_not_exists(db, &schema, retrospect::Entity).await?;

    // 3. Dependent Entities (Level 2)
    create_table_if_not_exists(db, &schema, response::Entity).await?;
    create_table_if_not_exists(db, &schema, retro_reference::Entity).await?;
    create_table_if_not_exists(db, &schema, member_retro_room::Entity).await?;

    // 4. Dependent Entities (Level 3 & Join Tables)
    create_table_if_not_exists(db, &schema, response_comment::Entity).await?;
    create_table_if_not_exists(db, &schema, response_like::Entity).await?;
    create_table_if_not_exists(db, &schema, assistant_usage::Entity).await?;
    // 월간 사용량 쿼리 최적화를 위한 인덱스
    create_index_if_not_exists(
        db,
        "idx_assistant_usage_member_created",
        "assistant_usage",
        &["member_id", "created_at"],
    )
    .await?;
    create_unique_index_if_not_exists(
        db,
        "uq_response_like_member_response",
        "response_like",
        &["member_id", "response_id"],
    )
    .await?;
    create_table_if_not_exists(db, &schema, member_response::Entity).await?;
    create_table_if_not_exists(db, &schema, member_retro::Entity).await?;

    // 5. Payment 관련 테이블
    create_table_if_not_exists(db, &schema, member_subscription::Entity).await?;
    create_index_if_not_exists(
        db,
        "idx_member_subscription_member_status",
        "member_subscription",
        &["member_id", "status"],
    )
    .await?;

    // Apply migrations for existing tables
    apply_migrations(db).await?;

    info!("Database schema synchronization completed.");
    Ok(())
}

/// Apply ALTER TABLE migrations for existing tables.
/// This handles adding new columns to tables that already exist.
async fn apply_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Migration: Add missing columns to member table
    add_column_if_not_exists(db, "member", "nickname", "VARCHAR(255) NULL").await?;
    add_column_if_not_exists(
        db,
        "member",
        "social_type",
        "ENUM('KAKAO', 'GOOGLE') NOT NULL DEFAULT 'KAKAO'",
    )
    .await?;
    add_column_if_not_exists(db, "member", "insight_count", "INT NOT NULL DEFAULT 0").await?;
    add_column_if_not_exists(db, "member", "refresh_token", "VARCHAR(500) NULL").await?;
    add_column_if_not_exists(db, "member", "refresh_token_expires_at", "DATETIME NULL").await?;

    // Migration: Add created_at column to member_retro_room table
    add_column_if_not_exists(
        db,
        "member_retro_room",
        "created_at",
        "DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP",
    )
    .await?;

    Ok(())
}

/// Add a column to a table if it doesn't already exist.
async fn add_column_if_not_exists(
    db: &DatabaseConnection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let sql = format!(
        "ALTER TABLE {} ADD COLUMN {} {}",
        table_name, column_name, column_definition
    );
    let stmt = Statement::from_string(backend, sql);

    match db.execute(stmt).await {
        Ok(_) => {
            info!("Added column '{}' to table '{}'", column_name, table_name);
            Ok(())
        }
        Err(e) => {
            // Ignore "column already exists" errors for idempotency
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("duplicate")
                || err_str.contains("already exists")
                || err_str.contains("duplicate column")
            {
                Ok(())
            } else {
                tracing::error!(
                    "Failed to add column '{}' to table '{}': {}",
                    column_name,
                    table_name,
                    e
                );
                Err(e)
            }
        }
    }
}

async fn create_index_if_not_exists(
    db: &DatabaseConnection,
    index_name: &str,
    table_name: &str,
    columns: &[&str],
) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let cols = columns.join(", ");
    let sql = format!("CREATE INDEX {} ON {} ({})", index_name, table_name, cols);
    let stmt = Statement::from_string(backend, sql);
    match db.execute(stmt).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("duplicate")
                || err_str.contains("already exists")
                || err_str.contains("exists")
            {
                Ok(())
            } else {
                tracing::error!("Failed to create index {}: {}", index_name, e);
                Err(e)
            }
        }
    }
}

async fn create_unique_index_if_not_exists(
    db: &DatabaseConnection,
    index_name: &str,
    table_name: &str,
    columns: &[&str],
) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let cols = columns.join(", ");
    let sql = format!(
        "CREATE UNIQUE INDEX {} ON {} ({})",
        index_name, table_name, cols
    );
    let stmt = Statement::from_string(backend, sql);
    match db.execute(stmt).await {
        Ok(_) => Ok(()),
        Err(e) => {
            // Ignore duplicate index errors for idempotency.
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("duplicate")
                || err_str.contains("already exists")
                || err_str.contains("exists")
            {
                Ok(())
            } else {
                tracing::error!("Failed to create unique index {}: {}", index_name, e);
                Err(e)
            }
        }
    }
}

async fn create_table_if_not_exists<E>(
    db: &DatabaseConnection,
    schema: &Schema,
    entity: E,
) -> Result<(), DbErr>
where
    E: sea_orm::EntityTrait,
{
    let backend = db.get_database_backend();
    let create_stmt: Statement =
        backend.build(schema.create_table_from_entity(entity).if_not_exists());

    match db.execute(create_stmt).await {
        Ok(_) => {
            // Log success (optional, verbose)
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to create table: {}", e);
            Err(e)
        }
    }
}
