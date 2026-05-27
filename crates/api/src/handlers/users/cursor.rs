use domain::users::model::{SortDirection, SortField, UserCursor, UserId};
use uuid::Uuid;

use crate::cursor;
use crate::dtos::common::{ApiSortDirection, ApiSortField};
use crate::dtos::user::UserResponse;
use crate::error::AppError;

pub(crate) fn encode(user: &UserResponse, sort_by: SortField, direction: SortDirection) -> String {
    let sort_value = match sort_by {
        SortField::CreatedAt => user.created_at.to_rfc3339(),
        SortField::Email => user.email.clone(),
    };
    cursor::encode(
        sort_by.column(),
        direction.as_str(),
        &sort_value,
        &user.id.to_string(),
    )
}

pub(crate) fn decode(
    s: &str,
    api_sort_by: ApiSortField,
    api_direction: ApiSortDirection,
) -> Result<UserCursor, AppError> {
    let sort_by = SortField::from(api_sort_by);
    let direction = SortDirection::from(api_direction);
    let raw = cursor::decode(s, sort_by.column(), direction.as_str())?;
    let value = sort_by
        .parse_cursor_value(&raw.value)
        .map_err(AppError::Validation)?;
    let id = UserId::from(
        Uuid::parse_str(&raw.id).map_err(|_| AppError::Validation("invalid cursor".to_string()))?,
    );
    Ok(UserCursor {
        sort_by,
        direction,
        value,
        id,
    })
}
