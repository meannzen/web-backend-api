use crate::error::AppError;

pub mod cursor;
pub mod dtos;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod shutdown;
pub mod state;

pub type AppResult<T> = Result<T, AppError>;
