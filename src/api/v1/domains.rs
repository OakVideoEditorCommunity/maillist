use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_domains).post(create_domain))
        .route("/{id}", get(get_domain).put(update_domain).delete(delete_domain))
        .route("/{id}/verify-dkim", post(verify_dkim))
}

async fn list_domains(State(_state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn create_domain(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn get_domain(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn update_domain(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn delete_domain(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn verify_dkim(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}
