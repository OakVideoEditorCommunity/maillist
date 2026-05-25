use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
}
