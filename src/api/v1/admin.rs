use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
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
    let ai = &state.config.ai_moderation;
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
            "enabled": ai.enabled,
            "provider": ai.provider,
            "access_key_id": ai.access_key_id,
            "access_key_secret": ai.access_key_secret,
            "region": ai.region,
            "endpoint": ai.endpoint,
            "high_risk_threshold": ai.high_risk_threshold,
            "medium_risk_threshold": ai.medium_risk_threshold,
        },
    }))))
}

async fn update_settings(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let config_path = crate::config::AppConfig::default_config_path();

    let content = std::fs::read_to_string(&config_path).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: format!("Failed to read config: {}", e),
        details: None,
        request_id: None,
    })?;

    let mut doc: toml::Value = content.parse().map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: format!("Failed to parse config: {}", e),
        details: None,
        request_id: None,
    })?;

    // Update ai_moderation section
    if let Some(ai) = req.get("ai_moderation") {
        if let Some(table) = doc.get_mut("ai_moderation").and_then(|v| v.as_table_mut()) {
            if let Some(v) = ai.get("enabled").and_then(|v| v.as_bool()) {
                table.insert("enabled".to_string(), toml::Value::Boolean(v));
            }
            for key in [
                "provider",
                "access_key_id",
                "access_key_secret",
                "region",
                "service",
                "endpoint",
            ] {
                if let Some(v) = ai.get(key).and_then(|v| v.as_str()) {
                    table.insert(key.to_string(), toml::Value::String(v.to_string()));
                }
            }
            if let Some(v) = ai.get("high_risk_threshold").and_then(|v| v.as_i64()) {
                table.insert(
                    "high_risk_threshold".to_string(),
                    toml::Value::Integer(v),
                );
            }
            if let Some(v) = ai.get("medium_risk_threshold").and_then(|v| v.as_i64()) {
                table.insert(
                    "medium_risk_threshold".to_string(),
                    toml::Value::Integer(v),
                );
            }
        }
    }

    // Update smtp.outgoing section
    if let Some(smtp) = req.get("smtp") {
        if let Some(table) = doc.get_mut("smtp").and_then(|v| v.as_table_mut()) {
            if let Some(out) = smtp.get("outgoing") {
                if let Some(out_table) = table.get_mut("outgoing").and_then(|v| v.as_table_mut()) {
                    for key in ["host", "username", "password", "from_address"] {
                        if let Some(v) = out.get(key).and_then(|v| v.as_str()) {
                            out_table.insert(key.to_string(), toml::Value::String(v.to_string()));
                        }
                    }
                    if let Some(v) = out.get("port").and_then(|v| v.as_i64()) {
                        out_table.insert("port".to_string(), toml::Value::Integer(v));
                    }
                }
            }
        }
    }

    // Update branding section
    if let Some(branding) = req.get("branding") {
        if let Some(table) = doc.get_mut("branding").and_then(|v| v.as_table_mut()) {
            for key in ["site_name", "primary_color", "logo_url"] {
                if let Some(v) = branding.get(key).and_then(|v| v.as_str()) {
                    table.insert(key.to_string(), toml::Value::String(v.to_string()));
                }
            }
        }
    }

    let updated = doc.to_string();
    std::fs::write(&config_path, updated).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: format!("Failed to write config: {}", e),
        details: None,
        request_id: None,
    })?;

    // Trigger reload
    state
        .should_reload
        .store(true, std::sync::atomic::Ordering::SeqCst);
    state.shutdown.notify_waiters();

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Settings saved. Server is reloading..."
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
