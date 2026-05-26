use chrono::Utc;
use migration::MigratorTrait;
use oak_maillist::config::AppConfig;
use oak_maillist::models::AppState;
use oak_maillist::services::notification_service::NotificationService;
use sea_orm::{ConnectionTrait, Database};

async fn setup_db() -> AppState {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    migration::Migrator::up(&db, None).await.unwrap();
    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF".to_string(),
    ))
    .await
    .unwrap();

    let config: AppConfig = serde_json::from_str(
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
    .unwrap();
    AppState::new(db, config)
}

fn subscriber_model(list_id: uuid::Uuid, email: &str) -> oak_maillist::models::subscriber::Model {
    oak_maillist::models::subscriber::Model {
        id: uuid::Uuid::new_v4(),
        list_id,
        email: email.to_string(),
        name: Some("Test User".to_string()),
        status: "active".to_string(),
        digest_mode: "none".to_string(),
        subscribe_ip: None,
        subscribe_source: None,
        bounce_count: 0,
        last_bounce_at: None,
        token: "token123".to_string(),
        confirmed_at: Some(Utc::now().into()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    }
}

fn list_model(domain_id: uuid::Uuid, name: &str) -> oak_maillist::models::mailing_list::Model {
    oak_maillist::models::mailing_list::Model {
        id: uuid::Uuid::new_v4(),
        domain_id,
        name: name.to_string(),
        display_name: Some(name.to_string()),
        email_local_part: name.to_lowercase().replace(" ", "-"),
        description: None,
        visibility: "public".to_string(),
        subscription_policy: "open".to_string(),
        post_policy: "open".to_string(),
        reply_to: "list".to_string(),
        archive_enabled: true,
        archive_visibility: "public".to_string(),
        max_message_size_kb: 1024,
        digest_enabled: false,
        header_template: None,
        footer_template: None,
        ai_moderation_enabled: false,
        is_active: true,
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    }
}

#[tokio::test]
async fn test_send_subscription_confirm() {
    let state = setup_db().await;
    let svc = NotificationService::new(state.db.clone(), state.config.smtp.outgoing.clone());

    let sub = subscriber_model(uuid::Uuid::new_v4(), "sub@example.com");
    let list = list_model(uuid::Uuid::new_v4(), "Test List");

    // SMTP host is empty, so it returns Ok without sending
    let result = svc
        .send_subscription_confirm(&sub, &list, "http://confirm.url")
        .await;
    if let Err(ref e) = result {
        eprintln!("send_subscription_confirm error: {}", e);
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_unsubscribe_confirm() {
    let state = setup_db().await;
    let svc = NotificationService::new(state.db.clone(), state.config.smtp.outgoing.clone());

    let sub = subscriber_model(uuid::Uuid::new_v4(), "sub@example.com");
    let list = list_model(uuid::Uuid::new_v4(), "Test List");

    let result = svc
        .send_unsubscribe_confirm(&sub, &list, "http://unsubscribe.url")
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_moderation_notice() {
    let state = setup_db().await;
    let svc = NotificationService::new(state.db.clone(), state.config.smtp.outgoing.clone());

    let list = list_model(uuid::Uuid::new_v4(), "Test List");

    let result = svc
        .send_moderation_notice(
            "admin@example.com",
            &list,
            "Suspicious email",
            "http://review.url",
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_welcome() {
    let state = setup_db().await;
    let svc = NotificationService::new(state.db.clone(), state.config.smtp.outgoing.clone());

    let sub = subscriber_model(uuid::Uuid::new_v4(), "sub@example.com");
    let list = list_model(uuid::Uuid::new_v4(), "Test List");

    let result = svc.send_welcome(&sub, &list).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_notification_missing_template() {
    let state = setup_db().await;

    // Delete all seeded templates
    let db = &state.db;
    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "DELETE FROM email_template".to_string(),
    ))
    .await
    .unwrap();

    let svc = NotificationService::new(state.db.clone(), state.config.smtp.outgoing.clone());

    let sub = subscriber_model(uuid::Uuid::new_v4(), "sub@example.com");
    let list = list_model(uuid::Uuid::new_v4(), "Test List");

    let result = svc
        .send_subscription_confirm(&sub, &list, "http://confirm.url")
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found"));
}
