use axum::{extract::State, Json};
use validator::Validate;

use super::dto::{
    CreateSubscriptionRequest, FluxPayWebhookPayload, PlanResponse, SubscriptionResponse,
};
use super::service::PaymentService;
use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

/// 플랜 목록 조회 API
#[utoipa::path(
    get,
    path = "/api/v1/plans",
    responses(
        (status = 200, description = "플랜 목록 조회 성공", body = SuccessPlansResponse)
    ),
    tag = "Payment"
)]
pub async fn list_plans() -> Json<BaseResponse<Vec<PlanResponse>>> {
    let plans = PaymentService::list_plans();
    Json(BaseResponse::success(plans))
}

/// 내 구독 조회 API
#[utoipa::path(
    get,
    path = "/api/v1/subscriptions/me",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "구독 조회 성공", body = SuccessSubscriptionResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse)
    ),
    tag = "Payment"
)]
pub async fn get_subscription(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<BaseResponse<SubscriptionResponse>>, AppError> {
    let member_id = user.user_id()?;
    let subscription = PaymentService::get_subscription(&state, member_id).await?;
    Ok(Json(BaseResponse::success(subscription)))
}

/// 구독 생성 API
#[utoipa::path(
    post,
    path = "/api/v1/subscriptions",
    security(("bearer_auth" = [])),
    request_body = CreateSubscriptionRequest,
    responses(
        (status = 200, description = "구독 생성 성공", body = SuccessSubscriptionResponse),
        (status = 400, description = "잘못된 플랜", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 409, description = "이미 활성 구독", body = ErrorResponse),
        (status = 502, description = "FluxPay 서비스 오류", body = ErrorResponse)
    ),
    tag = "Payment"
)]
pub async fn create_subscription(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<Json<BaseResponse<SubscriptionResponse>>, AppError> {
    req.validate()?;
    let member_id = user.user_id()?;
    let subscription =
        PaymentService::create_subscription(&state, member_id, &req.plan_name).await?;
    Ok(Json(BaseResponse::success_with_message(
        subscription,
        "구독이 생성되었습니다.".to_string(),
    )))
}

/// 구독 해지 API
#[utoipa::path(
    post,
    path = "/api/v1/subscriptions/cancel",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "구독 해지 성공", body = SuccessCancelResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "활성 구독 없음", body = ErrorResponse)
    ),
    tag = "Payment"
)]
pub async fn cancel_subscription(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<BaseResponse<()>>, AppError> {
    let member_id = user.user_id()?;
    PaymentService::cancel_subscription(&state, member_id).await?;
    Ok(Json(BaseResponse {
        is_success: true,
        code: "COMMON200".to_string(),
        message: "구독이 해지되었습니다.".to_string(),
        result: None,
    }))
}

/// FluxPay 결제 완료 웹훅 API
#[utoipa::path(
    post,
    path = "/api/v1/webhooks/fluxpay",
    request_body = FluxPayWebhookPayload,
    responses(
        (status = 200, description = "웹훅 처리 성공", body = SuccessCancelResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Payment"
)]
pub async fn handle_fluxpay_webhook(
    State(state): State<AppState>,
    Json(payload): Json<FluxPayWebhookPayload>,
) -> Result<Json<BaseResponse<()>>, AppError> {
    PaymentService::handle_fluxpay_webhook(&state, payload).await?;
    Ok(Json(BaseResponse::success_with_message(
        (),
        "웹훅 처리 완료".to_string(),
    )))
}
