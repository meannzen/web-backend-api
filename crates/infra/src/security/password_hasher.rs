use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString},
};
use async_trait::async_trait;
use domain::ports::password_hasher::PasswordHasher;
use rand::rngs::OsRng;

pub struct Argon2PasswordHasher;

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: String) -> Result<String, anyhow::Error> {
        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|e| anyhow::anyhow!("failed to hash password: {}", e))
        })
        .await
        .map_err(|e| anyhow::anyhow!("spawn_blocking failed: {}", e))?
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, anyhow::Error> {
        let password_owned = password.to_string();
        let hash_owned = hash.to_string();
        tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash_owned)
                .map_err(|e| anyhow::anyhow!("invalid password hash format: {}", e))?;

            Ok(Argon2::default()
                .verify_password(password_owned.as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await
        .map_err(|e| anyhow::anyhow!("spawn_blocking failed: {}", e))?
    }
}
