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

use crate::dtos::common::{ApiSortDirection, ApiSortField};
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
        let secret = config
            .application
            .jwt_secret
            .as_ref()
            .ok_or(AppError::Unauthorized)?;

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
        Ok(AuthUser {
            user_id: claims.sub,
            email: claims.email,
            role: claims.role,
        })
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

pub struct Pagination<F = crate::dtos::common::EmptyFilter, S = ApiSortField, D = ApiSortDirection>
{
    pub limit: u32,
    pub after: Option<String>,
    pub filter: F,
    pub sort_by: S,
    pub sort_direction: D,
}

impl<F, S, T, D> FromRequestParts<T> for Pagination<F, S, D>
where
    T: Send + Sync,
    F: DeserializeOwned,
    S: DeserializeOwned + Default,
    D: DeserializeOwned + Default,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &T) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or_default();
        let params = serde_urlencoded::from_str::<
            crate::dtos::common::CursorPaginationParams<F, S, D>,
        >(query)
        .map_err(|e| AppError::Validation(describe_query_error::<F, S, D>(query, &e)))?;

        Ok(Pagination {
            limit: params.limit.clamp(1, 100),
            after: params.after,
            filter: params.filter,
            sort_by: params.sort_by,
            sort_direction: params.sort_direction,
        })
    }
}

/// serde's `flatten` drops field names from errors, so find the offending
/// parameter by re-parsing each `key=value` pair on its own (all fields are
/// optional/defaulted, so a single valid pair always parses).
fn describe_query_error<F, S, D>(query: &str, original: &serde_urlencoded::de::Error) -> String
where
    F: DeserializeOwned,
    S: DeserializeOwned + Default,
    D: DeserializeOwned + Default,
{
    let pairs = serde_urlencoded::from_str::<Vec<(String, String)>>(query).unwrap_or_default();
    for (key, value) in pairs {
        let Ok(single) = serde_urlencoded::to_string([(&key, &value)]) else {
            continue;
        };
        if let Err(e) = serde_urlencoded::from_str::<
            crate::dtos::common::CursorPaginationParams<F, S, D>,
        >(&single)
        {
            let reason = query_param_hint(&key).map_or_else(|| e.to_string(), str::to_string);
            return format!(
                "invalid query parameter `{key}` (value `{}`): {reason}",
                truncate(&value, MAX_ECHOED_VALUE_LEN)
            );
        }
    }
    format!("invalid query string: {original}")
}

const MAX_ECHOED_VALUE_LEN: usize = 100;

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        let head: String = value.chars().take(max_chars).collect();
        format!("{head}...")
    }
}

/// Client-facing explanation of the expected format for known query parameters.
fn query_param_hint(key: &str) -> Option<&'static str> {
    match key {
        "created_at_gt" | "created_at_eq" | "created_at_lt" => {
            Some("expected an RFC 3339 timestamp, e.g. 2026-11-20T04:00:00Z")
        }
        "role" => Some("expected one of: teacher, admin, principal, hr"),
        "limit" => Some("expected a non-negative integer"),
        // sort_by has no hint: the valid fields differ per resource, and serde's
        // "unknown variant" error already lists them.
        "sort_direction" => Some("expected one of: asc, desc"),
        _ => None,
    }
}
