use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use domain::users::model::{SortDirection, SortField, UserId, UserListQuery};
use domain::users::service::UserService;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppResult;
use crate::dtos::common::{
    ApiSortDirection, ApiSortField, CursorPaginatedResponse, CursorPaginationMeta,
    ErrorResponse,
};
use crate::dtos::user::{CreateUserRequest, UserResponse};
use crate::extractors::{Pagination, RequireAdmin, ValidatedJson};

mod cursor;

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(
        ("limit" = Option<u32>, Query, description = "Items per page (default: 20, max: 100)"),
        ("after" = Option<String>, Query, description = "Opaque cursor from previous page's next_cursor"),
        ("search" = Option<String>, Query, description = "Filter by email (substring match)"),
        ("sort_by" = Option<ApiSortField>, Query, description = "Sort field: created_at (default) or email"),
        ("sort_direction" = Option<ApiSortDirection>, Query, description = "Sort direction: desc (default) or asc"),
    ),
    responses(
        (status = 200, description = "List of users", body = inline(CursorPaginatedResponse<UserResponse>)),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
    )
)]
#[tracing::instrument(skip(user_service, pagination), fields(limit = pagination.limit))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn list_users(
    _: RequireAdmin,
    State(user_service): State<Arc<UserService>>,
    pagination: Pagination,
) -> AppResult<Json<CursorPaginatedResponse<UserResponse>>> {
    let sort_by: SortField = pagination.sort_by.into();
    let direction: SortDirection = pagination.sort_direction.into();

    let after = pagination
        .after
        .as_deref()
        .map(|s| cursor::decode(s, pagination.sort_by, pagination.sort_direction))
        .transpose()?;

    let list_query = UserListQuery {
        search: pagination.search,
        sort_by,
        direction,
    };
    let (users, has_next_page) = user_service.list(&list_query, after, pagination.limit).await?;

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
            limit: pagination.limit,
            has_next_page,
            next_cursor,
        },
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    security(("bearer_auth" = [])),
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 409, description = "Email already taken", body = ErrorResponse),
    )
)]
#[tracing::instrument(skip(user_service, req))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn create_user(
    _: RequireAdmin,
    State(user_service): State<Arc<UserService>>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let user = user_service
        .create(&req.email, req.password, req.first_name, req.last_name, domain::users::model::Role::User)
        .await?;

    tracing::info!(user_id = %user.id().as_uuid(), "user created");

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    )
)]
#[tracing::instrument(skip(user_service))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn get_user(
    _: RequireAdmin,
    State(user_service): State<Arc<UserService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let user = user_service.get_by_id(UserId::from(id)).await?;
    Ok(Json(UserResponse::from(user)))
}
