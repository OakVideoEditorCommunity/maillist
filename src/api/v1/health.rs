use axum::{extract::State, response::Json};
use serde_json::json;
use crate::models::AppState;

pub async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db_ok = check_db(&state.db).await;
    let status = if db_ok { "healthy" } else { "degraded" };
    Json(json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "database": db_ok,
        }
    }))
}

pub async fn readiness_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db_ok = check_db(&state.db).await;
    let status = if db_ok { "ready" } else { "not_ready" };
    Json(json!({ "status": status }))
}

pub async fn liveness_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "alive" }))
}

pub async fn metrics_handler() -> String {
    "# metrics endpoint\n".to_string()
}

async fn check_db(db: &sea_orm::DatabaseConnection) -> bool {
    use sea_orm::ConnectionTrait;
    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT 1".to_string(),
    ))
    .await
    .is_ok()
}
