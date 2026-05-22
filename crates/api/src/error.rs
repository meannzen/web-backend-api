use std::collections::HashMap;

use axum::{Json, http::StatusCode, response::IntoResponse};

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

    #[error("insufficient permissions")]
    Forbidden,

    #[error("{0}")]
    Conflict(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<domain::error::UserError> for AppError {
    fn from(e: domain::error::UserError) -> Self {
        match e {
            domain::error::UserError::EmailTaken => {
                AppError::Conflict("email already taken".to_string())
            }
            domain::error::UserError::NotFound => AppError::NotFound,
            domain::error::UserError::InvalidEmail(msg) => AppError::Validation(msg),
            domain::error::UserError::Internal(e) => AppError::Internal(e),
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
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
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
