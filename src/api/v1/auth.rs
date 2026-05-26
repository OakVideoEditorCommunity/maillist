use crate::api::middleware::auth::Claims;
use crate::models::AppState;
use crate::services::auth_service::AuthService;
use crate::services::mfa_service::MfaService;
use crate::services::passkey_service::PasskeyService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    routing::{get, post},
};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use webauthn_rs::prelude::{Base64UrlSafeData, PublicKeyCredential, RegisterPublicKeyCredential};

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
        .route(
            "/mfa/totp/regenerate-backup-codes",
            post(totp_regenerate_backup),
        )
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

#[derive(Deserialize)]
pub struct PasskeyRegisterOptionsRequest {
    pub device_name: Option<String>,
}

#[derive(Deserialize)]
pub struct PasskeyRegisterRequest {
    pub challenge_id: String,
    pub credential: RegisterPublicKeyCredential,
}

#[derive(Deserialize)]
pub struct PasskeyAuthOptionsRequest {
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct PasskeyLoginRequest {
    pub challenge_id: String,
    pub credential: PublicKeyCredential,
}

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

fn verify_token(state: &AppState, headers: &HeaderMap) -> Result<Claims, ApiError> {
    let token = extract_bearer(headers).ok_or(ApiError {
        code: "UNAUTHORIZED".to_string(),
        message: "Missing authorization header".to_string(),
        details: None,
        request_id: None,
    })?;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    let tc = svc.verify_access_token(token).map_err(|_| ApiError {
        code: "UNAUTHORIZED".to_string(),
        message: "Invalid token".to_string(),
        details: None,
        request_id: None,
    })?;
    Ok(Claims {
        sub: tc.sub,
        email: tc.email,
        role: tc.role,
        iat: tc.iat,
        exp: tc.exp,
    })
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
        "language": user.language,
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
            "language": user.language,
        }
    })))
}

async fn logout(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let token = req
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "refresh_token is required".to_string(),
            details: None,
            request_id: None,
        })?;

    let svc = AuthService::new(state.db.clone(), state.config.clone());
    svc.revoke_refresh_token(token).await.ok();

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Logged out"
    }))))
}

async fn logout_all(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<serde_json::Value> {
    let claims = verify_token(&state, &headers)?;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    svc.revoke_all_user_tokens(&claims.sub)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "All sessions revoked"
    }))))
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

    service.revoke_refresh_token(&req.refresh_token).await.ok();

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
    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Password reset not yet implemented"
    }))))
}

async fn reset_password(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Password reset not yet implemented"
    }))))
}

async fn send_magic_link(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Magic link not yet implemented"
    }))))
}

async fn magic_link_callback(State(_state): State<AppState>) -> ApiResult<TokenResponse> {
    Err(ApiError {
        code: "NOT_IMPLEMENTED".to_string(),
        message: "Magic link not yet implemented".to_string(),
        details: None,
        request_id: None,
    })
}

async fn totp_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<serde_json::Value> {
    let claims = verify_token(&state, &headers)?;
    let svc = MfaService::new(state.db.clone(), state.config.clone());
    let (secret, qr_url) = svc
        .setup_totp(&claims.sub, "Oak MailList")
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "secret": secret,
        "qr_url": qr_url,
    }))))
}

async fn totp_verify_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let claims = verify_token(&state, &headers)?;
    let code = req.get("code").and_then(|v| v.as_str()).ok_or(ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "code is required".to_string(),
        details: None,
        request_id: None,
    })?;

    let svc = MfaService::new(state.db.clone(), state.config.clone());
    let codes = svc
        .verify_totp_setup(&claims.sub, code)
        .await
        .map_err(|e| ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "backup_codes": codes,
    }))))
}

async fn totp_verify(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<TokenResponse> {
    let user_id = req
        .get("user_id")
        .and_then(|v| v.as_str())
        .ok_or(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "user_id is required".to_string(),
            details: None,
            request_id: None,
        })?;
    let code = req.get("code").and_then(|v| v.as_str()).ok_or(ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "code is required".to_string(),
        details: None,
        request_id: None,
    })?;

    let mfa = MfaService::new(state.db.clone(), state.config.clone());
    let valid = mfa.verify_totp(user_id, code).await.map_err(|e| ApiError {
        code: "UNAUTHORIZED".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    if !valid {
        return Err(ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Invalid TOTP code".to_string(),
            details: None,
            request_id: None,
        });
    }

    let auth = AuthService::new(state.db.clone(), state.config.clone());
    let user =
        crate::models::user::Entity::find_by_id(uuid::Uuid::parse_str(user_id).map_err(|e| {
            ApiError {
                code: "VALIDATION_ERROR".to_string(),
                message: e.to_string(),
                details: None,
                request_id: None,
            }
        })?)
        .one(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "User not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let access_token = auth
        .generate_access_token(&user.id.to_string(), &user.email, "subscriber")
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let refresh_token = auth
        .create_refresh_token(&user.id.to_string(), None)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.security.jwt_expiration_seconds,
    })))
}

async fn totp_disable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let claims = verify_token(&state, &headers)?;
    let code = req.get("code").and_then(|v| v.as_str()).ok_or(ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "code is required".to_string(),
        details: None,
        request_id: None,
    })?;

    let svc = MfaService::new(state.db.clone(), state.config.clone());
    svc.disable_totp(&claims.sub, code)
        .await
        .map_err(|e| ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "TOTP disabled"
    }))))
}

async fn totp_regenerate_backup(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<serde_json::Value> {
    let claims = verify_token(&state, &headers)?;
    let svc = MfaService::new(state.db.clone(), state.config.clone());
    let codes = svc
        .regenerate_backup_codes(&claims.sub)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "backup_codes": codes,
    }))))
}

async fn totp_backup_codes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<serde_json::Value> {
    let claims = verify_token(&state, &headers)?;
    let svc = MfaService::new(state.db.clone(), state.config.clone());
    let count = svc
        .get_backup_codes_count(&claims.sub)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "backup_codes_count": count,
    }))))
}

async fn passkey_register_options(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<serde_json::Value> {
    let claims = verify_token(&state, &headers)?;
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let Some(webauthn) = state.webauthn.clone() else {
        return Err(ApiError {
            code: "NOT_CONFIGURED".to_string(),
            message: "WebAuthn is not configured".to_string(),
            details: None,
            request_id: None,
        });
    };

    let svc =
        PasskeyService::from_state(state.db.clone(), webauthn, state.passkey_challenges.clone());
    let (ccr, challenge_id) = svc
        .start_registration(user_id, &claims.email, None)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "challenge_id": challenge_id,
        "options": ccr,
    }))))
}

async fn passkey_register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PasskeyRegisterRequest>,
) -> ApiResult<serde_json::Value> {
    let _claims = verify_token(&state, &headers)?;

    let Some(webauthn) = state.webauthn.clone() else {
        return Err(ApiError {
            code: "NOT_CONFIGURED".to_string(),
            message: "WebAuthn is not configured".to_string(),
            details: None,
            request_id: None,
        });
    };

    let svc =
        PasskeyService::from_state(state.db.clone(), webauthn, state.passkey_challenges.clone());
    let cred = svc
        .finish_registration(&req.challenge_id, &req.credential)
        .await
        .map_err(|e| ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let cred_id_b64 = Base64UrlSafeData::from(cred.credential_id.clone());
    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": cred.id,
        "credential_id": cred_id_b64,
        "created_at": cred.created_at,
    }))))
}

async fn passkey_auth_options(
    State(state): State<AppState>,
    Json(req): Json<PasskeyAuthOptionsRequest>,
) -> ApiResult<serde_json::Value> {
    let Some(webauthn) = state.webauthn.clone() else {
        return Err(ApiError {
            code: "NOT_CONFIGURED".to_string(),
            message: "WebAuthn is not configured".to_string(),
            details: None,
            request_id: None,
        });
    };

    let svc =
        PasskeyService::from_state(state.db.clone(), webauthn, state.passkey_challenges.clone());
    let (rcr, challenge_id) = svc
        .start_authentication(req.email.as_deref())
        .await
        .map_err(|e| ApiError {
            code: "NOT_FOUND".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "challenge_id": challenge_id,
        "options": rcr,
    }))))
}

async fn passkey_login(
    State(state): State<AppState>,
    Json(req): Json<PasskeyLoginRequest>,
) -> ApiResult<TokenResponse> {
    let Some(webauthn) = state.webauthn.clone() else {
        return Err(ApiError {
            code: "NOT_CONFIGURED".to_string(),
            message: "WebAuthn is not configured".to_string(),
            details: None,
            request_id: None,
        });
    };

    let svc =
        PasskeyService::from_state(state.db.clone(), webauthn, state.passkey_challenges.clone());
    let (user_id, email) = svc
        .finish_authentication(&req.challenge_id, &req.credential)
        .await
        .map_err(|e| ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let auth = AuthService::new(state.db.clone(), state.config.clone());
    let access_token = auth
        .generate_access_token(&user_id.to_string(), &email, "subscriber")
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let refresh_token = auth
        .create_refresh_token(&user_id.to_string(), None)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.security.jwt_expiration_seconds,
    })))
}

async fn mfa_verify(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<TokenResponse> {
    let user_id = req
        .get("user_id")
        .and_then(|v| v.as_str())
        .ok_or(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "user_id is required".to_string(),
            details: None,
            request_id: None,
        })?;
    let code = req.get("code").and_then(|v| v.as_str()).ok_or(ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "code is required".to_string(),
        details: None,
        request_id: None,
    })?;

    let mfa = MfaService::new(state.db.clone(), state.config.clone());
    let valid = mfa.verify_totp(user_id, code).await.map_err(|e| ApiError {
        code: "UNAUTHORIZED".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    if !valid {
        return Err(ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Invalid MFA code".to_string(),
            details: None,
            request_id: None,
        });
    }

    let auth = AuthService::new(state.db.clone(), state.config.clone());
    let user =
        crate::models::user::Entity::find_by_id(uuid::Uuid::parse_str(user_id).map_err(|e| {
            ApiError {
                code: "VALIDATION_ERROR".to_string(),
                message: e.to_string(),
                details: None,
                request_id: None,
            }
        })?)
        .one(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "User not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let access_token = auth
        .generate_access_token(&user.id.to_string(), &user.email, "subscriber")
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let refresh_token = auth
        .create_refresh_token(&user.id.to_string(), None)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.security.jwt_expiration_seconds,
    })))
}
