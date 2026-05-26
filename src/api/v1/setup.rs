use crate::models::AppState;
use crate::services::auth_service::AuthService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct SetupStatusResponse {
    needs_setup: bool,
    has_admin: bool,
    user_count: i64,
}

#[derive(Debug, Deserialize)]
struct SetupRequest {
    email: String,
    password: String,
    name: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(setup_status))
        .route("/setup", post(perform_setup))
}

async fn setup_status(State(state): State<AppState>) -> ApiResult<SetupStatusResponse> {
    let count = crate::models::user::Entity::find()
        .count(&state.db)
        .await
        .unwrap_or(0);

    let has_admin = crate::models::user::Entity::find()
        .filter(crate::models::user::Column::IsSiteAdmin.eq(true))
        .one(&state.db)
        .await
        .unwrap_or(None)
        .is_some();

    Ok(Json(ApiResponse::new(SetupStatusResponse {
        needs_setup: count == 0 || !has_admin,
        has_admin,
        user_count: count as i64,
    })))
}

async fn perform_setup(
    State(state): State<AppState>,
    Json(payload): Json<SetupRequest>,
) -> ApiResult<serde_json::Value> {
    let count = crate::models::user::Entity::find()
        .count(&state.db)
        .await
        .unwrap_or(0);

    if count > 0 {
        return Err(ApiError {
            code: "INVALID_REQUEST".to_string(),
            message: "Setup already completed".to_string(),
            details: None,
            request_id: None,
        });
    }

    let auth_svc = AuthService::new(state.db.clone(), state.config.clone());
    let user = auth_svc
        .register_admin(&payload.email, &payload.password, payload.name.as_deref())
        .await
        .map_err(|e| ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "is_site_admin": user.is_site_admin,
        "message": "Administrator account created successfully. Please log in."
    }))))
}
