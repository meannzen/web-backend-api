use axum::http::{Method, header};
use shared::config::{Config, Environment};
use tower_http::cors::CorsLayer;

pub fn cors_layer(config: &Config) -> CorsLayer {
    match config.application.environment {
        Environment::Development => CorsLayer::permissive(),
        Environment::Staging | Environment::Production => {
            let origins: Vec<_> = config
                .application
                .cors_origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect();

            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                ])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                .allow_credentials(true)
        }
    }
}
