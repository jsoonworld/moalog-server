use std::env;

/// 애플리케이션 설정
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
    pub signup_token_expiration: i64,

    // Social Login
    pub google_client_id: String,
    pub google_client_secret: String,
    pub kakao_client_id: String,
    pub kakao_client_secret: String,

    // AI Service
    pub openai_api_key: String,

    // Rate Limiting
    pub rate_limiter_url: String,
    pub rate_limiter_enabled: bool,
}

impl AppConfig {
    /// 환경 변수에서 설정 로드
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidPort)?;

        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(v) => v,
            Err(_) if cfg!(debug_assertions) => {
                tracing::warn!(
                    "JWT_SECRET 환경변수가 설정되지 않았습니다. 개발 환경에서 기본값을 사용합니다."
                );
                "secret".to_string()
            }
            Err(_) => return Err(ConfigError::MissingJwtSecret),
        };

        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "1800".to_string()) // Default 30 min
            .parse()
            .map_err(|_| ConfigError::InvalidExpiration)?;

        let refresh_token_expiration = env::var("REFRESH_TOKEN_EXPIRATION")
            .unwrap_or_else(|_| "1209600".to_string()) // Default 14 days
            .parse()
            .map_err(|_| ConfigError::InvalidExpiration)?;

        let signup_token_expiration = env::var("SIGNUP_TOKEN_EXPIRATION")
            .unwrap_or_else(|_| "600".to_string()) // Default 10 min
            .parse()
            .map_err(|_| ConfigError::InvalidExpiration)?;

        let google_client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
        let google_client_secret = match env::var("GOOGLE_CLIENT_SECRET") {
            Ok(v) => v,
            Err(_) if cfg!(debug_assertions) => {
                tracing::warn!("GOOGLE_CLIENT_SECRET 환경변수가 설정되지 않았습니다.");
                String::new()
            }
            Err(_) => return Err(ConfigError::MissingGoogleClientSecret),
        };
        let kakao_client_id = env::var("KAKAO_CLIENT_ID").unwrap_or_default();
        let kakao_client_secret = match env::var("KAKAO_CLIENT_SECRET") {
            Ok(v) => v,
            Err(_) if cfg!(debug_assertions) => {
                tracing::warn!("KAKAO_CLIENT_SECRET 환경변수가 설정되지 않았습니다.");
                String::new()
            }
            Err(_) => return Err(ConfigError::MissingKakaoClientSecret),
        };

        let openai_api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
            tracing::warn!(
                "OPENAI_API_KEY 환경변수가 설정되지 않았습니다. 프로덕션 환경에서는 반드시 설정하세요."
            );
            "test-key".to_string()
        });

        let rate_limiter_url = env::var("RATE_LIMITER_URL")
            .unwrap_or_else(|_| "http://localhost:8082".to_string());
        let rate_limiter_enabled = env::var("RATE_LIMITER_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Ok(Self {
            server_port,
            jwt_secret,
            jwt_expiration,
            refresh_token_expiration,
            signup_token_expiration,
            google_client_id,
            google_client_secret,
            kakao_client_id,
            kakao_client_secret,
            openai_api_key,
            rate_limiter_url,
            rate_limiter_enabled,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port number")]
    InvalidPort,
    #[error("Invalid expiration time")]
    InvalidExpiration,
    #[error("JWT_SECRET environment variable is required in production")]
    MissingJwtSecret,
    #[error("GOOGLE_CLIENT_SECRET environment variable is required in production")]
    MissingGoogleClientSecret,
    #[error("KAKAO_CLIENT_SECRET environment variable is required in production")]
    MissingKakaoClientSecret,
}
