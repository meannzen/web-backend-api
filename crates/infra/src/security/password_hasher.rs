use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString},
};
use async_trait::async_trait;
use domain::ports::password_hasher::{HasherBusy, PasswordHasher};
use rand::rngs::OsRng;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// How long a request waits for a free hashing slot before giving up with `HasherBusy`.
const DEFAULT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(2);

/// Argon2id hasher that caps concurrent hashes. Each hash holds ~19 MiB and a full
/// core, so running more at once than there are cores adds memory and starves the
/// async runtime without raising throughput.
pub struct Argon2PasswordHasher {
    permits: Arc<Semaphore>,
    acquire_timeout: Duration,
}

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
        Self::with_limits(cores, DEFAULT_ACQUIRE_TIMEOUT)
    }

    pub fn with_limits(max_concurrent: usize, acquire_timeout: Duration) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(max_concurrent)),
            acquire_timeout,
        }
    }

    async fn acquire(&self) -> Result<OwnedSemaphorePermit, anyhow::Error> {
        tokio::time::timeout(self.acquire_timeout, self.permits.clone().acquire_owned())
            .await
            .map_err(|_| anyhow::Error::new(HasherBusy))?
            .context("password hasher semaphore closed")
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!("failed to hash password: {e}"))
}

fn verify_password(password: &str, hash: &str) -> Result<bool, anyhow::Error> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("invalid password hash format: {e}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: String) -> Result<String, anyhow::Error> {
        let permit = self.acquire().await?;
        tokio::task::spawn_blocking(move || {
            // Held inside the closure so the slot is released only when hashing ends,
            // even if the request future is dropped while the blocking task still runs.
            let _permit = permit;
            hash_password(&password)
        })
        .await
        .context("password hashing task failed")?
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, anyhow::Error> {
        let password = password.to_string();
        let hash = hash.to_string();
        let permit = self.acquire().await?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            verify_password(&password, &hash)
        })
        .await
        .context("password verification task failed")?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hash_then_verify_round_trips() {
        let hasher = Argon2PasswordHasher::new();
        let hash = hasher.hash("correct horse".to_string()).await.unwrap();
        assert!(hasher.verify("correct horse", &hash).await.unwrap());
        assert!(!hasher.verify("wrong horse", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn returns_busy_when_no_slot_frees_up_in_time() {
        let hasher = Argon2PasswordHasher::with_limits(0, Duration::from_millis(10));
        let err = hasher.hash("pw".to_string()).await.unwrap_err();
        assert!(err.is::<HasherBusy>());
        let err = hasher.verify("pw", "irrelevant").await.unwrap_err();
        assert!(err.is::<HasherBusy>());
    }
}
