use crate::api::middleware::auth::Claims;
use crate::models::AppState;
use crate::services::auth_service::AuthService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, put},
    Json, Router,
};
use sea_orm::EntityTrait;

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
    use sea_orm::ActiveModelTrait;
    use sea_orm::Set;

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
    use sea_orm::ActiveModelTrait;
    use sea_orm::Set;

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
    Extension(_claims): Extension<Claims>,
    State(_state): State<AppState>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn revoke_session(
    Extension(_claims): Extension<Claims>,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn get_mfa_status(
    Extension(_claims): Extension<Claims>,
    State(_state): State<AppState>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn list_passkeys(
    Extension(_claims): Extension<Claims>,
    State(_state): State<AppState>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn delete_passkey(
    Extension(_claims): Extension<Claims>,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn rename_passkey(
    Extension(_claims): Extension<Claims>,
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
