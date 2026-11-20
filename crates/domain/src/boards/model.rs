use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::pagination;

#[derive(Debug, Clone)]
pub struct BoardId(Uuid);

impl BoardId {
    pub fn new() -> Self {
        BoardId(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<Uuid> for BoardId {
    fn from(id: Uuid) -> Self {
        BoardId(id)
    }
}

impl Default for BoardId {
    fn default() -> Self {
        Self::new()
    }
}

/// Short identifier for a board (e.g. the Ministry board number or "TDSB").
/// Stored uppercase; allowed characters are A-Z, 0-9, '-' and '_'.
#[derive(Debug, Clone)]
pub struct BoardCode(String);

impl BoardCode {
    pub const MAX_LEN: usize = 20;

    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim().to_uppercase();
        if s.is_empty() || s.len() > Self::MAX_LEN {
            return Err(format!(
                "must be between 1 and {} characters",
                Self::MAX_LEN
            ));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err("may only contain letters, digits, '-' and '_'".to_string());
        }
        Ok(BoardCode(s))
    }
}

impl AsRef<str> for BoardCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct NewBoard {
    pub name: String,
    pub code: BoardCode,
}

#[derive(Debug, Clone)]
pub struct Board {
    id: BoardId,
    name: String,
    code: BoardCode,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Board {
    pub fn new(
        id: BoardId,
        data: NewBoard,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name: data.name,
            code: data.code,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> &BoardId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn code(&self) -> &BoardCode {
        &self.code
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    CreatedAt,
    Name,
}

impl pagination::SortField for SortField {
    fn column(&self) -> &'static str {
        match self {
            SortField::CreatedAt => "created_at",
            SortField::Name => "name",
        }
    }

    fn parse_cursor_value(&self, s: &str) -> Result<pagination::CursorValue, String> {
        match self {
            SortField::CreatedAt => {
                let ts = DateTime::parse_from_rfc3339(s)
                    .map_err(|_| "invalid cursor".to_string())?
                    .to_utc();
                Ok(pagination::CursorValue::Timestamp(ts))
            }
            SortField::Name => Ok(pagination::CursorValue::Text(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoardFilter {
    /// Substring match against name or code.
    pub search: Option<String>,
}

pub type BoardListQuery = pagination::ListQuery<BoardFilter, SortField>;
pub type BoardCursor = pagination::Cursor<SortField>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_code_is_trimmed_and_uppercased() {
        let code = BoardCode::parse("  tdsb-01 ").unwrap();
        assert_eq!(code.as_ref(), "TDSB-01");
    }

    #[test]
    fn board_code_rejects_empty() {
        assert!(BoardCode::parse("   ").is_err());
    }

    #[test]
    fn board_code_rejects_too_long() {
        assert!(BoardCode::parse(&"A".repeat(BoardCode::MAX_LEN + 1)).is_err());
        assert!(BoardCode::parse(&"A".repeat(BoardCode::MAX_LEN)).is_ok());
    }

    #[test]
    fn board_code_rejects_invalid_characters() {
        assert!(BoardCode::parse("TD SB").is_err());
        assert!(BoardCode::parse("TDSB!").is_err());
        assert!(BoardCode::parse("ÉCOLE").is_err());
    }
}
