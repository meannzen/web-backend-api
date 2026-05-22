use async_trait::async_trait;

#[async_trait]
pub trait HealthIndicator: Send + Sync + 'static {
    async fn ping(&self) -> bool;
}
