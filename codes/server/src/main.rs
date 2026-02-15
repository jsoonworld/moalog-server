//! Web Team 3 Backend Server

mod automation;
mod config;
mod domain;
mod event;
mod global;
mod monitoring;
mod state;
mod utils;

use axum::http::{header, HeaderValue, Method};
use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::domain::auth::dto::{
    EmailLoginRequest, EmailLoginResponse, LogoutRequest, SignupRequest, SignupResponse,
    SocialLoginRequest, SocialLoginResponse, SuccessEmailLoginResponse, SuccessLogoutResponse,
    SuccessSignupResponse, SuccessSocialLoginResponse, SuccessTokenRefreshResponse,
    TokenRefreshRequest, TokenRefreshResponse,
};
use crate::domain::member::dto::{
    MemberProfileResponse, SuccessProfileResponse, SuccessWithdrawResponse,
};
use crate::domain::member::entity::member_retro::RetrospectStatus;
use crate::domain::retrospect::dto::{
    AnalysisResponse, AssistantRequest, AssistantResponse, CommentItem, CreateCommentRequest,
    CreateCommentResponse, CreateParticipantResponse, CreateRetrospectRequest,
    CreateRetrospectResponse, DeleteRetroRoomResponse, DraftItem, DraftSaveRequest,
    DraftSaveResponse, EmotionRankItem, GuideItem, GuideType, JoinRetroRoomRequest,
    JoinRetroRoomResponse, LikeToggleResponse, ListCommentsQuery, ListCommentsResponse,
    MissionItem, PersonalMissionItem, ReferenceItem, ResponseCategory, ResponseListItem,
    ResponsesListResponse, RetroRoomCreateRequest, RetroRoomCreateResponse, RetroRoomListItem,
    RetroRoomMemberItem, RetroRoomOrderItem, RetrospectDetailResponse, RetrospectListItem,
    RetrospectMemberItem, RetrospectQuestionItem, SearchRetrospectItem, StorageRangeFilter,
    StorageResponse, StorageRetrospectItem, StorageYearGroup, SubmitAnswerItem,
    SubmitRetrospectRequest, SubmitRetrospectResponse, SuccessAnalysisResponse,
    SuccessAssistantResponse, SuccessCreateCommentResponse, SuccessCreateParticipantResponse,
    SuccessCreateRetrospectResponse, SuccessDeleteRetroRoomResponse,
    SuccessDeleteRetrospectResponse, SuccessDraftSaveResponse, SuccessEmptyResponse,
    SuccessJoinRetroRoomResponse, SuccessLikeToggleResponse, SuccessListCommentsResponse,
    SuccessReferencesListResponse, SuccessResponsesListResponse, SuccessRetroRoomCreateResponse,
    SuccessRetroRoomListResponse, SuccessRetroRoomMembersResponse, SuccessRetrospectDetailResponse,
    SuccessRetrospectListResponse, SuccessSearchResponse, SuccessStorageResponse,
    SuccessSubmitRetrospectResponse, SuccessUpdateRetroRoomNameResponse,
    UpdateRetroRoomNameRequest, UpdateRetroRoomNameResponse, UpdateRetroRoomOrderRequest,
};
use crate::domain::retrospect::entity::retrospect::RetrospectMethod;
use crate::state::AppState;
use crate::utils::{BaseResponse, ErrorResponse};

/// OpenAPI 문서 정의
#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        domain::auth::handler::social_login,
        domain::auth::handler::signup,
        domain::auth::handler::refresh_token,
        domain::auth::handler::logout,
        domain::auth::handler::login_by_email,
        domain::auth::handler::auth_test,
        // RetroRoom APIs
        domain::retrospect::handler::create_retro_room,
        domain::retrospect::handler::join_retro_room,
        domain::retrospect::handler::list_retro_rooms,
        domain::retrospect::handler::list_retro_room_members,
        domain::retrospect::handler::update_retro_room_order,
        domain::retrospect::handler::update_retro_room_name,
        domain::retrospect::handler::delete_retro_room,
        domain::retrospect::handler::list_retrospects,
        // Retrospect APIs
        domain::retrospect::handler::create_retrospect,
        domain::retrospect::handler::create_participant,
        domain::retrospect::handler::list_references,
        domain::retrospect::handler::save_draft,
        domain::retrospect::handler::get_retrospect_detail,
        domain::retrospect::handler::submit_retrospect,
        domain::retrospect::handler::get_storage,
        domain::retrospect::handler::analyze_retrospective_handler,
        domain::retrospect::handler::search_retrospects,
        domain::retrospect::handler::list_responses,
        domain::retrospect::handler::export_retrospect,
        domain::retrospect::handler::delete_retrospect,
        domain::retrospect::handler::list_comments,
        domain::retrospect::handler::create_comment,
        domain::retrospect::handler::toggle_like,
        domain::retrospect::handler::assistant_guide,
        // Member APIs
        domain::member::handler::get_profile,
        domain::member::handler::withdraw
    ),
    components(
        schemas(
            ErrorResponse,
            HealthResponse,
            SuccessHealthResponse,
            SocialLoginRequest,
            SocialLoginResponse,
            SuccessSocialLoginResponse,
            SignupRequest,
            SignupResponse,
            SuccessSignupResponse,
            TokenRefreshRequest,
            TokenRefreshResponse,
            SuccessTokenRefreshResponse,
            LogoutRequest,
            SuccessLogoutResponse,
            EmailLoginRequest,
            EmailLoginResponse,
            SuccessEmailLoginResponse,
            // RetroRoom DTOs
            RetroRoomCreateRequest,
            RetroRoomCreateResponse,
            SuccessRetroRoomCreateResponse,
            JoinRetroRoomRequest,
            JoinRetroRoomResponse,
            SuccessJoinRetroRoomResponse,
            RetroRoomListItem,
            SuccessRetroRoomListResponse,
            RetroRoomMemberItem,
            SuccessRetroRoomMembersResponse,
            RetroRoomOrderItem,
            UpdateRetroRoomOrderRequest,
            SuccessEmptyResponse,
            UpdateRetroRoomNameRequest,
            UpdateRetroRoomNameResponse,
            SuccessUpdateRetroRoomNameResponse,
            DeleteRetroRoomResponse,
            SuccessDeleteRetroRoomResponse,
            RetrospectListItem,
            SuccessRetrospectListResponse,
            // Retrospect DTOs
            CreateRetrospectRequest,
            CreateRetrospectResponse,
            SuccessCreateRetrospectResponse,
            RetrospectMethod,
            CreateParticipantResponse,
            SuccessCreateParticipantResponse,
            ReferenceItem,
            SuccessReferencesListResponse,
            DraftSaveRequest,
            DraftItem,
            DraftSaveResponse,
            SuccessDraftSaveResponse,
            SubmitRetrospectRequest,
            SubmitRetrospectResponse,
            SubmitAnswerItem,
            SuccessSubmitRetrospectResponse,
            RetrospectStatus,
            StorageRangeFilter,
            StorageRetrospectItem,
            StorageYearGroup,
            StorageResponse,
            SuccessStorageResponse,
            RetrospectDetailResponse,
            RetrospectMemberItem,
            RetrospectQuestionItem,
            SuccessRetrospectDetailResponse,
            AnalysisResponse,
            EmotionRankItem,
            MissionItem,
            PersonalMissionItem,
            SuccessAnalysisResponse,
            SearchRetrospectItem,
            SuccessSearchResponse,
            SuccessDeleteRetrospectResponse,
            ResponseCategory,
            ResponseListItem,
            ResponsesListResponse,
            SuccessResponsesListResponse,
            LikeToggleResponse,
            SuccessLikeToggleResponse,
            ListCommentsQuery,
            CommentItem,
            ListCommentsResponse,
            SuccessListCommentsResponse,
            CreateCommentRequest,
            CreateCommentResponse,
            SuccessCreateCommentResponse,
            AssistantRequest,
            AssistantResponse,
            GuideItem,
            GuideType,
            SuccessAssistantResponse,
            // Member DTOs
            MemberProfileResponse,
            SuccessProfileResponse,
            SuccessWithdrawResponse
        )
    ),
    tags(
        (name = "Health", description = "헬스 체크 API"),
        (name = "Auth", description = "인증 API"),
        (name = "RetroRoom", description = "회고방 관리 API"),
        (name = "Retrospect", description = "회고 API"),
        (name = "Response", description = "회고 답변 API"),
        (name = "Member", description = "회원 API")
    ),
    modifiers(&SecurityAddon),
    info(
        title = "회고록 서비스 API",
        version = "1.0.0",
        description = "회고록 서비스의 API 문서입니다."
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 환경 변수 로드
    dotenvy::dotenv().ok();

    // 로깅 초기화
    utils::init_logging();

    // 설정 로드
    let config = AppConfig::from_env()?;
    let port = config.server_port;

    // PDF 폰트 설정 검증
    validate_pdf_fonts();

    // DB 연결 및 테이블 생성 (Auto-Schema)
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = crate::config::establish_connection(&database_url).await?;

    // AI 서비스 초기화
    let ai_service = domain::ai::service::AiService::new(&config);

    // Rate Limiter 초기화
    let rate_limit_client = std::sync::Arc::new(global::DistributedRateLimitClient::new(
        config.rate_limiter_url.clone(),
        config.rate_limiter_enabled,
    ));
    let ai_rate_limiters = std::sync::Arc::new(global::AiRateLimiters::new());

    info!(
        "Rate Limiter 설정: url={}, enabled={}",
        config.rate_limiter_url, config.rate_limiter_enabled
    );

    // 애플리케이션 상태 생성
    let app_state = AppState {
        db,
        config: config.clone(),
        ai_service,
        rate_limit_client,
        ai_rate_limiters,
    };

    // CORS 설정 — ALLOWED_ORIGINS 환경변수에서 읽기 (미설정 시 기본값 사용)
    let default_origins = [
        "http://localhost:3000",
        "http://localhost:5173",
        "http://localhost:5174",
        "https://www.moalog.me",
        "https://moalog.me",
        "https://moaofficial.kr",
    ];

    let origins: Vec<HeaderValue> = match std::env::var("ALLOWED_ORIGINS") {
        Ok(val) if !val.trim().is_empty() => {
            val.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .filter_map(|origin| {
                    match origin.parse::<HeaderValue>() {
                        Ok(hv) => Some(hv),
                        Err(_) => {
                            warn!("ALLOWED_ORIGINS에 유효하지 않은 오리진이 포함되어 무시합니다: {}", origin);
                            None
                        }
                    }
                })
                .collect()
        }
        _ => {
            info!("ALLOWED_ORIGINS 환경변수가 설정되지 않았습니다. 기본 오리진을 사용합니다.");
            default_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect()
        }
    };

    info!("CORS 허용 오리진 수: {}", origins.len());

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ORIGIN,
        ])
        .allow_credentials(true);

    // 라우터 구성
    let app = Router::new()
        .route("/health", get(health_check))
        // [API-001] 소셜 로그인
        .route(
            "/api/v1/auth/social-login",
            axum::routing::post(domain::auth::handler::social_login),
        )
        // [API-002] 회원가입
        .route(
            "/api/v1/auth/signup",
            axum::routing::post(domain::auth::handler::signup),
        )
        // [API-003] 토큰 갱신
        .route(
            "/api/v1/auth/token/refresh",
            axum::routing::post(domain::auth::handler::refresh_token),
        )
        // [API-004] 로그아웃
        .route(
            "/api/v1/auth/logout",
            axum::routing::post(domain::auth::handler::logout),
        )
        // 테스트/개발용 API
        .route(
            "/api/auth/login/email",
            axum::routing::post(domain::auth::handler::login_by_email),
        )
        .route(
            "/api/auth/test",
            axum::routing::get(domain::auth::handler::auth_test),
        )
        // 하위 호환성을 위한 구 엔드포인트 (deprecated)
        .route("/api/auth/login", {
            #[allow(deprecated)]
            axum::routing::post(domain::auth::handler::login)
        })
        // RetroRoom routes
        .route(
            "/api/v1/retro-rooms",
            axum::routing::post(domain::retrospect::handler::create_retro_room)
                .get(domain::retrospect::handler::list_retro_rooms),
        )
        .route(
            "/api/v1/retro-rooms/join",
            axum::routing::post(domain::retrospect::handler::join_retro_room),
        )
        .route(
            "/api/v1/retro-rooms/order",
            axum::routing::patch(domain::retrospect::handler::update_retro_room_order),
        )
        .route(
            "/api/v1/retro-rooms/:retro_room_id/name",
            axum::routing::patch(domain::retrospect::handler::update_retro_room_name),
        )
        .route(
            "/api/v1/retro-rooms/:retro_room_id",
            axum::routing::delete(domain::retrospect::handler::delete_retro_room),
        )
        .route(
            "/api/v1/retro-rooms/:retro_room_id/members",
            axum::routing::get(domain::retrospect::handler::list_retro_room_members),
        )
        .route(
            "/api/v1/retro-rooms/:retro_room_id/retrospects",
            axum::routing::get(domain::retrospect::handler::list_retrospects),
        )
        // Retrospect API
        .route(
            "/api/v1/retrospects",
            axum::routing::post(domain::retrospect::handler::create_retrospect),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/participants",
            axum::routing::post(domain::retrospect::handler::create_participant),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/references",
            axum::routing::get(domain::retrospect::handler::list_references),
        )
        .route(
            "/api/v1/retrospects/search",
            axum::routing::get(domain::retrospect::handler::search_retrospects),
        )
        .route(
            "/api/v1/retrospects/storage",
            axum::routing::get(domain::retrospect::handler::get_storage),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id",
            axum::routing::get(domain::retrospect::handler::get_retrospect_detail)
                .delete(domain::retrospect::handler::delete_retrospect),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/drafts",
            axum::routing::put(domain::retrospect::handler::save_draft),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/submit",
            axum::routing::post(domain::retrospect::handler::submit_retrospect),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/analysis",
            axum::routing::post(domain::retrospect::handler::analyze_retrospective_handler),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/responses",
            axum::routing::get(domain::retrospect::handler::list_responses),
        )
        .route(
            "/api/v1/retrospects/:retrospect_id/export",
            axum::routing::get(domain::retrospect::handler::export_retrospect),
        )
        .route(
            "/api/v1/responses/:response_id/comments",
            axum::routing::get(domain::retrospect::handler::list_comments)
                .post(domain::retrospect::handler::create_comment),
        )
        // [API-025] 회고 답변 좋아요 토글
        .route(
            "/api/v1/responses/:response_id/likes",
            axum::routing::post(domain::retrospect::handler::toggle_like),
        )
        // 로그인된 유저 프로필 조회
        .route(
            "/api/v1/members/me",
            axum::routing::get(domain::member::handler::get_profile),
        )
        // [API-025] 서비스 탈퇴
        .route(
            "/api/v1/members/withdraw",
            axum::routing::post(domain::member::handler::withdraw),
        )
        // [API-029] 회고 어시스턴트
        .route(
            "/api/v1/retrospects/:retrospect_id/questions/:question_id/assistant",
            axum::routing::post(domain::retrospect::handler::assistant_guide),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // 레이어 순서: 아래에서 위로 적용됨
        // global_rate_limit → request_id → cors → TraceLayer → handler
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(axum::middleware::from_fn(global::request_id_middleware))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            global::middleware::global_rate_limit_middleware,
        ))
        .with_state(app_state);

    // 서버 시작
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server running on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 헬스 체크 엔드포인트
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "서버 상태 정상", body = SuccessHealthResponse)
    ),
    tag = "Health"
)]
async fn health_check() -> axum::Json<BaseResponse<HealthResponse>> {
    axum::Json(BaseResponse::success(HealthResponse {
        status: "healthy".to_string(),
    }))
}

/// 헬스 체크 응답 DTO
#[derive(serde::Serialize, utoipa::ToSchema)]
struct HealthResponse {
    /// 서버 상태
    status: String,
}

/// PDF 폰트 파일 존재 여부 검증
fn validate_pdf_fonts() {
    let font_dir = std::env::var("PDF_FONT_DIR").unwrap_or_else(|_| "./fonts".to_string());
    let font_family =
        std::env::var("PDF_FONT_FAMILY").unwrap_or_else(|_| "NanumGothic".to_string());

    let font_path = std::path::Path::new(&font_dir);
    let regular_font = font_path.join(format!("{}-Regular.ttf", font_family));

    if !font_path.exists() {
        warn!(
            "PDF 폰트 디렉토리가 존재하지 않습니다: {}. PDF 내보내기 기능이 작동하지 않을 수 있습니다.",
            font_dir
        );
        return;
    }

    if !regular_font.exists() {
        warn!(
            "PDF 폰트 파일이 존재하지 않습니다: {}. PDF 내보내기 기능이 작동하지 않을 수 있습니다.",
            regular_font.display()
        );
        return;
    }

    info!(
        "PDF 폰트 검증 완료: {} ({})",
        font_family,
        regular_font.display()
    );
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct SuccessHealthResponse {
    pub is_success: bool,
    pub code: String,
    pub message: String,
    pub result: HealthResponse,
}
