use crate::models::AppState;
use crate::services::domain_service::DomainService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_domains).post(create_domain))
        .route(
            "/{id}",
            get(get_domain).put(update_domain).delete(delete_domain),
        )
        .route("/{id}/verify-dkim", post(verify_dkim))
}

async fn list_domains(State(state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    let svc = DomainService::new(state.db.clone());
    let items = svc.list().await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let result: Vec<_> = items
        .into_iter()
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "name": d.name,
                "created_at": d.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}

async fn create_domain(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let name = req.get("name").and_then(|v| v.as_str()).ok_or(ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "name is required".to_string(),
        details: None,
        request_id: None,
    })?;

    let svc = DomainService::new(state.db.clone());
    let domain = svc.create(name).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": domain.id,
        "name": domain.name,
    }))))
}

async fn get_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    let domain = svc
        .find_by_id(&id)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "Domain not found".to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": domain.id,
        "name": domain.name,
        "smtp_host": domain.smtp_host,
        "smtp_port": domain.smtp_port,
        "dkim_selector": domain.dkim_selector,
        "created_at": domain.created_at,
    }))))
}

async fn update_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    let domain = svc.update(&id, req).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": domain.id,
        "name": domain.name,
    }))))
}

async fn delete_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    svc.delete(&id).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Domain deleted"
    }))))
}

async fn verify_dkim(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "DKIM verification not yet implemented"
    }))))
}
