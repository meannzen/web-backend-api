use thiserror::Error;

#[derive(Debug, Error)]
pub enum BoardError {
    #[error("board code already taken")]
    CodeTaken,
    #[error("board not found")]
    NotFound,
    #[error("invalid board name: {0}")]
    InvalidName(String),
    #[error("invalid board code: {0}")]
    InvalidCode(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
