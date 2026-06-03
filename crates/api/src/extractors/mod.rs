use std::sync::Arc;

use axum::{
    Json,
    extract::{FromRef, FromRequest, FromRequestParts, Request},
    http::{header, request::Parts},
};
use domain::users::model::Role;
use jsonwebtoken::{DecodingKey, decode};
use secrecy::ExposeSecret;
use serde::de::DeserializeOwned;
use shared::config::Config;
use validator::Validate;

use crate::error::AppError;
use crate::middleware::auth::{AuthUser, Claims, make_validation};

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::Validation(e.body_text()))?;

        value.validate().map_err(|e| {
            let fields = e
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    let messages = errors
                        .iter()
                        .map(|e| {
                            e.message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| e.code.to_string())
                        })
                        .collect();
                    (field.to_string(), messages)
                })
                .collect();
            AppError::ValidationFields(fields)
        })?;

        Ok(ValidatedJson(value))
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<Config>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Fast path: if middleware already decoded and inserted AuthUser into extensions
        if let Some(user) = parts.extensions.get::<AuthUser>() {
            return Ok(user.clone());
        }

        // Fall back: decode from the Authorization header directly
        let config = Arc::<Config>::from_ref(state);
        let secret = config.application.jwt_secret.as_ref().ok_or(AppError::Unauthorized)?;

        let token = parts
            .headers
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
        Ok(AuthUser { user_id: claims.sub, email: claims.email, role: claims.role })
    }
}

#[derive(Debug, Clone)]
pub struct MaybeAuthUser(pub Option<AuthUser>);

impl<S> FromRequestParts<S> for MaybeAuthUser
where
    S: Send + Sync,
    Arc<Config>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if !parts.headers.contains_key(header::AUTHORIZATION) {
            return Ok(MaybeAuthUser(None));
        }
        let user = AuthUser::from_request_parts(parts, state).await?;
        Ok(MaybeAuthUser(Some(user)))
    }
}

#[derive(Debug, Clone)]
pub struct RequireAdmin(pub AuthUser);

impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
    Arc<Config>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        if user.role != Role::Admin {
            return Err(AppError::Forbidden);
        }
        Ok(RequireAdmin(user))
    }
}
