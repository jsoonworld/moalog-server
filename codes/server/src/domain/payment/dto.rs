use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::entity::member_subscription::SubscriptionStatus;
use super::entity::subscription_plan::PlanName;

// ─── Requests ──────────────────────────────────────────────────

/// 구독 생성 요청
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionRequest {
    /// 플랜 이름 ("FREE", "PRO", "TEAM")
    #[validate(length(min = 1, message = "플랜 이름은 필수입니다"))]
    pub plan_name: String,
}

/// FluxPay 웹훅 페이로드
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FluxPayWebhookPayload {
    /// 이벤트 타입 (e.g., "payment.confirmed", "payment.failed")
    pub event_type: String,
    /// FluxPay 주문 ID
    pub order_id: String,
    /// FluxPay 결제 ID
    pub payment_id: String,
    /// 결제 상태
    pub status: String,
}

// ─── Responses ──────────────────────────────────────────────────

/// 플랜 정보 응답
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanResponse {
    /// 플랜 이름
    pub name: String,
    /// 표시 이름
    pub display_name: String,
    /// 월간 AI 분석 제한 (-1 = 무제한)
    pub ai_analysis_limit: i32,
    /// 월간 AI 어시스턴트 제한 (-1 = 무제한)
    pub ai_assistant_limit: i32,
    /// PDF 내보내기 기능
    pub pdf_export_enabled: bool,
    /// 회고방 최대 인원 (-1 = 무제한)
    pub max_room_members: i32,
    /// 월간 가격 (원)
    pub monthly_price: i32,
}

impl PlanResponse {
    pub fn from_plan(plan: PlanName) -> Self {
        Self {
            name: format!("{:?}", plan).to_uppercase(),
            display_name: plan.display_name().to_string(),
            ai_analysis_limit: plan.ai_analysis_limit().unwrap_or(-1),
            ai_assistant_limit: plan.ai_assistant_limit().unwrap_or(-1),
            pdf_export_enabled: plan.pdf_export_enabled(),
            max_room_members: plan.max_room_members(),
            monthly_price: plan.monthly_price(),
        }
    }
}

/// 구독 정보 응답
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionResponse {
    /// 구독 ID
    pub subscription_id: i64,
    /// 플랜 이름
    pub plan_name: PlanName,
    /// 구독 상태
    pub status: SubscriptionStatus,
    /// 구독 시작일
    pub started_at: String,
    /// 만료일 (없으면 null)
    pub expires_at: Option<String>,
    /// 해지일 (없으면 null)
    pub cancelled_at: Option<String>,
}

// ─── Swagger용 래퍼 타입 ──────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessPlansResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Vec<PlanResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessSubscriptionResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: SubscriptionResponse,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessCancelResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: Option<()>,
}
