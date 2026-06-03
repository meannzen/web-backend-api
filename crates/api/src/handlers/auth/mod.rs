use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{Json, extract::State, http::StatusCode};
use domain::users::model::Role;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use secrecy::ExposeSecret;
use domain::users::service::UserService;

use crate::AppResult;
use crate::dtos::auth::{LoginRequest, RegisterRequest, TokenResponse};
use crate::dtos::user::UserResponse;
use crate::error::AppError;
use crate::extractors::ValidatedJson;
use crate::middleware::auth::{Claims, JWT_AUDIENCE, JWT_ISSUER};
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = UserResponse),
        (status = 400, description = "Invalid input", body = crate::dtos::common::ErrorResponse),
        (status = 409, description = "Email already taken", body = crate::dtos::common::ErrorResponse),
    )
)]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn register(
    State(user_service): State<Arc<UserService>>,
    ValidatedJson(req): ValidatedJson<RegisterRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let user = user_service
        .create(&req.email, req.password, req.first_name, req.last_name, Role::User)
        .await?;
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials", body = crate::dtos::common::ErrorResponse),
    )
)]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    let user = state.user_service.authenticate(&req.email, req.password).await?;

    let secret = state
        .config
        .application
        .jwt_secret
        .as_ref()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("jwt_secret not configured")))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize;

    let claims = Claims {
        sub: *user.id().as_uuid(),
        email: user.email().as_ref().to_string(),
        role: user.role().clone(),
        iss: JWT_ISSUER.to_string(),
        aud: vec![JWT_AUDIENCE.to_string()],
        iat: now,
        exp: now + 15 * 60,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.expose_secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e).context("failed to encode jwt")))?;

    Ok(Json(TokenResponse { token }))
}
