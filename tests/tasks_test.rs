use migration::MigratorTrait;
use oak_maillist::config::AppConfig;
use oak_maillist::models::AppState;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, EntityTrait, Set};

async fn setup_db() -> AppState {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    migration::Migrator::up(&db, None).await.unwrap();
    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF".to_string(),
    ))
    .await
    .unwrap();
    let config = AppConfig::load().unwrap_or_else(|_| {
        serde_json::from_str(r#"
        {
            "server": {"host":"127.0.0.1","port":3000,"base_url":"http://localhost:3000"},
            "database": {"url":"sqlite::memory:","max_connections":5,"min_connections":1,"connect_timeout":5,"idle_timeout":300},
            "security": {"jwt_secret":"test-secret","jwt_expiration_seconds":900,"refresh_token_expiration_days":7,"session_token_expiration_seconds":600,"password_min_length":8},
            "smtp": {"incoming":{"enabled":false,"host":"0.0.0.0","port":2525},"outgoing":{"host":"","port":587,"username":"","password":"","from_address":"test@example.com"}},
            "ai_moderation": {"enabled":false,"provider":"aliyun","access_key_id":"","access_key_secret":"","region":"cn-shanghai","service":"ugc_moderation_byllm","endpoint":"","high_risk_threshold":80,"medium_risk_threshold":50,"request_timeout_seconds":30,"max_text_length":2000},
            "archive": {"enabled":true,"storage_path":"./storage/archives","max_attachment_size_mb":10},
            "logging": {"level":"error","format":"pretty"}
        }
        "#).unwrap()
    });
    AppState::new(
        db,
        config,
        std::sync::Arc::new(tokio::sync::Notify::new()),
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    )
}

#[tokio::test]
async fn test_deliver_task_creates_email_message() {
    let state = setup_db().await;
    let list_svc = oak_maillist::services::list_service::ListService::new(state.db.clone());

    let domain = oak_maillist::models::domain::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        name: Set("example.com".to_string()),
        smtp_host: Set(None),
        smtp_port: Set(None),
        smtp_username: Set(None),
        smtp_password: Set(None),
        dkim_selector: Set(None),
        dkim_private_key: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let domain_model = domain.insert(&state.db).await.unwrap();
    let list = list_svc
        .create(
            &domain_model.id.to_string(),
            "deliver-test",
            "del",
            None,
            None,
        )
        .await
        .unwrap();

    let item = oak_maillist::models::moderation_queue::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        list_id: Set(list.id),
        message_id: Set(None),
        from_addr: Set("sender@example.com".to_string()),
        subject: Set(Some("Hello".to_string())),
        reason: Set("test".to_string()),
        status: Set("approved".to_string()),
        source: Set("smtp".to_string()),
        ai_risk_score: Set(None),
        ai_labels: Set(None),
        ai_raw_response: Set(None),
        ai_reviewed: Set(false),
        moderated_by: Set(None),
        moderated_at: Set(None),
        moderation_note: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    };
    let model = item.insert(&state.db).await.unwrap();

    let task = oak_maillist::tasks::deliver::DeliverTask::new(state.db.clone());
    task.run().await.unwrap();

    let updated = oak_maillist::models::moderation_queue::Entity::find_by_id(model.id)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();
    assert!(updated.message_id.is_some());

    let msg_count = oak_maillist::models::email_message::Entity::find()
        .all(&state.db)
        .await
        .unwrap()
        .len();
    assert_eq!(msg_count, 1);
}

#[tokio::test]
async fn test_ai_moderate_task_marks_reviewed() {
    let state = setup_db().await;
    let list_svc = oak_maillist::services::list_service::ListService::new(state.db.clone());

    let domain = oak_maillist::models::domain::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        name: Set("example.com".to_string()),
        smtp_host: Set(None),
        smtp_port: Set(None),
        smtp_username: Set(None),
        smtp_password: Set(None),
        dkim_selector: Set(None),
        dkim_private_key: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let domain_model = domain.insert(&state.db).await.unwrap();
    let list = list_svc
        .create(&domain_model.id.to_string(), "ai-test", "ai", None, None)
        .await
        .unwrap();

    let item = oak_maillist::models::moderation_queue::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        list_id: Set(list.id),
        message_id: Set(None),
        from_addr: Set("sender@example.com".to_string()),
        subject: Set(Some("Spam".to_string())),
        reason: Set("ai_moderation".to_string()),
        status: Set("pending".to_string()),
        source: Set("ai_flagged".to_string()),
        ai_risk_score: Set(Some(80)),
        ai_labels: Set(None),
        ai_raw_response: Set(None),
        ai_reviewed: Set(false),
        moderated_by: Set(None),
        moderated_at: Set(None),
        moderation_note: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    };
    let model = item.insert(&state.db).await.unwrap();

    let task = oak_maillist::tasks::ai_moderate::AiModerateTask::new(state.db.clone());
    task.run().await.unwrap();

    let updated = oak_maillist::models::moderation_queue::Entity::find_by_id(model.id)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();
    assert!(updated.ai_reviewed);
}

#[tokio::test]
async fn test_cleanup_task_deletes_old_records() {
    let state = setup_db().await;

    let old_time = chrono::Utc::now() - chrono::Duration::days(60);

    let session = oak_maillist::models::auth_session::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        user_id: Set(oak_maillist::utils::crypto::generate_uuid()),
        session_token: Set("old-token".to_string()),
        step: Set("complete".to_string()),
        mfa_type: Set(None),
        ip_address: Set(None),
        user_agent: Set(None),
        expires_at: Set(old_time.into()),
        created_at: Set(old_time.into()),
    };
    session.insert(&state.db).await.unwrap();

    let task = oak_maillist::tasks::cleanup::CleanupTask::new(state.db.clone());
    task.run().await.unwrap();

    let remaining = oak_maillist::models::auth_session::Entity::find()
        .all(&state.db)
        .await
        .unwrap()
        .len();
    assert_eq!(remaining, 0);
}
