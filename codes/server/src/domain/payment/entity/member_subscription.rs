use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::subscription_plan::PlanName;

/// 구독 상태
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "SubscriptionStatus"
)]
pub enum SubscriptionStatus {
    #[sea_orm(string_value = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "CANCELLED")]
    Cancelled,
    #[sea_orm(string_value = "EXPIRED")]
    Expired,
    #[sea_orm(string_value = "PENDING")]
    Pending,
}

/// 회원 구독 엔티티
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "member_subscription")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub subscription_id: i64,
    pub member_id: i64,
    pub plan_name: PlanName,
    pub status: SubscriptionStatus,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub fluxpay_order_id: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    pub fluxpay_payment_id: Option<String>,
    pub started_at: DateTime,
    #[sea_orm(nullable)]
    pub expires_at: Option<DateTime>,
    #[sea_orm(nullable)]
    pub cancelled_at: Option<DateTime>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::member::entity::member::Entity",
        from = "Column::MemberId",
        to = "crate::domain::member::entity::member::Column::MemberId"
    )]
    Member,
}

impl Related<crate::domain::member::entity::member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
