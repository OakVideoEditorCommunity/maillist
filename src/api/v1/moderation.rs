use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_moderation_item))
        .route("/{id}/approve", post(approve))
        .route("/{id}/reject", post(reject))
        .route("/{id}/discard", post(discard))
        .route("/{id}/whitelist-sender", post(whitelist_sender))
        .route("/{id}/blacklist-sender", post(blacklist_sender))
        .route("/{id}/ai-feedback", post(ai_feedback))
}

async fn get_moderation_item(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn approve(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn reject(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn discard(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn whitelist_sender(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn blacklist_sender(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn ai_feedback(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}
