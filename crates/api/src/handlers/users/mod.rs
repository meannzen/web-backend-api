use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use domain::users::model::{SortDirection, SortField, UserId, UserListQuery};
use domain::users::service::UserService;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppResult;
use crate::dtos::common::{
    ApiSortDirection, ApiSortField, CursorPaginatedResponse, CursorPaginationMeta,
    CursorPaginationParams, ErrorResponse,
};
use crate::dtos::user::{CreateUserRequest, UserResponse};
use crate::extractors::ValidatedJson;

mod cursor;

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    params(
        ("limit" = Option<u32>, Query, description = "Items per page (default: 20, max: 100)"),
        ("after" = Option<String>, Query, description = "Opaque cursor from previous page's next_cursor"),
        ("search" = Option<String>, Query, description = "Filter by email (substring match)"),
        ("sort_by" = Option<ApiSortField>, Query, description = "Sort field: created_at (default) or email"),
        ("sort_direction" = Option<ApiSortDirection>, Query, description = "Sort direction: desc (default) or asc"),
    ),
    responses(
        (status = 200, description = "List of users", body = inline(CursorPaginatedResponse<UserResponse>)),
    )
)]
#[tracing::instrument(skip(user_service, params), fields(limit = params.limit))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn list_users(
    State(user_service): State<Arc<UserService>>,
    Query(params): Query<CursorPaginationParams>,
) -> AppResult<Json<CursorPaginatedResponse<UserResponse>>> {
    let limit = params.limit.clamp(1, 100);
    let sort_by: SortField = params.sort_by.into();
    let direction: SortDirection = params.sort_direction.into();

    let after = params
        .after
        .as_deref()
        .map(|s| cursor::decode(s, params.sort_by, params.sort_direction))
        .transpose()?;

    let list_query = UserListQuery {
        search: params.search,
        sort_by,
        direction,
    };
    let (users, has_next_page) = user_service.list(&list_query, after, limit).await?;

    tracing::info!(count = users.len(), has_next_page, "listed users");

    let users: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    let next_cursor = if has_next_page {
        users.last().map(|u| cursor::encode(u, sort_by, direction))
    } else {
        None
    };

    Ok(Json(CursorPaginatedResponse {
        data: users,
        meta: CursorPaginationMeta {
            limit,
            has_next_page,
            next_cursor,
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
