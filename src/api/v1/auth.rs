use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/logout-all", post(logout_all))
        .route("/refresh", post(refresh))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/magic-link", post(send_magic_link))
        .route("/magic-link/callback", get(magic_link_callback))
        .route("/mfa/totp/setup", post(totp_setup))
        .route("/mfa/totp/verify-setup", post(totp_verify_setup))
        .route("/mfa/totp/verify", post(totp_verify))
        .route("/mfa/totp/disable", post(totp_disable))
        .route("/mfa/totp/regenerate-backup-codes", post(totp_regenerate_backup))
        .route("/mfa/totp/backup-codes", get(totp_backup_codes))
        .route("/passkey/register-options", post(passkey_register_options))
        .route("/passkey/register", post(passkey_register))
        .route("/passkey/auth-options", post(passkey_auth_options))
        .route("/passkey/login", post(passkey_login))
        .route("/mfa/verify", post(mfa_verify))
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

async fn register(
    State(_state): State<AppState>,
    Json(_req): Json<RegisterRequest>,
) -> ApiResult<TokenResponse> {
    todo!()
}

async fn login(
    State(_state): State<AppState>,
    Json(_req): Json<LoginRequest>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn logout(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn logout_all(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn refresh(State(_state): State<AppState>) -> ApiResult<TokenResponse> {
    todo!()
}

async fn forgot_password(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn reset_password(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn send_magic_link(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn magic_link_callback(
    State(_state): State<AppState>,
) -> ApiResult<TokenResponse> {
    todo!()
}

async fn totp_setup(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn totp_verify_setup(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn totp_verify(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<TokenResponse> {
    todo!()
}

async fn totp_disable(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn totp_regenerate_backup(
    State(_state): State<AppState>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn totp_backup_codes(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn passkey_register_options(
    State(_state): State<AppState>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn passkey_register(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn passkey_auth_options(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn passkey_login(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<TokenResponse> {
    todo!()
}

async fn mfa_verify(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<TokenResponse> {
    todo!()
}
