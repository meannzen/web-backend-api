use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email(message = "must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, max = 100, message = "must be between 1 and 100 characters"))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100, message = "must be between 1 and 100 characters"))]
    pub last_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<domain::users::model::User> for UserResponse {
    fn from(u: domain::users::model::User) -> Self {
        UserResponse {
            id: *u.id().as_uuid(),
            email: u.email().as_ref().to_string(),
            first_name: u.first_name().to_string(),
            last_name: u.last_name().to_string(),
            role: u.role().as_str().to_string(),
            created_at: u.created_at(),
            updated_at: u.updated_at(),
        }
    }
}
