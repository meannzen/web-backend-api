use std::collections::HashMap;

use axum::{
    Json,
    http::{StatusCode, header},
    response::IntoResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,

    #[error("{0}")]
    Validation(String),

    #[error("validation failed")]
    ValidationFields(HashMap<String, Vec<String>>),

    #[error("authentication required")]
    Unauthorized,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("insufficient permissions")]
    Forbidden,

    #[error("{0}")]
    Conflict(String),

    #[error("service is busy, retry later")]
    ServiceUnavailable,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<domain::boards::errors::BoardError> for AppError {
    fn from(e: domain::boards::errors::BoardError) -> Self {
        match e {
            domain::boards::errors::BoardError::CodeTaken => {
                AppError::Conflict("board code already taken".to_string())
            }
            domain::boards::errors::BoardError::NotFound => AppError::NotFound,
            domain::boards::errors::BoardError::InvalidName(msg) => {
                AppError::Validation(format!("invalid board name: {msg}"))
            }
            domain::boards::errors::BoardError::InvalidCode(msg) => {
                AppError::Validation(format!("invalid board code: {msg}"))
            }
            domain::boards::errors::BoardError::Internal(e) => AppError::Internal(e),
        }
    }
}

impl From<domain::users::errors::UserError> for AppError {
    fn from(e: domain::users::errors::UserError) -> Self {
        match e {
            domain::users::errors::UserError::EmailTaken => {
                AppError::Conflict("email already taken".to_string())
            }
            domain::users::errors::UserError::NotFound => AppError::NotFound,
            domain::users::errors::UserError::InvalidCredentials => AppError::InvalidCredentials,
            domain::users::errors::UserError::InvalidEmail(msg) => AppError::Validation(msg),
            domain::users::errors::UserError::Busy => AppError::ServiceUnavailable,
            domain::users::errors::UserError::Internal(e) => AppError::Internal(e),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_type, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", self.to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg.clone()),
            AppError::ValidationFields(fields) => {
                let body = serde_json::json!({
                    "error": {
                        "type": "validation_error",
                        "message": "request validation failed",
                        "fields": fields,
                    }
                });
                return (StatusCode::BAD_REQUEST, Json(body)).into_response();
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string()),
            AppError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string())
            }
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::ServiceUnavailable => {
                let body = serde_json::json!({
                    "error": {
                        "type": "service_unavailable",
                        "message": self.to_string(),
                    }
                });
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    [(header::RETRY_AFTER, "1")],
                    Json(body),
                )
                    .into_response();
            }
            AppError::Internal(err) => {
                tracing::error!(error = ?err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "an internal error occurred".to_string(),
                )
            }
        };

        let body = serde_json::json!({
          "error" : {
              "type": error_type,
              "message": message
          }
        });

        (status, Json(body)).into_response()
    }
}
