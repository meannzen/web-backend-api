use std::time::Duration;

use thiserror::Error;

use crate::ports::password_hasher::PasswordHasherError;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("email already taken")]
    EmailTaken,
    #[error("user not found")]
    NotFound,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("invalid email: {0}")]
    InvalidEmail(String),
    #[error("service is busy, retry later")]
    Busy { retry_after: Duration },
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<PasswordHasherError> for UserError {
    fn from(e: PasswordHasherError) -> Self {
        match e {
            PasswordHasherError::Busy { retry_after } => UserError::Busy { retry_after },
            PasswordHasherError::Internal(e) => UserError::Internal(e),
        }
    }
}
