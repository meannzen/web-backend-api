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
use domain::models::user::{Email, NewUser, UserId};
use domain::ports::user_repository::UserRepository;
use uuid::Uuid;

use crate::AppResult;
use crate::dtos::user::{CreateUserRequest, UserResponse};
use crate::extractors::ValidatedJson;

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
#[axum::debug_handler]
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
#[axum::debug_handler]
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
