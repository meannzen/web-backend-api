use std::time::Duration;

use api::routes::router;
use api::shutdown::shutdown_signal;
use api::state::AppState;
use infra::db::Database;
use shared::config::Config;
use shared::observability;
use tokio::net::TcpListener;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    observability::init_tracing(&config.application.log_level);

    let db = Database::connect(&config.database).await?;
    db.migrate().await?;

    let root_token = CancellationToken::new();
    let http_token = root_token.clone();

    tokio::spawn({
        let signal_token = root_token.clone();
        async move {
            shutdown_signal().await;
            tracing::info!("Shutdown signal received. Cancelling tokens...");
            signal_token.cancel();
        }
    });

    let http_handle = tokio::spawn({
        let state = AppState::new(config.clone(), db.clone());
        let addr = format!(
            "{}:{}",
            state.config.application.host, state.config.application.port
        );
        async move {
            let listener = TcpListener::bind(&addr).await.expect("failed to bind");
            tracing::info!(
                "listening on {}",
                listener.local_addr().expect("address error")
            );

            axum::serve(listener, router(state).into_make_service_with_connect_info::<std::net::SocketAddr>())
                .with_graceful_shutdown(http_token.cancelled_owned())
                .await
                .expect("server error");
        }
    });
    // example backegrond task
    let worker_token = root_token.clone();
    let worker = tokio::spawn({
        async {
            schedule_job(worker_token).await;
        }
    });

    let _ = tokio::join!(http_handle, worker);

    tracing::info!("closing database pool");
    db.close().await;

    tracing::info!("shutdown complete");
    Ok(())
}

async fn schedule_job(shutdown: CancellationToken) {
    let mut ticker = interval(Duration::from_secs(5));
    loop {
        tokio::select! {
            _ = ticker.tick()=> {
                tracing::info!("do something");
            },
            _= shutdown.cancelled()=> {
                tracing::info!("cache cleanup shutting down");
                return;
            }
        }
    }
}
