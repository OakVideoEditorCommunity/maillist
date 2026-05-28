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
use std::path::Path;

#[derive(Debug, Serialize)]
struct SetupStatusResponse {
    needs_setup: bool,
    has_admin: bool,
    user_count: i64,
}

#[derive(Debug, Deserialize)]
struct TestDbRequest {
    db_type: String,
    db_host: Option<String>,
    db_port: Option<u16>,
    db_name: Option<String>,
    db_user: Option<String>,
    db_password: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SetupRequest {
    db_type: String,
    db_host: Option<String>,
    db_port: Option<u16>,
    db_name: Option<String>,
    db_user: Option<String>,
    db_password: Option<String>,
    base_url: String,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_user: Option<String>,
    smtp_password: Option<String>,
    smtp_from: Option<String>,
    site_name: Option<String>,
    primary_color: Option<String>,
    logo_url: Option<String>,
    ai_enabled: Option<bool>,
    ai_provider: Option<String>,
    ai_access_key_id: Option<String>,
    ai_access_key_secret: Option<String>,
    ai_region: Option<String>,
    ai_endpoint: Option<String>,
    email: String,
    password: String,
    name: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(setup_status))
        .route("/test-db", post(test_db_connection))
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

    let config_exists = crate::config::AppConfig::default_config_path().exists();

    Ok(Json(ApiResponse::new(SetupStatusResponse {
        needs_setup: !config_exists || count == 0 || !has_admin,
        has_admin,
        user_count: count as i64,
    })))
}

async fn test_db_connection(Json(payload): Json<TestDbRequest>) -> ApiResult<serde_json::Value> {
    let db_url = build_db_url(
        &payload.db_type,
        payload.db_host.as_deref(),
        payload.db_port,
        payload.db_name.as_deref(),
        payload.db_user.as_deref(),
        payload.db_password.as_deref(),
    );

    match sea_orm::Database::connect(&db_url).await {
        Ok(db) => {
            let _ = db.close().await;
            Ok(Json(ApiResponse::new(serde_json::json!({
                "connected": true,
                "url": db_url,
            }))))
        }
        Err(e) => Err(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: format!("Database connection failed: {}", e),
            details: None,
            request_id: None,
        }),
    }
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

    let db_url = build_db_url(
        &payload.db_type,
        payload.db_host.as_deref(),
        payload.db_port,
        payload.db_name.as_deref(),
        payload.db_user.as_deref(),
        payload.db_password.as_deref(),
    );

    // Test DB connection before writing config
    if let Err(e) = sea_orm::Database::connect(&db_url).await {
        return Err(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: format!("Database connection failed: {}", e),
            details: None,
            request_id: None,
        });
    }

    // Generate and write config file
    let jwt_secret = generate_jwt_secret();
    let config_toml = build_config_toml(&db_url, &payload.base_url, &jwt_secret, &payload);

    let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());
    let config_path = Path::new(&config_dir).join("default.toml");

    if let Err(e) = tokio::fs::write(&config_path, config_toml).await {
        return Err(ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: format!("Failed to write config file: {}", e),
            details: None,
            request_id: None,
        });
    }

    // Create admin user using the current DB (which may differ from config DB if env var was used)
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

    // Signal the server to reload configuration
    state
        .should_reload
        .store(true, std::sync::atomic::Ordering::SeqCst);
    state.shutdown.notify_waiters();

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "is_site_admin": user.is_site_admin,
        "config_path": config_path.to_string_lossy(),
        "message": "Setup completed. Server is reloading configuration...",
    }))))
}

fn build_db_url(
    db_type: &str,
    host: Option<&str>,
    port: Option<u16>,
    name: Option<&str>,
    user: Option<&str>,
    password: Option<&str>,
) -> String {
    match db_type.to_lowercase().as_str() {
        "sqlite" => name
            .map(|n| format!("sqlite:///{}?mode=rwc", n))
            .unwrap_or_else(|| "sqlite::memory:".to_string()),
        "postgres" | "postgresql" => {
            let host = host.unwrap_or("localhost");
            let port = port.unwrap_or(5432);
            let name = name.unwrap_or("oak_maillist");
            let user = user.unwrap_or("oak");
            match password {
                Some(p) if !p.is_empty() => {
                    format!("postgres://{}:{}@{}:{}/{}", user, p, host, port, name)
                }
                _ => format!("postgres://{}@{}:{}/{}", user, host, port, name),
            }
        }
        "mysql" => {
            let host = host.unwrap_or("localhost");
            let port = port.unwrap_or(3306);
            let name = name.unwrap_or("oak_maillist");
            let user = user.unwrap_or("oak");
            match password {
                Some(p) if !p.is_empty() => {
                    format!("mysql://{}:{}@{}:{}/{}", user, p, host, port, name)
                }
                _ => format!("mysql://{}@{}:{}/{}", user, host, port, name),
            }
        }
        _ => db_type.to_string(),
    }
}

fn generate_jwt_secret() -> String {
    use rand::Rng;
    use rand::distr::Alphanumeric;
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn build_config_toml(
    db_url: &str,
    base_url: &str,
    jwt_secret: &str,
    payload: &SetupRequest,
) -> String {
    let smtp_host = payload.smtp_host.as_deref().unwrap_or("");
    let smtp_port = payload.smtp_port.unwrap_or(587);
    let smtp_user = payload.smtp_user.as_deref().unwrap_or("");
    let smtp_password = payload.smtp_password.as_deref().unwrap_or("");
    let smtp_from = payload
        .smtp_from
        .as_deref()
        .unwrap_or("noreply@example.com");
    let site_name = payload.site_name.as_deref().unwrap_or("Oak MailList");
    let primary_color = payload.primary_color.as_deref().unwrap_or("#409EFF");
    let logo_url = payload.logo_url.as_deref().unwrap_or("");
    let ai_enabled = payload.ai_enabled.unwrap_or(false);
    let ai_provider = payload.ai_provider.as_deref().unwrap_or("aliyun");
    let ai_access_key_id = payload.ai_access_key_id.as_deref().unwrap_or("");
    let ai_access_key_secret = payload.ai_access_key_secret.as_deref().unwrap_or("");
    let ai_region = payload.ai_region.as_deref().unwrap_or("cn-shanghai");
    let ai_endpoint = payload.ai_endpoint.as_deref().unwrap_or("https://green-cip.cn-shanghai.aliyuncs.com");

    format!(
        r#"[server]
host = "0.0.0.0"
port = 3000
base_url = "{base_url}"

[database]
url = "{db_url}"
max_connections = 10
min_connections = 2
connect_timeout = 10
idle_timeout = 300

[security]
jwt_secret = "{jwt_secret}"
jwt_expiration_seconds = 900
refresh_token_expiration_days = 7
session_token_expiration_seconds = 600
password_min_length = 8

[smtp.incoming]
enabled = true
host = "0.0.0.0"
port = 2525

[smtp.outgoing]
host = "{smtp_host}"
port = {smtp_port}
username = "{smtp_user}"
password = "{smtp_password}"
from_address = "{smtp_from}"

[ai_moderation]
enabled = {ai_enabled}
provider = "{ai_provider}"
access_key_id = "{ai_access_key_id}"
access_key_secret = "{ai_access_key_secret}"
region = "{ai_region}"
service = "ugc_moderation_byllm"
endpoint = "{ai_endpoint}"
high_risk_threshold = 80
medium_risk_threshold = 50
request_timeout_seconds = 30
max_text_length = 2000

[archive]
enabled = true
storage_path = "./storage/archives"
max_attachment_size_mb = 10

[logging]
level = "info"
format = "pretty"

[branding]
site_name = "{site_name}"
primary_color = "{primary_color}"
logo_url = "{logo_url}"
"#
    )
}
