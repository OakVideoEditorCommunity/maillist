use crate::api::middleware::auth::Claims;
use crate::models::AppState;
use crate::services::auth_service::AuthService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{delete, get, put},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .route("/me/password", put(change_password))
        .route("/me/sessions", get(list_sessions))
        .route("/me/sessions/{id}", delete(revoke_session))
        .route("/me/mfa", get(get_mfa_status))
        .route("/me/passkeys", get(list_passkeys))
        .route(
            "/me/passkeys/{id}",
            delete(delete_passkey).put(rename_passkey),
        )
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/", get(list_users))
}

async fn get_me(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> ApiResult<serde_json::Value> {
    let user = crate::models::user::Entity::find_by_id(
        uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?,
    )
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

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "avatar_url": user.avatar_url,
        "timezone": user.timezone,
        "language": user.language,
        "is_site_admin": user.is_site_admin,
        "mfa_enabled": user.mfa_enabled,
        "last_login_at": user.last_login_at,
        "created_at": user.created_at,
    }))))
}

async fn update_me(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let user = crate::models::user::Entity::find_by_id(user_id)
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

    let mut active: crate::models::user::ActiveModel = user.into();

    if let Some(name) = req.get("name").and_then(|v| v.as_str()) {
        active.name = Set(Some(name.to_string()));
    }
    if let Some(timezone) = req.get("timezone").and_then(|v| v.as_str()) {
        active.timezone = Set(timezone.to_string());
    }
    if let Some(language) = req.get("language").and_then(|v| v.as_str()) {
        active.language = Set(language.to_string());
    }
    if let Some(avatar_url) = req.get("avatar_url").and_then(|v| v.as_str()) {
        active.avatar_url = Set(Some(avatar_url.to_string()));
    }

    let updated = active.update(&state.db).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": updated.id,
        "email": updated.email,
        "name": updated.name,
        "avatar_url": updated.avatar_url,
        "timezone": updated.timezone,
        "language": updated.language,
    }))))
}

async fn change_password(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    use crate::utils::crypto::verify_password;

    let old_password = req
        .get("old_password")
        .and_then(|v| v.as_str())
        .ok_or(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "old_password is required".to_string(),
            details: None,
            request_id: None,
        })?;

    let new_password = req
        .get("new_password")
        .and_then(|v| v.as_str())
        .ok_or(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "new_password is required".to_string(),
            details: None,
            request_id: None,
        })?;

    if new_password.len() < state.config.security.password_min_length {
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

    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let user = crate::models::user::Entity::find_by_id(user_id)
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

    let hash = user.password_hash.clone().ok_or(ApiError {
        code: "FORBIDDEN".to_string(),
        message: "Password login not available".to_string(),
        details: None,
        request_id: None,
    })?;

    if !verify_password(old_password, &hash).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })? {
        return Err(ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Invalid old password".to_string(),
            details: None,
            request_id: None,
        });
    }

    let new_hash = crate::utils::crypto::hash_password(new_password).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let mut active: crate::models::user::ActiveModel = user.into();
    active.password_hash = Set(Some(new_hash));
    active.update(&state.db).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Password updated successfully"
    }))))
}

async fn list_sessions(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> ApiResult<Vec<serde_json::Value>> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let sessions = crate::models::auth_session::Entity::find()
        .filter(crate::models::auth_session::Column::UserId.eq(user_id))
        .all(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let result: Vec<_> = sessions
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "ip_address": s.ip_address,
                "user_agent": s.user_agent,
                "expires_at": s.expires_at,
                "created_at": s.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}

async fn revoke_session(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let session_id = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let session = crate::models::auth_session::Entity::find_by_id(session_id)
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
            message: "Session not found".to_string(),
            details: None,
            request_id: None,
        })?;

    if session.user_id != user_id {
        return Err(ApiError {
            code: "FORBIDDEN".to_string(),
            message: "Not your session".to_string(),
            details: None,
            request_id: None,
        });
    }

    crate::models::auth_session::Entity::delete_by_id(session_id)
        .exec(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Session revoked"
    }))))
}

async fn get_mfa_status(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> ApiResult<serde_json::Value> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let user = crate::models::user::Entity::find_by_id(user_id)
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

    let totp_count = crate::models::totp_credential::Entity::find()
        .filter(crate::models::totp_credential::Column::UserId.eq(user_id))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len();

    Ok(Json(ApiResponse::new(serde_json::json!({
        "mfa_enabled": user.mfa_enabled,
        "totp_enabled": totp_count > 0,
    }))))
}

async fn list_passkeys(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> ApiResult<Vec<serde_json::Value>> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let items = crate::models::passkey_credential::Entity::find()
        .filter(crate::models::passkey_credential::Column::UserId.eq(user_id))
        .all(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let result: Vec<_> = items
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "credential_id": p.credential_id,
                "device_name": p.device_name,
                "created_at": p.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}

async fn delete_passkey(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let pk_id = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let pk = crate::models::passkey_credential::Entity::find_by_id(pk_id)
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
            message: "Passkey not found".to_string(),
            details: None,
            request_id: None,
        })?;

    if pk.user_id != user_id {
        return Err(ApiError {
            code: "FORBIDDEN".to_string(),
            message: "Not your passkey".to_string(),
            details: None,
            request_id: None,
        });
    }

    crate::models::passkey_credential::Entity::delete_by_id(pk_id)
        .exec(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Passkey deleted"
    }))))
}

async fn rename_passkey(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let pk_id = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let pk = crate::models::passkey_credential::Entity::find_by_id(pk_id)
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
            message: "Passkey not found".to_string(),
            details: None,
            request_id: None,
        })?;

    if pk.user_id != user_id {
        return Err(ApiError {
            code: "FORBIDDEN".to_string(),
            message: "Not your passkey".to_string(),
            details: None,
            request_id: None,
        });
    }

    let name = req.get("name").and_then(|v| v.as_str()).ok_or(ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "name is required".to_string(),
        details: None,
        request_id: None,
    })?;

    let mut active: crate::models::passkey_credential::ActiveModel = pk.into();
    active.device_name = Set(Some(name.to_string()));
    active.update(&state.db).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Passkey renamed"
    }))))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let user_id = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let user = crate::models::user::Entity::find_by_id(user_id)
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

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "is_site_admin": user.is_site_admin,
        "created_at": user.created_at,
    }))))
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let user_id = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let user = crate::models::user::Entity::find_by_id(user_id)
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

    let mut active: crate::models::user::ActiveModel = user.into();
    if let Some(v) = req.get("name").and_then(|v| v.as_str()) {
        active.name = Set(Some(v.to_string()));
    }
    if let Some(v) = req.get("is_site_admin").and_then(|v| v.as_bool()) {
        active.is_site_admin = Set(v);
    }
    active.updated_at = Set(chrono::Utc::now().into());

    let updated = active.update(&state.db).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": updated.id,
        "name": updated.name,
    }))))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let user_id = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    crate::models::user::Entity::delete_by_id(user_id)
        .exec(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "User deleted"
    }))))
}

async fn list_users(State(state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    let users = crate::models::user::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let result: Vec<_> = users
        .into_iter()
        .map(|u| {
            serde_json::json!({
                "id": u.id,
                "email": u.email,
                "name": u.name,
                "is_site_admin": u.is_site_admin,
                "created_at": u.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}
