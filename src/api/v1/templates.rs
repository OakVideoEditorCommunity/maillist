use crate::models::{AppState, email_template};
use crate::services::template_service::TemplateService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tera::Context;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_templates))
        .route("/{name}", get(get_template).put(update_template))
        .route("/{name}/preview", post(preview_template))
}

async fn list_templates(State(state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    let items = email_template::Entity::find()
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
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "name": t.name,
                "subject": t.subject,
                "is_system": t.is_system,
                "created_at": t.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}

async fn get_template(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<serde_json::Value> {
    let tmpl = email_template::Entity::find()
        .filter(email_template::Column::Name.eq(&name))
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
            message: "Template not found".to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": tmpl.id,
        "name": tmpl.name,
        "subject": tmpl.subject,
        "body_text": tmpl.body_text,
        "body_html": tmpl.body_html,
        "variables": tmpl.variables,
        "is_system": tmpl.is_system,
    }))))
}

async fn update_template(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let tmpl = email_template::Entity::find()
        .filter(email_template::Column::Name.eq(&name))
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
            message: "Template not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let mut active: email_template::ActiveModel = tmpl.into();
    if let Some(v) = req.get("subject").and_then(|v| v.as_str()) {
        active.subject = Set(Some(v.to_string()));
    }
    if let Some(v) = req.get("body_text").and_then(|v| v.as_str()) {
        active.body_text = Set(Some(v.to_string()));
    }
    if let Some(v) = req.get("body_html").and_then(|v| v.as_str()) {
        active.body_html = Set(Some(v.to_string()));
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

async fn preview_template(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let svc = TemplateService::new(state.db.clone());
    let mut ctx = Context::new();

    if let Some(obj) = req.as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                ctx.insert(k, s);
            }
        }
    }

    let (subject, body) = svc
        .render_template(&name, &ctx)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "subject": subject,
        "body": body,
    }))))
}
