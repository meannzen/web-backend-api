use std::sync::Arc;
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::{Router, middleware, routing::get};
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::CompressionLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::health::{health_live, health_ready};
use crate::handlers::users::{create_user, get_user, list_users};
use crate::middleware::{auth::auth_middleware, cors::cors_layer, timing::timing_middleware};
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "TPA API",
        version = "1.0.0",
        description = "REST API for the TPA platform",
        contact(name = "TPA Team", email = "meann.sen@realwat.net"),
        license(name = "MIT"),
    ),
    paths(
        crate::handlers::health::health_live,
        crate::handlers::health::health_ready,
        crate::handlers::users::list_users,
        crate::handlers::users::create_user,
        crate::handlers::users::get_user,
    ),
    components(schemas(
        crate::dtos::user::CreateUserRequest,
        crate::dtos::user::UserResponse,
        crate::dtos::common::CursorPaginationMeta,
        crate::dtos::common::ApiSortField,
        crate::dtos::common::ApiSortDirection,
        crate::dtos::common::ErrorResponse,
        crate::dtos::common::ErrorDetail,
    )),
    tags(
        (name = "Health", description = "Liveness and readiness probes"),
        (name = "Users", description = "User management"),
    ),
    servers(
        (url = "/", description = "Current server"),
    ),
)]
struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let config = Arc::clone(&state.config);

    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(public_routes())
        .merge(protected_routes(state.clone()))
        // Layers applied innermost-first (last .layer() = outermost):
        // GovernorLayer innermost: wraps Route directly, avoids body-type conflicts
        .layer(GovernorLayer::new(governor_config))
        // DefaultBodyLimit: axum-native limit that doesn't change the body type
        .layer(DefaultBodyLimit::max(1024 * 1024))
        // Outer stack via ServiceBuilder: compression, tracing, request-id, timeout, cors
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(
                    TraceLayer::new_for_http().make_span_with(
                        |request: &axum::http::Request<_>| {
                            let request_id = request
                                .headers()
                                .get("x-request-id")
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("unknown");

                            let matched_path = request
                                .extensions()
                                .get::<axum::extract::MatchedPath>()
                                .map(|p| p.as_str().to_string())
                                .unwrap_or_else(|| request.uri().path().to_string());

                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                path = %matched_path,
                                request_id = %request_id,
                            )
                        },
                    ),
                )
                .layer(PropagateRequestIdLayer::x_request_id())
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    Duration::from_secs(30),
                ))
                .layer(middleware::from_fn(timing_middleware))
                .layer(cors_layer(&config)),
        )
        .with_state(state)
}

fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_live))
        .route("/health/ready", get(health_ready))
}

fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/api/v1", v1_routes())
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

fn v1_routes() -> Router<AppState> {
    Router::new().nest("/users", user_routes())
}

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user))
}
