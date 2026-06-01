use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use domain::ports::health_indicator::HealthIndicator;
use secrecy::ExposeSecret;
use shared::config::DatabaseSettings;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct Database {
    pool: Option<PgPool>,
    mock_ping: Option<bool>,
}

impl Database {
    pub async fn connect(settings: &DatabaseSettings) -> anyhow::Result<Database> {
        let pool = PgPoolOptions::new()
            .max_connections(settings.max_connections)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .connect(settings.connection_string().expose_secret())
            .await
            .context("failed to connect to the database")?;

        Ok(Database {
            pool: Some(pool),
            mock_ping: None,
        })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Database {
            pool: Some(pool),
            mock_ping: None,
        }
    }

    pub fn mock(ping_success: bool) -> Self {
        Database {
            pool: None,
            mock_ping: Some(ping_success),
        }
    }

    pub fn pool(&self) -> &PgPool {
        self.pool
            .as_ref()
            .expect("database pool is not available in mock mode")
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        if let Some(ref pool) = self.pool {
            sqlx::migrate!("../../migrations")
                .run(pool)
                .await
                .context("failed to run migrations")
        } else {
            Ok(())
        }
    }

    pub async fn close(&self) {
        if let Some(ref pool) = self.pool {
            pool.close().await;
        }
    }

    pub async fn ping(&self) -> bool {
        if let Some(status) = self.mock_ping {
            return status;
        }
        if let Some(ref pool) = self.pool {
            sqlx::query("SELECT 1").execute(pool).await.is_ok()
        } else {
            false
        }
    }
}

#[async_trait]
impl HealthIndicator for Database {
    async fn ping(&self) -> bool {
        self.ping().await
    }
}
