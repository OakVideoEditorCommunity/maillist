use crate::models::AppState;
use crate::services::auth_service::AuthService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{ConnectInfo, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

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

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Serialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub session_token: String,
    pub available_methods: Vec<String>,
    pub expires_in: i64,
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<serde_json::Value> {
    if req.password.len() < state.config.security.password_min_length {
        return Err(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: format!(
                "Password must be at least {} characters",
                state.config.security.password_min_length
            ),
            details: None,
            request_id: None,
        });
    }

    let service = AuthService::new(state.db.clone(), state.config.clone());
    let user = service
        .register(&req.email, &req.password, req.name.as_deref())
        .await
        .map_err(|e| ApiError {
            code: "CONFLICT".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "created_at": user.created_at,
    }))))
}

async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let service = AuthService::new(state.db.clone(), state.config.clone());

    let user = service
        .login(&req.email, &req.password)
        .await
        .map_err(|_| ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Invalid email or password".to_string(),
            details: None,
            request_id: None,
        })?;

    if user.mfa_enabled {
        return Err(ApiError {
            code: "MFA_REQUIRED".to_string(),
            message: "Multi-factor authentication required".to_string(),
            details: Some(serde_json::json!({
                "user_id": user.id,
            })),
            request_id: None,
        });
    }

    let access_token = service
        .generate_access_token(&user.id.to_string(), &user.email, "subscriber")
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let refresh_token = service
        .create_refresh_token(&user.id.to_string(), Some(&addr.ip().to_string()))
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "Bearer",
        "expires_in": state.config.security.jwt_expiration_seconds,
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
        }
    })))
}

async fn logout(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn logout_all(State(_state): State<AppState>) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let service = AuthService::new(state.db.clone(), state.config.clone());

    let user = service
        .verify_refresh_token(&req.refresh_token)
        .await
        .map_err(|_| ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Invalid or expired refresh token".to_string(),
            details: None,
            request_id: None,
        })?;

    let access_token = service
        .generate_access_token(&user.id.to_string(), &user.email, "subscriber")
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let refresh_token = service
        .create_refresh_token(&user.id.to_string(), None)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    service
        .revoke_refresh_token(&req.refresh_token)
        .await
        .ok();

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "Bearer",
        "expires_in": state.config.security.jwt_expiration_seconds,
    })))
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
