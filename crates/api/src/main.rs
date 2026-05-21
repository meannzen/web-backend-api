use api::handlers::router;
use api::state::AppState;
use infra::db::Database;
use shared::config::Config;
use shared::observability;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    observability::init_tracing(&config.server.log_level);

    let db = Database::connect(&config.database.url, config.database.max_connections).await?;
    db.migrate().await?;
    let state = AppState::new(config, db);

    let addr = format!("127.0.0.1:{}", state.config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    let app = router(state);

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())
}
