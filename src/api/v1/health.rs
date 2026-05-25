use axum::{extract::State, response::Json};
use serde_json::json;
use crate::models::AppState;

pub async fn health_check(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn readiness_check(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({ "status": "ready" }))
}

pub async fn liveness_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "alive" }))
}

pub async fn metrics_handler() -> String {
    "# metrics endpoint\n".to_string()
}
