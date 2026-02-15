use std::sync::Arc;

use crate::config::AppConfig;
use crate::domain::ai::service::AiService;
use crate::global::rate_limit::{AiRateLimiters, DistributedRateLimitClient};
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub ai_service: AiService,
    pub rate_limit_client: Arc<DistributedRateLimitClient>,
    pub ai_rate_limiters: Arc<AiRateLimiters>,
}
