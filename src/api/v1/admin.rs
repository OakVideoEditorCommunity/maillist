use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Query, State},
    routing::{get, put},
    Json, Router,
};
use std::collections::HashMap;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/stats", get(stats))
        .route("/activity-log", get(activity_log))
        .route("/settings", get(get_settings).put(update_settings))
        .route("/ai-moderation/stats", get(ai_moderation_stats))
}

async fn dashboard(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn stats(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn activity_log(
    State(_state): State<AppState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn get_settings(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn update_settings(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn ai_moderation_stats(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}
