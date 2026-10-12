use axum::{Router, routing::get};

use crate::shared::infrastructure::AppState;

use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(handlers::health))
}
