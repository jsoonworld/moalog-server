use axum::{
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use validator::Validate;

use crate::state::AppState;
use crate::utils::auth::AuthUser;
use crate::utils::error::AppError;
use crate::utils::BaseResponse;

use super::dto::{
    AnalysisResponse, AssistantRequest, AssistantResponse, CreateCommentRequest,
    CreateCommentResponse, CreateParticipantResponse, CreateRetrospectRequest,
    CreateRetrospectResponse, DeleteRetroRoomResponse, DraftSaveRequest, DraftSaveResponse,
    JoinRetroRoomRequest, JoinRetroRoomResponse, LikeToggleResponse, ListCommentsQuery,
    ListCommentsResponse, ReferenceItem, ResponseCategory, ResponsesListResponse,
    ResponsesQueryParams, RetroRoomCreateRequest, RetroRoomCreateResponse, RetroRoomListItem,
    RetroRoomMemberItem, RetrospectDetailResponse, RetrospectListItem, SearchQueryParams,
    SearchRetrospectItem, StorageQueryParams, StorageResponse, SubmitRetrospectRequest,
    SubmitRetrospectResponse, UpdateRetroRoomNameRequest, UpdateRetroRoomNameResponse,
    UpdateRetroRoomOrderRequest,
};
use super::service::RetrospectService;

// ============================================
// RetroRoom Handlers (API-004 ~ API-010)
// ============================================

/// 회고방 생성 API (API-004)
///
/// 새로운 회고방을 생성하고 생성자를 관리자로 설정합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retro-rooms",
    request_body = RetroRoomCreateRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고방 생성 성공", body = SuccessRetroRoomCreateResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 409, description = "이름 중복", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn create_retro_room(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<RetroRoomCreateRequest>,
) -> Result<Json<BaseResponse<RetroRoomCreateResponse>>, AppError> {
    req.validate()?;

    let member_id = user.user_id()?;

    let result = RetrospectService::create_retro_room(state, member_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고방 생성에 성공하였습니다.",
    )))
}

/// 회고방 참여 API (API-005)
///
/// 초대 링크(코드)를 통해 회고방에 참여합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retro-rooms/join",
    request_body = JoinRetroRoomRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고방 참여 성공", body = SuccessJoinRetroRoomResponse),
        (status = 400, description = "잘못된 초대 링크 또는 만료됨", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고방", body = ErrorResponse),
        (status = 409, description = "이미 참여 중", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn join_retro_room(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<JoinRetroRoomRequest>,
) -> Result<Json<BaseResponse<JoinRetroRoomResponse>>, AppError> {
    req.validate()?;

    let member_id = user.user_id()?;

    let result = RetrospectService::join_retro_room(state, member_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고방 참여에 성공하였습니다.",
    )))
}

/// 회고방 목록 조회 API (API-006)
///
/// 현재 로그인한 사용자가 참여 중인 모든 회고방 목록을 조회합니다.
#[utoipa::path(
    get,
    path = "/api/v1/retro-rooms",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "회고방 목록 조회 성공", body = SuccessRetroRoomListResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn list_retro_rooms(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<BaseResponse<Vec<RetroRoomListItem>>>, AppError> {
    let member_id = user.user_id()?;

    let result = RetrospectService::list_retro_rooms(state, member_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "참여 중인 회고방 목록 조회를 성공했습니다.",
    )))
}

/// 회고방 멤버 목록 조회 API (API-030)
///
/// 특정 회고방에 참여한 모든 멤버 목록을 조회합니다.
#[utoipa::path(
    get,
    path = "/api/v1/retro-rooms/{retro_room_id}/members",
    params(
        ("retro_room_id" = i64, Path, description = "회고방 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "회고방 멤버 목록 조회 성공", body = SuccessRetroRoomMembersResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "회고방 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn list_retro_room_members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(retro_room_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<RetroRoomMemberItem>>>, AppError> {
    let member_id = user.user_id()?;

    let result =
        RetrospectService::list_retro_room_members(state, member_id, retro_room_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고방 멤버 목록 조회를 성공했습니다.",
    )))
}

/// 회고방 순서 변경 API (API-007)
///
/// 드래그 앤 드롭으로 변경된 회고방들의 정렬 순서를 서버에 일괄 저장합니다.
#[utoipa::path(
    patch,
    path = "/api/v1/retro-rooms/order",
    request_body = UpdateRetroRoomOrderRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "순서 변경 성공", body = SuccessEmptyResponse),
        (status = 400, description = "잘못된 순서 데이터", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn update_retro_room_order(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UpdateRetroRoomOrderRequest>,
) -> Result<Json<BaseResponse<()>>, AppError> {
    req.validate()?;

    let member_id = user.user_id()?;

    RetrospectService::update_retro_room_order(state, member_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        (),
        "회고방 순서가 성공적으로 변경되었습니다.",
    )))
}

/// 회고방 이름 변경 API (API-008)
///
/// 기존 회고방의 이름을 새로운 이름으로 변경합니다. (Owner만 가능)
#[utoipa::path(
    patch,
    path = "/api/v1/retro-rooms/{retro_room_id}/name",
    request_body = UpdateRetroRoomNameRequest,
    params(
        ("retro_room_id" = i64, Path, description = "회고방 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "이름 변경 성공", body = SuccessUpdateRetroRoomNameResponse),
        (status = 400, description = "이름 길이 초과", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "회고방 없음", body = ErrorResponse),
        (status = 409, description = "이름 중복", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn update_retro_room_name(
    State(state): State<AppState>,
    user: AuthUser,
    Path(retro_room_id): Path<i64>,
    Json(req): Json<UpdateRetroRoomNameRequest>,
) -> Result<Json<BaseResponse<UpdateRetroRoomNameResponse>>, AppError> {
    req.validate()?;

    let member_id = user.user_id()?;

    let result =
        RetrospectService::update_retro_room_name(state, member_id, retro_room_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고방 이름 변경에 성공하였습니다.",
    )))
}

/// 회고방 삭제 API (API-009)
///
/// 회고방을 완전히 삭제합니다. (Owner만 가능)
#[utoipa::path(
    delete,
    path = "/api/v1/retro-rooms/{retro_room_id}",
    params(
        ("retro_room_id" = i64, Path, description = "회고방 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "삭제 성공", body = SuccessDeleteRetroRoomResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "회고방 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn delete_retro_room(
    State(state): State<AppState>,
    user: AuthUser,
    Path(retro_room_id): Path<i64>,
) -> Result<Json<BaseResponse<DeleteRetroRoomResponse>>, AppError> {
    let member_id = user.user_id()?;

    let result = RetrospectService::delete_retro_room(state, member_id, retro_room_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고방 삭제에 성공하였습니다.",
    )))
}

/// 회고방 내 회고 목록 조회 API (API-010)
///
/// 특정 회고방에 속한 모든 회고 목록을 조회합니다.
#[utoipa::path(
    get,
    path = "/api/v1/retro-rooms/{retro_room_id}/retrospects",
    params(
        ("retro_room_id" = i64, Path, description = "회고방 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "회고 목록 조회 성공", body = SuccessRetrospectListResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "회고방 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn list_retrospects(
    State(state): State<AppState>,
    user: AuthUser,
    Path(retro_room_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<RetrospectListItem>>>, AppError> {
    let member_id = user.user_id()?;

    let result = RetrospectService::list_retrospects(state, member_id, retro_room_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고방 내 전체 회고 목록 조회를 성공했습니다.",
    )))
}

// ============================================
// Retrospect Handlers
// ============================================

/// 회고 생성 API (API-011)
///
/// 진행한 프로젝트에 대한 회고 세션을 생성합니다.
/// 프로젝트 정보, 회고 방식, 참고 자료 등을 포함하며 생성된 회고의 고유 식별자를 반환합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects",
    request_body = CreateRetrospectRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고가 성공적으로 생성되었습니다.", body = SuccessCreateRetrospectResponse),
        (status = 400, description = "잘못된 요청 (프로젝트 이름 길이 초과, 날짜 형식 오류, URL 형식 오류 등)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "회고방 접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고방", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn create_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateRetrospectRequest>,
) -> Result<Json<BaseResponse<CreateRetrospectResponse>>, AppError> {
    // 입력값 검증
    req.validate()?;

    // 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::create_retrospect(state, user_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고가 성공적으로 생성되었습니다.",
    )))
}

/// 회고 참석자 등록 API (API-014)
///
/// 진행 예정인 회고에 참석자로 등록합니다.
/// JWT의 유저 정보를 기반으로 참석을 처리하며, 해당 회고가 속한 회고방의 멤버만 참석이 가능합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects/{retrospectId}/participants",
    params(
        ("retrospectId" = i64, Path, description = "참여하고자 하는 회고의 고유 ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 참석자로 성공적으로 등록되었습니다.", body = SuccessCreateParticipantResponse),
        (status = 400, description = "잘못된 요청 (retrospectId 유효성 오류)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고이거나 접근 권한 없음", body = ErrorResponse),
        (status = 409, description = "중복 참석", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn create_participant(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<CreateParticipantResponse>>, AppError> {
    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::create_participant(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 참석자로 성공적으로 등록되었습니다.",
    )))
}

/// 회고 참고자료 목록 조회 API (API-018)
///
/// 특정 회고에 등록된 모든 참고자료(URL) 목록을 조회합니다.
/// 회고 생성 시 등록했던 외부 링크들을 확인할 수 있습니다.
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/{retrospectId}/references",
    params(
        ("retrospectId" = i64, Path, description = "조회를 원하는 회고의 고유 ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "참고자료 목록을 성공적으로 조회했습니다.", body = SuccessReferencesListResponse),
        (status = 400, description = "잘못된 요청 (retrospectId 유효성 오류)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고이거나 접근 권한 없음", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn list_references(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<Vec<ReferenceItem>>>, AppError> {
    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::list_references(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "참고자료 목록을 성공적으로 조회했습니다.",
    )))
}

/// 회고 답변 임시 저장 API (API-016)
///
/// 진행 중인 회고의 답변을 임시로 저장합니다.
/// 기존에 저장된 내용이 있다면 전달받은 내용으로 덮어쓰기 처리됩니다.
#[utoipa::path(
    put,
    path = "/api/v1/retrospects/{retrospectId}/drafts",
    params(
        ("retrospectId" = i64, Path, description = "임시 저장할 회고의 고유 식별자")
    ),
    request_body = DraftSaveRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "임시 저장이 완료되었습니다.", body = SuccessDraftSaveResponse),
        (status = 400, description = "잘못된 요청 (답변 길이 초과, 잘못된 질문 번호, 빈 배열, 중복 질문 번호)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "작성 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn save_draft(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
    Json(req): Json<DraftSaveRequest>,
) -> Result<Json<BaseResponse<DraftSaveResponse>>, AppError> {
    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::save_draft(state, user_id, retrospect_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "임시 저장이 완료되었습니다.",
    )))
}

/// 회고 상세 정보 조회 API (API-012)
///
/// 특정 회고 세션의 상세 정보(제목, 일시, 유형, 참여 멤버, 질문 리스트 및 전체 통계)를 조회합니다.
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/{retrospectId}",
    params(
        ("retrospectId" = i64, Path, description = "조회할 회고의 고유 식별자")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 상세 정보 조회를 성공했습니다.", body = SuccessRetrospectDetailResponse),
        (status = 400, description = "잘못된 Path Parameter", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn get_retrospect_detail(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<RetrospectDetailResponse>>, AppError> {
    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::get_retrospect_detail(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 상세 정보 조회를 성공했습니다.",
    )))
}

/// 회고 최종 제출 API (API-017)
///
/// 작성한 모든 답변(총 5개)을 최종 제출합니다.
/// 각 답변은 최대 1,000자까지 입력 가능하며, 제출 완료 시 회고 상태가 SUBMITTED로 변경됩니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects/{retrospectId}/submit",
    params(
        ("retrospectId" = i64, Path, description = "제출할 회고의 고유 식별자")
    ),
    request_body = SubmitRetrospectRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 제출이 성공적으로 완료되었습니다.", body = SuccessSubmitRetrospectResponse),
        (status = 400, description = "잘못된 요청 (답변 누락, 답변 길이 초과, 공백만 입력 등)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "이미 제출 완료된 회고", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn submit_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
    Json(req): Json<SubmitRetrospectRequest>,
) -> Result<Json<BaseResponse<SubmitRetrospectResponse>>, AppError> {
    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::submit_retrospect(state, user_id, retrospect_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 제출이 성공적으로 완료되었습니다.",
    )))
}

/// 보관함 조회 API (API-019)
///
/// 완료된 회고 목록을 연도별로 그룹화하여 조회합니다.
/// 기간 필터를 통해 특정 기간의 회고만 조회할 수 있습니다.
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/storage",
    params(StorageQueryParams),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "보관함 조회를 성공했습니다.", body = SuccessStorageResponse),
        (status = 400, description = "유효하지 않은 기간 필터", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn get_storage(
    user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<StorageQueryParams>,
) -> Result<Json<BaseResponse<StorageResponse>>, AppError> {
    // 사용자 ID 추출
    let user_id = user.user_id()?;

    // 서비스 호출
    let result = RetrospectService::get_storage(state, user_id, params).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "보관함 조회를 성공했습니다.",
    )))
}

/// 회고 분석 API (API-022)
///
/// 특정 회고 세션에 쌓인 모든 회고방 멤버의 답변을 종합 분석하여 AI 인사이트, 감정 통계, 맞춤형 미션을 생성합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects/{retrospectId}/analysis",
    params(
        ("retrospectId" = i64, Path, description = "분석할 회고 ID")
    ),
    request_body = (),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고 분석 성공", body = SuccessAnalysisResponse),
        (status = 400, description = "잘못된 Path Parameter", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "월간 한도 초과 또는 접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "회고 없음", body = ErrorResponse),
        (status = 409, description = "이미 분석 완료된 회고", body = ErrorResponse),
        (status = 422, description = "분석 데이터 부족", body = ErrorResponse),
        (status = 500, description = "AI 분석 실패", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn analyze_retrospective_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<AnalysisResponse>>, AppError> {
    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    let user_id = user.user_id()?;

    // AI 분석 Rate Limit 체크 (유저별 5/min + OpenAI 전역 20/sec)
    state.ai_rate_limiters.check_analysis(user_id)?;

    // 서비스 호출
    let result = RetrospectService::analyze_retrospective(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "회고 분석이 성공적으로 완료되었습니다.",
    )))
}

/// 회고 검색 API (API-023)
///
/// 사용자가 참여하는 모든 회고방의 회고를 프로젝트명/회고명 기준으로 검색합니다.
/// 결과는 회고 날짜 내림차순(최신순), 동일 날짜인 경우 회고 시간 내림차순으로 정렬됩니다.
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/search",
    params(SearchQueryParams),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "검색을 성공했습니다.", body = SuccessSearchResponse),
        (status = 400, description = "검색어 누락 또는 유효하지 않음", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn search_retrospects(
    user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<BaseResponse<Vec<SearchRetrospectItem>>>, AppError> {
    let user_id = user.user_id()?;

    let result = RetrospectService::search_retrospects(state, user_id, params).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "검색을 성공했습니다.",
    )))
}

/// 회고 내보내기 API (API-021)
///
/// 특정 회고 세션의 전체 내용(회고방 인사이트, 멤버별 답변 등)을 요약하여 PDF 파일로 생성하고 다운로드합니다.
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/{retrospectId}/export",
    params(
        ("retrospectId" = i64, Path, description = "내보낼 회고의 고유 식별자")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "PDF 파일 다운로드", content_type = "application/pdf"),
        (status = 400, description = "잘못된 요청 (retrospectId 유효성 오류)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고이거나 접근 권한 없음", body = ErrorResponse),
        (status = 500, description = "PDF 생성 실패", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn export_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    let user_id = user.user_id()?;

    let pdf_bytes = RetrospectService::export_retrospect(state, user_id, retrospect_id).await?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("retrospect_report_{}_{}.pdf", retrospect_id, timestamp);

    let headers = [
        (
            header::CONTENT_TYPE,
            "application/pdf; charset=utf-8".to_string(),
        ),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        ),
        (
            header::CACHE_CONTROL,
            "no-cache, no-store, must-revalidate".to_string(),
        ),
    ];

    Ok((headers, pdf_bytes))
}

/// 회고 답변 카테고리별 조회 API (API-020)
#[utoipa::path(
    get,
    path = "/api/v1/retrospects/{retrospectId}/responses",
    params(
        ("retrospectId" = i64, Path, description = "조회를 진행할 회고 세션 고유 ID"),
        ResponsesQueryParams
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "답변 리스트 조회를 성공했습니다.", body = SuccessResponsesListResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn list_responses(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
    Query(params): Query<ResponsesQueryParams>,
) -> Result<Json<BaseResponse<ResponsesListResponse>>, AppError> {
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    let category: ResponseCategory = params.category.parse().map_err(|_| {
        AppError::RetroCategoryInvalid("유효하지 않은 카테고리 값입니다.".to_string())
    })?;

    if let Some(cursor) = params.cursor {
        if cursor < 1 {
            return Err(AppError::BadRequest(
                "cursor는 1 이상의 양수여야 합니다.".to_string(),
            ));
        }
    }

    let size = params.size.unwrap_or(10);
    if !(1..=100).contains(&size) {
        return Err(AppError::BadRequest(
            "size는 1~100 범위의 정수여야 합니다.".to_string(),
        ));
    }

    let user_id = user.user_id()?;

    let result = RetrospectService::list_responses(
        state,
        user_id,
        retrospect_id,
        category,
        params.cursor,
        size,
    )
    .await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "답변 리스트 조회를 성공했습니다.",
    )))
}

/// 회고 삭제 API (API-013)
#[utoipa::path(
    delete,
    path = "/api/v1/retrospects/{retrospectId}",
    params(
        ("retrospectId" = i64, Path, description = "삭제할 회고의 고유 식별자")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "회고가 성공적으로 삭제되었습니다.", body = SuccessDeleteRetrospectResponse),
        (status = 400, description = "잘못된 Path Parameter", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고이거나 접근 권한 없음", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn delete_retrospect(
    user: AuthUser,
    State(state): State<AppState>,
    Path(retrospect_id): Path<i64>,
) -> Result<Json<BaseResponse<()>>, AppError> {
    if retrospect_id < 1 {
        return Err(AppError::BadRequest(
            "retrospectId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    let user_id = user.user_id()?;

    RetrospectService::delete_retrospect(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        (),
        "회고가 성공적으로 삭제되었습니다.",
    )))
}

/// 회고 답변 댓글 목록 조회 API (API-026)
#[utoipa::path(
    get,
    path = "/api/v1/responses/{responseId}/comments",
    params(
        ("responseId" = i64, Path, description = "댓글을 조회할 회고 답변의 고유 식별자"),
        ("cursor" = Option<i64>, Query, description = "마지막으로 조회된 댓글 ID"),
        ("size" = Option<i32>, Query, description = "페이지당 조회 개수 (1~100, 기본값: 20)")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "댓글 조회를 성공했습니다.", body = SuccessListCommentsResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "접근 권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고 답변", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Response"
)]
pub async fn list_comments(
    user: AuthUser,
    State(state): State<AppState>,
    Path(response_id): Path<i64>,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<BaseResponse<ListCommentsResponse>>, AppError> {
    if response_id < 1 {
        return Err(AppError::BadRequest(
            "responseId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    if let Some(cursor) = query.cursor {
        if cursor < 1 {
            return Err(AppError::BadRequest(
                "cursor는 1 이상의 양수여야 합니다.".to_string(),
            ));
        }
    }

    let size = query.size.unwrap_or(20);
    if !(1..=100).contains(&size) {
        return Err(AppError::BadRequest(
            "size는 1~100 범위의 정수여야 합니다.".to_string(),
        ));
    }

    let user_id = user.user_id()?;

    let result =
        RetrospectService::list_comments(state, user_id, response_id, query.cursor, size).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "댓글 조회를 성공했습니다.",
    )))
}

/// 회고 답변 댓글 작성 API (API-027)
#[utoipa::path(
    post,
    path = "/api/v1/responses/{responseId}/comments",
    params(
        ("responseId" = i64, Path, description = "댓글을 작성할 대상 답변의 고유 ID")
    ),
    request_body = CreateCommentRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "댓글이 성공적으로 등록되었습니다.", body = SuccessCreateCommentResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고 답변", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Response"
)]
pub async fn create_comment(
    user: AuthUser,
    State(state): State<AppState>,
    Path(response_id): Path<i64>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<BaseResponse<CreateCommentResponse>>, AppError> {
    if response_id < 1 {
        return Err(AppError::BadRequest(
            "responseId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    req.validate()?;

    let user_id = user.user_id()?;

    let result = RetrospectService::create_comment(state, user_id, response_id, req).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "댓글이 성공적으로 등록되었습니다.",
    )))
}

/// 회고 답변 좋아요 토글 API (API-025)
///
/// 특정 회고 답변에 좋아요를 등록하거나 취소합니다.
/// 좋아요 미등록 상태에서 호출하면 등록, 등록 상태에서 호출하면 취소됩니다.
#[utoipa::path(
    post,
    path = "/api/v1/responses/{responseId}/likes",
    params(
        ("responseId" = i64, Path, description = "좋아요를 처리할 대상 답변의 고유 ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "좋아요 상태가 성공적으로 업데이트되었습니다.", body = SuccessLikeToggleResponse),
        (status = 400, description = "잘못된 요청 (responseId가 1 미만)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "회고방 멤버가 아닌 유저가 좋아요 시도", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 회고 답변", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류", body = ErrorResponse)
    ),
    tag = "Response"
)]
pub async fn toggle_like(
    user: AuthUser,
    State(state): State<AppState>,
    Path(response_id): Path<i64>,
) -> Result<Json<BaseResponse<LikeToggleResponse>>, AppError> {
    // responseId 검증 (1 이상의 양수)
    if response_id < 1 {
        return Err(AppError::BadRequest(
            "responseId는 1 이상의 양수여야 합니다.".to_string(),
        ));
    }

    // 사용자 ID 추출
    let user_id: i64 = user
        .0
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;

    // 서비스 호출
    let result = RetrospectService::toggle_like(state, user_id, response_id).await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "좋아요 상태가 성공적으로 업데이트되었습니다.",
    )))
}

/// 회고 어시스턴트 API (API-029)
///
/// 회고 작성 시 특정 질문에 대해 AI 어시스턴트가 작성 가이드를 제공합니다.
/// 사용자의 현재 입력 내용에 따라 초기 가이드(INITIAL) 또는 맞춤 가이드(PERSONALIZED)를 반환합니다.
#[utoipa::path(
    post,
    path = "/api/v1/retrospects/{retrospectId}/questions/{questionId}/assistant",
    params(
        ("retrospectId" = i64, Path, description = "회고의 고유 ID"),
        ("questionId" = i32, Path, description = "질문 번호 (1~5)")
    ),
    request_body = AssistantRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "가이드 생성 성공", body = SuccessAssistantResponse),
        (status = 400, description = "잘못된 요청 (content 길이 초과 등)", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "접근 권한 없음 또는 월간 사용 한도 초과", body = ErrorResponse),
        (status = 404, description = "회고 또는 질문을 찾을 수 없음", body = ErrorResponse),
        (status = 500, description = "서버 내부 오류 또는 AI 서비스 오류", body = ErrorResponse)
    ),
    tag = "Retrospect"
)]
pub async fn assistant_guide(
    user: AuthUser,
    State(state): State<AppState>,
    Path((retrospect_id, question_id)): Path<(i64, i32)>,
    Json(req): Json<AssistantRequest>,
) -> Result<Json<BaseResponse<AssistantResponse>>, AppError> {
    req.validate()?;

    let user_id = user.user_id()?;

    // AI 가이드 Rate Limit 체크 (유저별 10/min + OpenAI 전역 20/sec)
    state.ai_rate_limiters.check_guide(user_id)?;

    let result = RetrospectService::generate_assistant_guide(
        state,
        user_id,
        retrospect_id,
        question_id,
        req,
    )
    .await?;

    Ok(Json(BaseResponse::success_with_message(
        result,
        "가이드가 성공적으로 생성되었습니다.",
    )))
}
