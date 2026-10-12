use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::shared::infrastructure::AppState;

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    database: &'static str,
}

pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(Health {
                status: "ok",
                database: "ok",
            }),
        ),
        Err(err) => {
            tracing::error!("database health check failed: {err}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(Health {
                    status: "degraded",
                    database: "error",
                }),
            )
        }
    }
}
