use async_trait::async_trait;

#[async_trait]
pub trait PasswordHasher: Send + Sync + 'static {
    async fn hash(&self, password: String) -> Result<String, anyhow::Error>;
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, anyhow::Error>;
}
