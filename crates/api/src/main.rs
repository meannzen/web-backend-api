use std::sync::Arc;

use api::handlers::router;
use api::shutdown::shutdown_signal;
use api::state::AppState;
use infra::db::Database;
use infra::repositories::user_repository::PgUserRepository;
use shared::config::Config;
use shared::observability;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    observability::init_tracing(&config.application.log_level);

    let db = Database::connect(&config.database).await?;
    db.migrate().await?;

    let user_repo = Arc::new(PgUserRepository::new(db.pool().clone()));
    let state = AppState::new(config, db.clone(), user_repo);

    let token = CancellationToken::new();
    let signal_token = token.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        signal_token.cancel();
    });

    let addr = format!(
        "{}:{}",
        state.config.application.host, state.config.application.port
    );
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, router(state))
        .with_graceful_shutdown(token.cancelled_owned())
        .await?;

    tracing::info!("closing database pool");
    db.close().await;

    tracing::info!("shutdown complete");
    Ok(())
}
