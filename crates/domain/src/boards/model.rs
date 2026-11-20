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

/// Display name of a board. Trimmed; must be non-blank and at most `MAX_LEN` characters.
#[derive(Debug, Clone)]
pub struct BoardName(String);

impl BoardName {
    pub const MAX_LEN: usize = 200;

    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("must not be blank".to_string());
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(format!("must be at most {} characters", Self::MAX_LEN));
        }
        Ok(BoardName(s.to_string()))
    }
}

impl AsRef<str> for BoardName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Short identifier for a board (e.g. the Ministry board number or "TDSB").
/// Stored uppercase; allowed characters are A-Z, 0-9, '-' and '_'.
#[derive(Debug, Clone)]
pub struct BoardCode(String);

impl BoardCode {
    pub const MAX_LEN: usize = 20;

    pub fn parse(s: &str) -> Result<Self, String> {
        // Validate the raw input before uppercasing: Unicode case mapping can turn
        // non-ASCII characters into ASCII ones (e.g. 'ß' -> "SS").
        let s = s.trim();
        if s.is_empty() || s.len() > Self::MAX_LEN {
            return Err(format!(
                "must be between 1 and {} characters",
                Self::MAX_LEN
            ));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("may only contain letters, digits, '-' and '_'".to_string());
        }
        Ok(BoardCode(s.to_ascii_uppercase()))
    }
}

impl AsRef<str> for BoardCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct NewBoard {
    pub name: BoardName,
    pub code: BoardCode,
}

#[derive(Debug, Clone)]
pub struct Board {
    id: BoardId,
    name: BoardName,
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
        self.name.as_ref()
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
        // Would uppercase to ASCII "STRASSE" / "FF" if checked after uppercasing.
        assert!(BoardCode::parse("straße").is_err());
        assert!(BoardCode::parse("ﬀ").is_err());
    }

    #[test]
    fn board_name_is_trimmed() {
        let name = BoardName::parse("  Toronto DSB ").unwrap();
        assert_eq!(name.as_ref(), "Toronto DSB");
    }

    #[test]
    fn board_name_rejects_blank() {
        assert!(BoardName::parse("").is_err());
        assert!(BoardName::parse("   ").is_err());
    }

    #[test]
    fn board_name_length_counts_characters() {
        assert!(BoardName::parse(&"é".repeat(BoardName::MAX_LEN)).is_ok());
        assert!(BoardName::parse(&"é".repeat(BoardName::MAX_LEN + 1)).is_err());
    }
}
