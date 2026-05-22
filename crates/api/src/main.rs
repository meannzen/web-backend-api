use api::handlers::router;
use api::shutdown::shutdown_signal;
use api::state::AppState;
use infra::db::Database;
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

    let state = AppState::new(config, db.clone());
    let token = CancellationToken::new();

    // spawn signal watcher — cancels the token on SIGINT or SIGTERM
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

    // HTTP server drains in-flight requests before returning
    axum::serve(listener, router(state))
        .with_graceful_shutdown(token.cancelled_owned())
        .await?;

    // ordered teardown: server stopped, now close DB
    tracing::info!("closing database pool");
    db.close().await;

    tracing::info!("shutdown complete");
    Ok(())
}
