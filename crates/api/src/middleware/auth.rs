use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum::http::header;
use jsonwebtoken::{DecodingKey, Validation, decode};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        AuthUser {
            user_id: claims.sub.parse().unwrap_or_default(),
        }
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let secret = state
        .config
        .application
        .jwt_secret
        .as_ref()
        .ok_or(AppError::Unauthorized)?;

    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(AuthUser::from(token_data.claims));

    Ok(next.run(req).await)
}
