use thiserror::Error;

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
    Busy,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
