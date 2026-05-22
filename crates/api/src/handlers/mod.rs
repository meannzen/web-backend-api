use axum::{Router, routing::get, routing::post};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::health::{health_live, health_ready};
use crate::handlers::users::{create_user, get_user};
use crate::state::AppState;

pub mod health;
pub mod users;

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
        health::health_live,
        health::health_ready,
        users::create_user,
        users::get_user,
    ),
    components(schemas(
        users::CreateUserRequest,
        users::UserResponse,
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
        .nest("/api/v1", api_router())
        .route("/health", get(health_live))
        .route("/health_ready", get(health_ready))
        .with_state(state)
}

fn api_router() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user))
}
