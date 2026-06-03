use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum::http::header;
use domain::users::model::Role;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use secrecy::ExposeSecret;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

pub const JWT_ISSUER: &str = "tpa";
pub const JWT_AUDIENCE: &str = "tpa-api";

fn deserialize_aud<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(s) => vec![s],
        OneOrMany::Many(v) => v,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: Role,
    pub iss: String,
    #[serde(deserialize_with = "deserialize_aud")]
    pub aud: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: Role,
}

pub fn make_validation() -> Validation {
    let mut v = Validation::new(Algorithm::HS256);
    v.set_issuer(&[JWT_ISSUER]);
    v.set_audience(&[JWT_AUDIENCE]);
    v.leeway = 30;
    v
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
        &make_validation(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let claims = token_data.claims;
    req.extensions_mut().insert(AuthUser {
        user_id: claims.sub,
        email: claims.email,
        role: claims.role,
    });

    Ok(next.run(req).await)
}
