use std::sync::Arc;

use axum::extract::FromRef;
use domain::ports::user_repository::UserRepository;
use infra::db::Database;
use shared::config::Config;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Database,
    pub user_repo: Arc<dyn UserRepository>,
}

impl AppState {
    pub fn new(config: Config, db: Database, user_repo: Arc<dyn UserRepository>) -> Self {
        AppState {
            config: Arc::new(config),
            db,
            user_repo,
        }
    }
}
