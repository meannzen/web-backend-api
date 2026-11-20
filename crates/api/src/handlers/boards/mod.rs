use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use domain::boards::model::{BoardId, BoardListQuery, SortField};
use domain::boards::service::BoardService;
use domain::pagination::SortDirection;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppResult;
use crate::cursor;
use crate::dtos::board::{ApiBoardSortField, BoardFilterParams, BoardResponse, CreateBoardRequest};
use crate::dtos::common::{
    ApiSortDirection, CursorPaginatedResponse, CursorPaginationMeta, ErrorResponse,
};
use crate::error::AppError;
use crate::extractors::{Pagination, RequireAdmin, ValidatedJson};

#[utoipa::path(
    get,
    path = "/api/v1/boards",
    tag = "Boards",
    security(("bearer_auth" = [])),
    params(
        ("limit" = Option<u32>, Query, description = "Items per page (default: 20, max: 100)"),
        ("after" = Option<String>, Query, description = "Opaque cursor from previous page's next_cursor"),
        ("search" = Option<String>, Query, description = "Filter by name or code (substring match)"),
        ("sort_by" = Option<ApiBoardSortField>, Query, description = "Sort field: created_at (default) or name"),
        ("sort_direction" = Option<ApiSortDirection>, Query, description = "Sort direction: desc (default) or asc"),
    ),
    responses(
        (status = 200, description = "List of boards", body = inline(CursorPaginatedResponse<BoardResponse>)),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
    )
)]
#[tracing::instrument(skip(board_service, pagination), fields(limit = pagination.limit))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn list_boards(
    _: RequireAdmin,
    State(board_service): State<Arc<BoardService>>,
    pagination: Pagination<BoardFilterParams, ApiBoardSortField>,
) -> AppResult<Json<CursorPaginatedResponse<BoardResponse>>> {
    if pagination
        .filter
        .search
        .as_deref()
        .is_some_and(|s| s.contains('\0'))
    {
        return Err(AppError::Validation("invalid search term".to_string()));
    }

    let sort_by: SortField = pagination.sort_by.into();
    let direction: SortDirection = pagination.sort_direction.into();

    let after = pagination
        .after
        .as_deref()
        .map(|s| cursor::decode(s, sort_by, direction))
        .transpose()?;

    let list_query = BoardListQuery {
        filter: pagination.filter.into(),
        sort_by,
        direction,
    };
    let (boards, has_next_page) = board_service
        .list(&list_query, after, pagination.limit)
        .await?;

    tracing::info!(count = boards.len(), has_next_page, "listed boards");

    let boards: Vec<BoardResponse> = boards.into_iter().map(BoardResponse::from).collect();
    let next_cursor = if has_next_page {
        boards.last().map(|b| cursor::encode(b, sort_by, direction))
    } else {
        None
    };

    Ok(Json(CursorPaginatedResponse {
        data: boards,
        meta: CursorPaginationMeta {
            limit: pagination.limit,
            has_next_page,
            next_cursor,
        },
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/boards",
    tag = "Boards",
    security(("bearer_auth" = [])),
    request_body = CreateBoardRequest,
    responses(
        (status = 201, description = "Board created", body = BoardResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 409, description = "Board code already taken", body = ErrorResponse),
    )
)]
#[tracing::instrument(skip(board_service, req))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn create_board(
    _: RequireAdmin,
    State(board_service): State<Arc<BoardService>>,
    ValidatedJson(req): ValidatedJson<CreateBoardRequest>,
) -> AppResult<(StatusCode, Json<BoardResponse>)> {
    let board = board_service.create(&req.name, &req.code).await?;

    tracing::info!(board_id = %board.id().as_uuid(), "board created");

    Ok((StatusCode::CREATED, Json(BoardResponse::from(board))))
}

#[utoipa::path(
    get,
    path = "/api/v1/boards/{id}",
    tag = "Boards",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Board ID")),
    responses(
        (status = 200, description = "Board found", body = BoardResponse),
        (status = 401, description = "Missing or invalid token", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Board not found", body = ErrorResponse),
    )
)]
#[tracing::instrument(skip(board_service))]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn get_board(
    _: RequireAdmin,
    State(board_service): State<Arc<BoardService>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<BoardResponse>> {
    let board = board_service.get_by_id(BoardId::from(id)).await?;
    Ok(Json(BoardResponse::from(board)))
}
