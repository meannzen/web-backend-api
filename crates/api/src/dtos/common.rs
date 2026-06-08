use domain::users::model::{SortDirection, SortField};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Serialize, ToSchema)]
pub struct CursorPaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub meta: CursorPaginationMeta,
}

#[derive(Serialize, ToSchema)]
pub struct CursorPaginationMeta {
    pub limit: u32,
    pub has_next_page: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone, Copy, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiSortField {
    #[default]
    CreatedAt,
    Email,
}

impl From<ApiSortField> for SortField {
    fn from(f: ApiSortField) -> Self {
        match f {
            ApiSortField::CreatedAt => SortField::CreatedAt,
            ApiSortField::Email => SortField::Email,
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone, Copy, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiSortDirection {
    #[default]
    Desc,
    Asc,
}

impl From<ApiSortDirection> for SortDirection {
    fn from(d: ApiSortDirection) -> Self {
        match d {
            ApiSortDirection::Asc => SortDirection::Asc,
            ApiSortDirection::Desc => SortDirection::Desc,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CursorPaginationParams<S = ApiSortField, D = ApiSortDirection> {
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub after: Option<String>,
    pub search: Option<String>,
    #[serde(default)]
    pub sort_by: S,
    #[serde(default)]
    pub sort_direction: D,
}

fn default_limit() -> u32 {
    20
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorDetail {
    #[serde(rename = "type")]
    pub kind: String,
    pub message: String,
}
