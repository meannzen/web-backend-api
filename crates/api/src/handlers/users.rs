use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use domain::models::user::{Email, NewUser, UserId};
use domain::ports::user_repository::UserRepository;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::AppResult;
use crate::extractors::ValidatedJson;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email(message = "must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<domain::models::user::User> for UserResponse {
    fn from(u: domain::models::user::User) -> Self {
        UserResponse {
            id: *u.id().as_uuid(),
            email: u.email().as_ref().to_string(),
            created_at: u.created_at(),
            updated_at: u.updated_at(),
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 409, description = "Email already taken"),
        (status = 400, description = "Invalid input"),
    )
)]
pub async fn create_user(
    State(user_repo): State<Arc<dyn UserRepository>>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let email = Email::parse(&req.email).map_err(crate::error::AppError::Validation)?;
    let password_hash = hash_password(req.password).await?;
    let new_user = NewUser {
        email,
        password_hash,
    };

    let user = user_repo.create(new_user).await?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found"),
    )
)]
pub async fn get_user(
    State(user_repo): State<Arc<dyn UserRepository>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let user = user_repo.find_by_id(&UserId::from(id)).await?;
    Ok(Json(UserResponse::from(user)))
}

async fn hash_password(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| {
                crate::error::AppError::Internal(anyhow::anyhow!("failed to hash password: {}", e))
            })
    })
    .await
    .map_err(|e| {
        crate::error::AppError::Internal(anyhow::anyhow!("spawn_blocking failed: {}", e))
    })?
}
