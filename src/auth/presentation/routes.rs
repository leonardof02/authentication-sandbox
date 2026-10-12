use axum::routing::get;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::handlers;
use crate::shared::infrastructure::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::register))
        .routes(routes!(handlers::login))
        .routes(routes!(handlers::me))
        .routes(routes!(handlers::start_passkey_registration))
        .routes(routes!(handlers::finish_passkey_registration))
        .routes(routes!(handlers::start_passkey_login))
        .routes(routes!(handlers::finish_passkey_login))
        .route("/passkeys/demo", get(handlers::passkeys_demo))
}
