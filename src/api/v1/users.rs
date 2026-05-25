use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Path, State},
    routing::{delete, get, put},
    Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .route("/me/password", put(change_password))
        .route("/me/sessions", get(list_sessions))
        .route("/me/sessions/{id}", delete(revoke_session))
        .route("/me/mfa", get(get_mfa_status))
        .route("/me/passkeys", get(list_passkeys))
        .route("/me/passkeys/{id}", delete(delete_passkey).put(rename_passkey))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/", get(list_users))
}

async fn get_me(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn update_me(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn change_password(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn list_sessions(State(_state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn revoke_session(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn get_mfa_status(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn list_passkeys(State(_state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn delete_passkey(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn rename_passkey(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn get_user(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn update_user(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn delete_user(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn list_users(State(_state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}
