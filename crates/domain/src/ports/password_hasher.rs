use std::time::Duration;

use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum PasswordHasherError {
    /// No hashing slot freed up within the wait limit; callers should retry after `retry_after`.
    #[error("password hasher is at capacity")]
    Busy { retry_after: Duration },
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait]
pub trait PasswordHasher: Send + Sync + 'static {
    async fn hash(&self, password: String) -> Result<String, PasswordHasherError>;
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordHasherError>;
}
