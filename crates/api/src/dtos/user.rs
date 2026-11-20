use chrono::{DateTime, Utc};
use domain::users::model::{Role, SortField};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::cursor::CursorSource;

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

impl CursorSource<SortField> for UserResponse {
    fn cursor_value(&self, field: SortField) -> String {
        match field {
            SortField::CreatedAt => self.created_at.to_rfc3339(),
            SortField::Email => self.email.clone(),
        }
    }

    fn cursor_id(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Debug, Deserialize, Clone, Copy, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApiRole {
    Teacher,
    Admin,
    Principal,
    Hr,
}

impl From<ApiRole> for Role {
    fn from(r: ApiRole) -> Self {
        match r {
            ApiRole::Teacher => Role::Teacher,
            ApiRole::Admin => Role::Admin,
            ApiRole::Principal => Role::Principal,
            ApiRole::Hr => Role::Hr,
        }
    }
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct UserFilterParams {
    pub search: Option<String>,
    pub role: Option<ApiRole>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_eq: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
}

impl From<UserFilterParams> for domain::users::model::UserFilter {
    fn from(f: UserFilterParams) -> Self {
        domain::users::model::UserFilter {
            search: f.search,
            role: f.role.map(Role::from),
            created_at_gt: f.created_at_gt,
            created_at_eq: f.created_at_eq,
            created_at_lt: f.created_at_lt,
        }
    }
}
