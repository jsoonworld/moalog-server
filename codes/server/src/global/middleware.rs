use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tracing::Instrument;
use uuid::Uuid;

use crate::state::AppState;
use crate::utils::response::ErrorResponse;

// TODO: Phase 2에서 handler에서 RequestId 추출 시 사용 예정
#[derive(Clone)]
#[allow(dead_code)]
pub struct RequestId(pub String);

pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Span에 request_id 포함 - instrument()로 async-safe하게 적용
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri().path(),
    );

    let request_id_for_header = request_id.clone();

    // instrument()를 사용하여 멀티스레드 런타임에서도 안전하게 span 유지
    async move {
        let mut response = next.run(request).await;
        response.headers_mut().insert(
            "x-request-id",
            request_id_for_header
                .parse()
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("unknown")),
        );
        response
    }
    .instrument(span)
    .await
}

/// 전역 API Rate Limit 미들웨어 (IP 기반, distributed-rate-limiter 연동)
/// Fail Open 정책: rate-limiter 서비스 장애 시 요청 허용
pub async fn global_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // 클라이언트 IP 추출
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("unknown").trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(String::from)
        })
        .unwrap_or_else(|| "unknown".to_string());

    let key = format!("global_api:ip:{}", client_ip);

    match state.rate_limit_client.check(&key, "SLIDING_WINDOW").await {
        Ok(result) if !result.allowed => {
            let error_body = ErrorResponse::new(
                "RATE4291",
                "요청 한도를 초과했습니다. 잠시 후 다시 시도해주세요.",
            );
            let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(error_body)).into_response();
            if let Ok(val) = result.retry_after_seconds.to_string().parse() {
                response.headers_mut().insert("Retry-After", val);
            }
            return response;
        }
        Ok(_) => { /* 허용 — 계속 진행 */ }
        Err(()) => { /* Fail Open — 계속 진행 */ }
    }

    next.run(request).await
}
