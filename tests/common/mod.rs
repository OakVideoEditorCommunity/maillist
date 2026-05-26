use axum::{Router, extract::Request, middleware::Next, response::Response};
use migration::MigratorTrait;
use oak_maillist::api::create_router;
use oak_maillist::config::AppConfig;
use oak_maillist::models::AppState;
use sea_orm::{ConnectionTrait, Database};
use std::net::SocketAddr;

pub async fn setup_db() -> AppState {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("connect to sqlite memory");
    migration::Migrator::up(&db, None)
        .await
        .expect("run migrations");
    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF".to_string(),
    ))
    .await
    .unwrap();

    let config = test_config();
    AppState::new(db, config)
}

async fn inject_connect_info(mut req: Request, next: Next) -> Response {
    req.extensions_mut()
        .insert(axum::extract::ConnectInfo(SocketAddr::from((
            [127, 0, 0, 1],
            3000,
        ))));
    next.run(req).await
}

pub async fn setup_app() -> Router {
    let state = setup_db().await;
    create_router(state).layer(axum::middleware::from_fn(inject_connect_info))
}

pub fn test_config() -> AppConfig {
    AppConfig::load().unwrap_or_else(|_| {
        serde_json::from_str(
            r#"
            {
                "server": {"host":"127.0.0.1","port":3000,"base_url":"http://localhost:3000"},
                "database": {"url":"sqlite::memory:","max_connections":5,"min_connections":1,"connect_timeout":5,"idle_timeout":300},
                "security": {"jwt_secret":"test-secret","jwt_expiration_seconds":900,"refresh_token_expiration_days":7,"session_token_expiration_seconds":600,"password_min_length":8},
                "smtp": {"incoming":{"enabled":false,"host":"0.0.0.0","port":2525},"outgoing":{"host":"","port":587,"username":"","password":"","from_address":"test@example.com"}},
                "ai_moderation": {"enabled":false,"provider":"aliyun","access_key_id":"","access_key_secret":"","region":"cn-shanghai","service":"ugc_moderation_byllm","endpoint":"","high_risk_threshold":80,"medium_risk_threshold":50,"request_timeout_seconds":30,"max_text_length":2000},
                "archive": {"enabled":true,"storage_path":"./storage/archives","max_attachment_size_mb":10},
                "logging": {"level":"error","format":"pretty"}
            }
            "#,
        )
        .unwrap()
    })
}
