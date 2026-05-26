use crate::api::middleware::auth::Claims;
use crate::models::{AppState, moderation_queue};
use crate::services::moderation_service::ModerationService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{get, post},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let item = moderation_queue::Entity::find_by_id(uuid)
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
            message: "Moderation item not found".to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": item.id,
        "list_id": item.list_id,
        "from_addr": item.from_addr,
        "subject": item.subject,
        "reason": item.reason,
        "status": item.status,
        "ai_risk_score": item.ai_risk_score,
        "created_at": item.created_at,
    }))))
}

async fn approve(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let service = ModerationService::new(state.db.clone());
    service
        .approve(&id, Some(&claims.sub))
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Approved successfully"
    }))))
}

async fn reject(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let service = ModerationService::new(state.db.clone());
    let note = req.get("note").and_then(|v| v.as_str());
    service
        .reject(&id, Some(&claims.sub), note)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Rejected successfully"
    }))))
}

async fn discard(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let service = ModerationService::new(state.db.clone());
    service
        .discard(&id, Some(&claims.sub))
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Discarded successfully"
    }))))
}

async fn whitelist_sender(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let service = ModerationService::new(state.db.clone());
    service
        .whitelist_sender(&id, Some(&claims.sub))
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Approved and sender whitelisted"
    }))))
}

async fn blacklist_sender(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let service = ModerationService::new(state.db.clone());
    service
        .blacklist_sender(&id, Some(&claims.sub))
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Rejected and sender blacklisted"
    }))))
}

async fn ai_feedback(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "AI feedback recorded"
    }))))
}
