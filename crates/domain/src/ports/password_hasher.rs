use async_trait::async_trait;

/// Returned (inside `anyhow::Error`) when the hasher has no free slot within its wait limit.
#[derive(Debug, thiserror::Error)]
#[error("password hasher is at capacity")]
pub struct HasherBusy;

#[async_trait]
pub trait PasswordHasher: Send + Sync + 'static {
    async fn hash(&self, password: String) -> Result<String, anyhow::Error>;
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, anyhow::Error>;
}
