use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("email already taken")]
    EmailTaken,
    #[error("user not found")]
    NotFound,
    #[error("invalid email: {0}")]
    InvalidEmail(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
