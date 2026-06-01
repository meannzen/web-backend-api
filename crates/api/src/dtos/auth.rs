use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, max = 100, message = "must be between 1 and 100 characters"))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100, message = "must be between 1 and 100 characters"))]
    pub last_name: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
}
