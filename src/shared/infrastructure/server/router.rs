use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use super::{AppState, openapi::ApiDoc};
use crate::{auth, health};

pub fn build(state: AppState) -> Router {
    let (api_router, api_doc) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/auth", auth::presentation::router())
        .split_for_parts();

    Router::new()
        .merge(api_router)
        .merge(health::router())
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api_doc))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
