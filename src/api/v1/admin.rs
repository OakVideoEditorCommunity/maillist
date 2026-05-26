use crate::models::AppState;
use crate::utils::response::{ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/stats", get(stats))
        .route("/activity-log", get(activity_log))
        .route("/settings", get(get_settings).put(update_settings))
        .route("/ai-moderation/stats", get(ai_moderation_stats))
}

async fn dashboard(State(state): State<AppState>) -> ApiResult<serde_json::Value> {
    let user_count = crate::models::user::Entity::find()
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;
    let list_count = crate::models::mailing_list::Entity::find()
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;
    let sub_count = crate::models::subscriber::Entity::find()
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;
    let msg_count = crate::models::email_message::Entity::find()
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "users": user_count,
        "lists": list_count,
        "subscribers": sub_count,
        "messages": msg_count,
    }))))
}

async fn stats(State(state): State<AppState>) -> ApiResult<serde_json::Value> {
    let pending_mod = crate::models::moderation_queue::Entity::find()
        .filter(crate::models::moderation_queue::Column::Status.eq("pending"))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "pending_moderation": pending_mod,
    }))))
}

async fn activity_log(
    State(_state): State<AppState>,
    _params: Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    Ok(Json(ApiResponse::new(vec![] as Vec<serde_json::Value>)))
}

async fn get_settings(State(state): State<AppState>) -> ApiResult<serde_json::Value> {
    Ok(Json(ApiResponse::new(serde_json::json!({
        "server": {
            "host": state.config.server.host,
            "port": state.config.server.port,
        },
        "smtp": {
            "incoming_enabled": state.config.smtp.incoming.enabled,
            "outgoing_host": state.config.smtp.outgoing.host,
        },
        "ai_moderation": {
            "enabled": state.config.ai_moderation.enabled,
            "provider": state.config.ai_moderation.provider,
        },
    }))))
}

async fn update_settings(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Settings update not yet implemented"
    }))))
}

async fn ai_moderation_stats(State(state): State<AppState>) -> ApiResult<serde_json::Value> {
    let total = crate::models::moderation_queue::Entity::find()
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;
    let pending = crate::models::moderation_queue::Entity::find()
        .filter(crate::models::moderation_queue::Column::Status.eq("pending"))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;
    let approved = crate::models::moderation_queue::Entity::find()
        .filter(crate::models::moderation_queue::Column::Status.eq("approved"))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;
    let rejected = crate::models::moderation_queue::Entity::find()
        .filter(crate::models::moderation_queue::Column::Status.eq("rejected"))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "total": total,
        "pending": pending,
        "approved": approved,
        "rejected": rejected,
    }))))
}
