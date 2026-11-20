use chrono::{DateTime, Utc};
use domain::boards::model::SortField;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::cursor::CursorSource;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateBoardRequest {
    #[validate(length(min = 1, max = 200, message = "must be between 1 and 200 characters"))]
    pub name: String,
    /// Short unique code, e.g. "TDSB". Letters, digits, '-' and '_'; stored uppercase.
    #[validate(length(min = 1, max = 20, message = "must be between 1 and 20 characters"))]
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoardResponse {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<domain::boards::model::Board> for BoardResponse {
    fn from(b: domain::boards::model::Board) -> Self {
        BoardResponse {
            id: *b.id().as_uuid(),
            name: b.name().to_string(),
            code: b.code().as_ref().to_string(),
            created_at: b.created_at(),
            updated_at: b.updated_at(),
        }
    }
}

impl CursorSource<SortField> for BoardResponse {
    fn cursor_value(&self, field: SortField) -> String {
        match field {
            SortField::CreatedAt => self.created_at.to_rfc3339(),
            SortField::Name => self.name.clone(),
        }
    }

    fn cursor_id(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Debug, Deserialize, Default, Clone, Copy, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiBoardSortField {
    #[default]
    CreatedAt,
    Name,
}

impl From<ApiBoardSortField> for SortField {
    fn from(f: ApiBoardSortField) -> Self {
        match f {
            ApiBoardSortField::CreatedAt => SortField::CreatedAt,
            ApiBoardSortField::Name => SortField::Name,
        }
    }
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct BoardFilterParams {
    pub search: Option<String>,
}

impl From<BoardFilterParams> for domain::boards::model::BoardFilter {
    fn from(f: BoardFilterParams) -> Self {
        domain::boards::model::BoardFilter { search: f.search }
    }
}
