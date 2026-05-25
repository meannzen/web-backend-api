use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::DateTime;
use domain::users::model::{CursorValue, SortField, UserId, UserCursor, UserListQuery};
use domain::users::service::UserService;
use uuid::Uuid;

use crate::AppResult;
use crate::dtos::common::{
    ApiSortDirection, ApiSortField, CursorPaginatedResponse, CursorPaginationMeta,
    CursorPaginationParams, ErrorResponse,
};
use crate::dtos::user::{CreateUserRequest, UserResponse};
use crate::error::AppError;
use crate::extractors::ValidatedJson;

fn sort_field_str(f: SortField) -> &'static str {
    match f {
        SortField::CreatedAt => "created_at",
        SortField::Email => "email",
    }
}

fn sort_direction_str(d: domain::users::model::SortDirection) -> &'static str {
    match d {
        domain::users::model::SortDirection::Asc => "asc",
        domain::users::model::SortDirection::Desc => "desc",
    }
}

fn encode_cursor(user: &UserResponse, sort_by: SortField, direction: domain::users::model::SortDirection) -> String {
    let sort_value = match sort_by {
        SortField::CreatedAt => user.created_at.to_rfc3339(),
        SortField::Email => user.email.clone(),
    };
    let raw = format!(
        "{}|{}|{}|{}",
        sort_field_str(sort_by),
        sort_direction_str(direction),
        sort_value,
        user.id,
    );
    URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

fn decode_cursor(
    s: &str,
    expected_sort_by: ApiSortField,
    expected_direction: ApiSortDirection,
) -> Result<UserCursor, AppError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|_| AppError::Validation("invalid cursor".to_string()))?;
    let raw =
        String::from_utf8(bytes).map_err(|_| AppError::Validation("invalid cursor".to_string()))?;

    // format: sort_by|direction|sort_value|uuid
    // We split from the end since sort_value may theoretically contain '|' if we ever add
    // fields that do; but for created_at and email it cannot. Split into exactly 4 parts.
    let parts: Vec<&str> = raw.splitn(4, '|').collect();
    if parts.len() != 4 {
        return Err(AppError::Validation("invalid cursor".to_string()));
    }
    let (cursor_sort, cursor_dir, cursor_value, cursor_uuid) =
        (parts[0], parts[1], parts[2], parts[3]);

    let expected_sort_str = match expected_sort_by {
        ApiSortField::CreatedAt => "created_at",
        ApiSortField::Email => "email",
    };
    let expected_dir_str = match expected_direction {
        ApiSortDirection::Asc => "asc",
        ApiSortDirection::Desc => "desc",
    };

    if cursor_sort != expected_sort_str || cursor_dir != expected_dir_str {
        return Err(AppError::Validation(
            "cursor sort order does not match request parameters".to_string(),
        ));
    }

    let sort_by = SortField::from(expected_sort_by);
    let direction = domain::users::model::SortDirection::from(expected_direction);

    let value = match sort_by {
        SortField::CreatedAt => {
            let ts = DateTime::parse_from_rfc3339(cursor_value)
                .map_err(|_| AppError::Validation("invalid cursor".to_string()))?
                .to_utc();
            CursorValue::Timestamp(ts)
        }
        SortField::Email => CursorValue::Text(cursor_value.to_string()),
    };

    let id = UserId::from(
        Uuid::parse_str(cursor_uuid)
            .map_err(|_| AppError::Validation("invalid cursor".to_string()))?,
    );

    Ok(UserCursor { sort_by, direction, value, id })
}

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
    let direction: domain::users::model::SortDirection = params.sort_direction.into();

    let after = params
        .after
        .as_deref()
        .map(|s| decode_cursor(s, params.sort_by, params.sort_direction))
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
        users.last().map(|u| encode_cursor(u, sort_by, direction))
    } else {
        None
    };

    Ok(Json(CursorPaginatedResponse {
        data: users,
        meta: CursorPaginationMeta { limit, has_next_page, next_cursor },
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
