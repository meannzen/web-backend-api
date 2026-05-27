use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::error::AppError;

fn invalid_cursor() -> AppError {
    AppError::Validation("invalid cursor".to_string())
}

pub fn encode(sort_field: &str, direction: &str, sort_value: &str, id: &str) -> String {
    let raw = format!("{}|{}|{}|{}", sort_field, direction, sort_value, id);
    URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

pub struct RawCursor {
    pub value: String,
    pub id: String,
}

pub fn decode(s: &str, expected_sort: &str, expected_dir: &str) -> Result<RawCursor, AppError> {
    let bytes = URL_SAFE_NO_PAD.decode(s).map_err(|_| invalid_cursor())?;
    let raw = String::from_utf8(bytes).map_err(|_| invalid_cursor())?;

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

    Ok(RawCursor { value: cursor_value.to_string(), id: cursor_id.to_string() })
}
