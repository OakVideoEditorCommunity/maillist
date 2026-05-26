use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

pub fn routes() -> Router<AppState> {
    Router::new()
}
