use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum CursorValue {
    Timestamp(DateTime<Utc>),
    Text(String),
}

pub trait SortField: Copy + Send + Sync + 'static {
    fn column(&self) -> &'static str;
    fn parse_cursor_value(&self, s: &str) -> Result<CursorValue, String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn sql_order(&self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }

    pub fn cursor_op(&self) -> &'static str {
        match self {
            SortDirection::Asc => ">",
            SortDirection::Desc => "<",
        }
    }
}

pub struct Cursor<S: SortField> {
    pub sort_by: S,
    pub direction: SortDirection,
    pub value: CursorValue,
    pub id: Uuid,
}

pub struct ListQuery<F, S: SortField> {
    pub filter: F,
    pub sort_by: S,
    pub direction: SortDirection,
}
