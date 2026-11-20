use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use domain::pagination::{Cursor, SortDirection, SortField};
use uuid::Uuid;

use crate::error::AppError;

fn invalid_cursor() -> AppError {
    AppError::Validation("invalid cursor".to_string())
}

struct RawCursor {
    value: String,
    id: String,
}

fn decode_raw(s: &str, expected_sort: &str, expected_dir: &str) -> Result<RawCursor, AppError> {
    let bytes = URL_SAFE_NO_PAD.decode(s).map_err(|_| invalid_cursor())?;
    let raw = String::from_utf8(bytes).map_err(|_| invalid_cursor())?;
    if raw.contains('\0') {
        return Err(invalid_cursor());
    }

    // format: sort_field|direction|sort_value|uuid
    // splitn(4) keeps sort_value whole even if it ever contains '|'
    let parts: Vec<&str> = raw.splitn(4, '|').collect();
    if parts.len() != 4 {
        return Err(invalid_cursor());
    }
    let (cursor_sort, cursor_dir, cursor_value, cursor_id) = (parts[0], parts[1], parts[2], parts[3]);

    if cursor_sort != expected_sort || cursor_dir != expected_dir {
        return Err(AppError::Validation(
            "cursor sort order does not match request parameters".to_string(),
        ));
    }

    Ok(RawCursor {
        value: cursor_value.to_string(),
        id: cursor_id.to_string(),
    })
}

/// Implemented by response DTOs so a generic cursor can be derived from them
/// without each resource writing its own encode/decode glue.
pub trait CursorSource<S> {
    fn cursor_value(&self, field: S) -> String;
    fn cursor_id(&self) -> String;
}

pub fn encode<S: SortField, T: CursorSource<S>>(item: &T, sort_by: S, direction: SortDirection) -> String {
    let raw = format!(
        "{}|{}|{}|{}",
        sort_by.column(),
        direction.as_str(),
        item.cursor_value(sort_by),
        item.cursor_id(),
    );
    URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

pub fn decode<S: SortField>(
    s: &str,
    sort_by: S,
    direction: SortDirection,
) -> Result<Cursor<S>, AppError> {
    let raw = decode_raw(s, sort_by.column(), direction.as_str())?;
    let value = sort_by
        .parse_cursor_value(&raw.value)
        .map_err(AppError::Validation)?;
    let id = Uuid::parse_str(&raw.id).map_err(|_| invalid_cursor())?;
    Ok(Cursor {
        sort_by,
        direction,
        value,
        id,
    })
}
