use std::time::Duration;

use anyhow::Context;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct Database(PgPool);

impl Database {
    pub async fn connect(url: &SecretString, max_connections: u32) -> anyhow::Result<Database> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .connect(url.expose_secret())
            .await
            .context("failed to connect to the database");

        Ok(Database(pool?))
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Database(pool)
    }

    pub fn pool(&self) -> &PgPool {
        &self.0
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.0)
            .await
            .context("failed to run migrations")
    }

    pub async fn ping(&self) -> bool {
        sqlx::query("SELECT 1").execute(&self.0).await.is_ok()
    }
}
