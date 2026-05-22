use crate::error::AppError;

pub mod error;
pub mod handlers;
pub mod shutdown;
pub mod state;

pub type AppResult<T> = Result<T, AppError>;
