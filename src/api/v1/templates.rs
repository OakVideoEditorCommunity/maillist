use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_templates))
        .route("/{name}", get(get_template).put(update_template))
        .route("/{name}/preview", post(preview_template))
}

async fn list_templates(State(_state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn get_template(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn update_template(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn preview_template(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}
