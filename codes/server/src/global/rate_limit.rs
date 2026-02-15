use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use governor::clock::DefaultClock;
use governor::state::keyed::DashMapStateStore;
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::utils::error::AppError;

// ─── Distributed Rate Limiter HTTP Client ───────────────────────

/// distributed-rate-limiter 서비스 응답 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitCheckResponse {
    pub allowed: bool,
    pub remaining: i64,
    pub retry_after_seconds: i64,
}

/// distributed-rate-limiter HTTP 클라이언트
/// Fail Open 정책: 서비스 장애 시 요청 허용
#[derive(Clone)]
pub struct DistributedRateLimitClient {
    client: Client,
    base_url: String,
    enabled: bool,
}

impl DistributedRateLimitClient {
    pub fn new(base_url: String, enabled: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .expect("Failed to build reqwest client for rate limiter");
        Self {
            client,
            base_url,
            enabled,
        }
    }

    /// rate-limiter 서비스에 허용 여부 확인
    /// Ok(response) = 정상 응답, Err(()) = 서비스 장애 (Fail Open 적용)
    pub async fn check(&self, key: &str, algorithm: &str) -> Result<RateLimitCheckResponse, ()> {
        if !self.enabled {
            return Err(());
        }

        let url = format!(
            "{}/api/v1/rate-limit/check?key={}&algorithm={}",
            self.base_url, key, algorithm
        );

        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<RateLimitCheckResponse>().await {
                Ok(body) => {
                    debug!(
                        key = %key,
                        allowed = %body.allowed,
                        remaining = %body.remaining,
                        "Rate limit check"
                    );
                    Ok(body)
                }
                Err(e) => {
                    warn!("Rate limit 응답 파싱 실패: {}. Fail Open 적용.", e);
                    Err(())
                }
            },
            Err(e) => {
                warn!("Rate limiter 서비스 연결 실패: {}. Fail Open 적용.", e);
                Err(())
            }
        }
    }
}

// ─── In-Process Governor Rate Limiters ──────────────────────────

/// Keyed governor rate limiter 타입 별칭
type KeyedLimiter = RateLimiter<String, DashMapStateStore<String>, DefaultClock>;

/// AI 엔드포인트용 in-process rate limiter
pub struct AiRateLimiters {
    /// AI 분석: 분당 5회, burst 3 (유저별)
    analysis: Arc<KeyedLimiter>,
    /// AI 가이드: 분당 10회 (유저별)
    guide: Arc<KeyedLimiter>,
    /// OpenAI 서비스 전체: 초당 20회
    openai_global: Arc<KeyedLimiter>,
}

impl AiRateLimiters {
    pub fn new() -> Self {
        Self {
            analysis: Arc::new(RateLimiter::keyed(
                Quota::per_minute(NonZeroU32::new(5).unwrap())
                    .allow_burst(NonZeroU32::new(3).unwrap()),
            )),
            guide: Arc::new(RateLimiter::keyed(
                Quota::per_minute(NonZeroU32::new(10).unwrap()),
            )),
            openai_global: Arc::new(RateLimiter::keyed(
                Quota::per_second(NonZeroU32::new(20).unwrap()),
            )),
        }
    }

    /// AI 분석 rate limit 체크 (유저별 + OpenAI 전역)
    pub fn check_analysis(&self, user_id: i64) -> Result<(), AppError> {
        let user_key = user_id.to_string();

        if self.analysis.check_key(&user_key).is_err() {
            return Err(AppError::RateLimitExceeded(
                "AI 분석 요청 한도를 초과했습니다. 1분 후 다시 시도해주세요.".to_string(),
            ));
        }

        if self
            .openai_global
            .check_key(&"openai".to_string())
            .is_err()
        {
            return Err(AppError::RateLimitExceeded(
                "AI 서비스가 일시적으로 과부하 상태입니다. 잠시 후 다시 시도해주세요.".to_string(),
            ));
        }

        Ok(())
    }

    /// AI 가이드 rate limit 체크 (유저별 + OpenAI 전역)
    pub fn check_guide(&self, user_id: i64) -> Result<(), AppError> {
        let user_key = user_id.to_string();

        if self.guide.check_key(&user_key).is_err() {
            return Err(AppError::RateLimitExceeded(
                "AI 가이드 요청 한도를 초과했습니다. 1분 후 다시 시도해주세요.".to_string(),
            ));
        }

        if self
            .openai_global
            .check_key(&"openai".to_string())
            .is_err()
        {
            return Err(AppError::RateLimitExceeded(
                "AI 서비스가 일시적으로 과부하 상태입니다. 잠시 후 다시 시도해주세요.".to_string(),
            ));
        }

        Ok(())
    }
}
