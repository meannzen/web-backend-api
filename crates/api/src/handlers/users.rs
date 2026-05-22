use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use domain::users::model::UserId;
use domain::users::service::UserService;
use uuid::Uuid;

use crate::AppResult;
use crate::dtos::common::{ErrorResponse, PaginatedResponse, PaginationMeta, PaginationParams};
use crate::dtos::user::{CreateUserRequest, UserResponse};
use crate::extractors::ValidatedJson;

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 20, max: 100)"),
    ),
    responses(
        (status = 200, description = "List of users", body = inline(PaginatedResponse<UserResponse>)),
    )
)]
#[tracing::instrument(skip(user_service, params), fields(page = params.page, per_page = params.per_page))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn list_users(
    State(user_service): State<Arc<UserService>>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<UserResponse>>> {
    let per_page = params.per_page.clamp(1, 100);
    let offset = params.page.saturating_sub(1) * per_page;

    let (users, total) = user_service.list(offset, per_page).await?;

    tracing::info!(total = total, "listed users");

    Ok(Json(PaginatedResponse {
        data: users.into_iter().map(UserResponse::from).collect(),
        meta: PaginationMeta {
            page: params.page,
            per_page,
            total,
            total_pages: total.div_ceil(per_page as u64) as u32,
        },
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 409, description = "Email already taken", body = ErrorResponse),
    )
)]
#[tracing::instrument(skip(user_service, req))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn create_user(
    State(user_service): State<Arc<UserService>>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let user = user_service.create(&req.email, req.password).await?;

    tracing::info!(user_id = %user.id().as_uuid(), "user created");

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    )
)]
#[tracing::instrument(skip(user_service))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn get_user(
    State(user_service): State<Arc<UserService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let user = user_service.get_by_id(UserId::from(id)).await?;
    Ok(Json(UserResponse::from(user)))
}
