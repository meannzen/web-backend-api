use axum::{Router, routing::get, routing::post};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::health::{health_live, health_ready};
use crate::handlers::users::{create_user, get_user};
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
        crate::handlers::users::create_user,
        crate::handlers::users::get_user,
    ),
    components(schemas(
        crate::dtos::user::CreateUserRequest,
        crate::dtos::user::UserResponse,
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
        .nest("/api/v1", api_routes())
        .route("/health", get(health_live))
        .route("/health/ready", get(health_ready))
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/users", user_routes())
}

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_user))
        .route("/{id}", get(get_user))
}
