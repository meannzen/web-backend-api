use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(log_level: &str) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    let json_output = std::env::var("LOG_JSON")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let fmt_layer = if json_output {
        fmt::layer().json().boxed()
    } else {
        fmt::layer().pretty().boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
