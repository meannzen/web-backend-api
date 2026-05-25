use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::health::{health_live, health_ready};
use crate::handlers::users::{create_user, get_user, list_users};
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
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_live))
        .route("/health/ready", get(health_ready))
        .nest("/api/v1", v1_routes())
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
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
            }),
        )
        .with_state(state)
}

fn v1_routes() -> Router<AppState> {
    Router::new()
        .nest("/users", user_routes())
}

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user))
}
