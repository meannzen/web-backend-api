use axum::extract::FromRef;
use infra::db::Database;
use shared::config::Config;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Database,
}

impl AppState {
    pub fn new(config: Config, db: Database) -> Self {
        AppState {
            config: Arc::new(config),
            db,
        }
    }
}
