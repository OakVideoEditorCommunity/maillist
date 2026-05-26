use chrono::Utc;
use oak_maillist::config::AppConfig;
use oak_maillist::models::AppState;
use oak_maillist::smtp::processor::MailPipeline;
use oak_maillist::smtp::server::IncomingEmail;
use migration::MigratorTrait;
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
    });
    AppState::new(db, config)
}

fn make_raw_email(to: &str, from: &str, subject: &str, body: &str) -> Vec<u8> {
    format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nMessage-Id: <test-123@example.com>\r\nContent-Type: text/plain\r\n\r\n{}",
        from, to, subject, body
    )
    .into_bytes()
}

#[tokio::test]
async fn test_mail_pipeline_process_success() {
    let state = setup_db().await;
    let db = state.db.clone();

    // Create domain
    let domain_id = uuid::Uuid::new_v4();
    let _domain = oak_maillist::models::domain::ActiveModel {
        id: Set(domain_id),
        name: Set("example.com".to_string()),
        smtp_host: Set(None),
        smtp_port: Set(None),
        smtp_username: Set(None),
        smtp_password: Set(None),
        dkim_selector: Set(None),
        dkim_private_key: Set(None),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    // Create mailing list
    let list_id = uuid::Uuid::new_v4();
    let _list = oak_maillist::models::mailing_list::ActiveModel {
        id: Set(list_id),
        domain_id: Set(domain_id),
        name: Set("Test List".to_string()),
        display_name: Set(Some("Test List Display".to_string())),
        email_local_part: Set("test-list".to_string()),
        description: Set(None),
        visibility: Set("public".to_string()),
        subscription_policy: Set("open".to_string()),
        post_policy: Set("subscriber_only".to_string()),
        reply_to: Set("list".to_string()),
        archive_enabled: Set(true),
        archive_visibility: Set("public".to_string()),
        max_message_size_kb: Set(1024),
        digest_enabled: Set(false),
        header_template: Set(None),
        footer_template: Set(None),
        ai_moderation_enabled: Set(false),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    // Create active subscriber
    let _sub = oak_maillist::models::subscriber::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        list_id: Set(list_id),
        email: Set("subscriber@example.com".to_string()),
        name: Set(Some("Sub".to_string())),
        status: Set("active".to_string()),
        digest_mode: Set("none".to_string()),
        subscribe_ip: Set(None),
        subscribe_source: Set(None),
        bounce_count: Set(0),
        last_bounce_at: Set(None),
        token: Set("token123".to_string()),
        confirmed_at: Set(Some(Utc::now().into())),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let raw = make_raw_email(
        "test-list@example.com",
        "subscriber@example.com",
        "Hello World",
        "This is a test email body.",
    );
    let raw_for_parse = raw.clone();
    let parsed = mailparse::parse_mail(&raw_for_parse).unwrap();
    let email = IncomingEmail {
        from: "subscriber@example.com".to_string(),
        to: vec!["test-list@example.com".to_string()],
        raw_data: raw,
        remote_addr: "127.0.0.1".to_string(),
    };

    let pipeline = MailPipeline::new(state);
    let result = pipeline.process(email, parsed).await;
    assert!(result.is_ok());

    // Verify email was saved
    let msgs = oak_maillist::models::email_message::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].subject, Some("Hello World".to_string()));
}

#[tokio::test]
async fn test_mail_pipeline_no_list() {
    let state = setup_db().await;
    let db = state.db.clone();
    let raw = make_raw_email(
        "nonexistent@example.com",
        "sender@example.com",
        "No List",
        "Body",
    );
    let raw_for_parse = raw.clone();
    let parsed = mailparse::parse_mail(&raw_for_parse).unwrap();
    let email = IncomingEmail {
        from: "sender@example.com".to_string(),
        to: vec!["nonexistent@example.com".to_string()],
        raw_data: raw,
        remote_addr: "127.0.0.1".to_string(),
    };

    let pipeline = MailPipeline::new(state);
    let result = pipeline.process(email, parsed).await;
    assert!(result.is_ok());

    // No message should be saved
    let msgs = oak_maillist::models::email_message::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(msgs.len(), 0);
}

#[tokio::test]
async fn test_mail_pipeline_inactive_list() {
    let state = setup_db().await;
    let db = state.db.clone();

    let domain_id = uuid::Uuid::new_v4();
    let _domain = oak_maillist::models::domain::ActiveModel {
        id: Set(domain_id),
        name: Set("example.com".to_string()),
        smtp_host: Set(None),
        smtp_port: Set(None),
        smtp_username: Set(None),
        smtp_password: Set(None),
        dkim_selector: Set(None),
        dkim_private_key: Set(None),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let list_id = uuid::Uuid::new_v4();
    let _list = oak_maillist::models::mailing_list::ActiveModel {
        id: Set(list_id),
        domain_id: Set(domain_id),
        name: Set("Inactive".to_string()),
        display_name: Set(None),
        email_local_part: Set("inactive-list".to_string()),
        description: Set(None),
        visibility: Set("public".to_string()),
        subscription_policy: Set("open".to_string()),
        post_policy: Set("open".to_string()),
        reply_to: Set("list".to_string()),
        archive_enabled: Set(true),
        archive_visibility: Set("public".to_string()),
        max_message_size_kb: Set(1024),
        digest_enabled: Set(false),
        header_template: Set(None),
        footer_template: Set(None),
        ai_moderation_enabled: Set(false),
        is_active: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let raw = make_raw_email(
        "inactive-list@example.com",
        "sender@example.com",
        "To Inactive",
        "Body",
    );
    let raw_for_parse = raw.clone();
    let parsed = mailparse::parse_mail(&raw_for_parse).unwrap();
    let email = IncomingEmail {
        from: "sender@example.com".to_string(),
        to: vec!["inactive-list@example.com".to_string()],
        raw_data: raw,
        remote_addr: "127.0.0.1".to_string(),
    };

    let pipeline = MailPipeline::new(state);
    let result = pipeline.process(email, parsed).await;
    assert!(result.is_ok());

    let msgs = oak_maillist::models::email_message::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(msgs.len(), 0);
}

#[tokio::test]
async fn test_mail_pipeline_non_subscriber_rejected() {
    let state = setup_db().await;
    let db = state.db.clone();

    let domain_id = uuid::Uuid::new_v4();
    let _domain = oak_maillist::models::domain::ActiveModel {
        id: Set(domain_id),
        name: Set("example.com".to_string()),
        smtp_host: Set(None),
        smtp_port: Set(None),
        smtp_username: Set(None),
        smtp_password: Set(None),
        dkim_selector: Set(None),
        dkim_private_key: Set(None),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let list_id = uuid::Uuid::new_v4();
    let _list = oak_maillist::models::mailing_list::ActiveModel {
        id: Set(list_id),
        domain_id: Set(domain_id),
        name: Set("Private".to_string()),
        display_name: Set(None),
        email_local_part: Set("private-list".to_string()),
        description: Set(None),
        visibility: Set("private".to_string()),
        subscription_policy: Set("confirm".to_string()),
        post_policy: Set("subscriber_only".to_string()),
        reply_to: Set("list".to_string()),
        archive_enabled: Set(true),
        archive_visibility: Set("private".to_string()),
        max_message_size_kb: Set(1024),
        digest_enabled: Set(false),
        header_template: Set(None),
        footer_template: Set(None),
        ai_moderation_enabled: Set(false),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let raw = make_raw_email(
        "private-list@example.com",
        "outsider@example.com",
        "Unauthorized",
        "Body",
    );
    let raw_for_parse = raw.clone();
    let parsed = mailparse::parse_mail(&raw_for_parse).unwrap();
    let email = IncomingEmail {
        from: "outsider@example.com".to_string(),
        to: vec!["private-list@example.com".to_string()],
        raw_data: raw,
        remote_addr: "127.0.0.1".to_string(),
    };

    let pipeline = MailPipeline::new(state);
    let result = pipeline.process(email, parsed).await;
    assert!(result.is_ok());

    let msgs = oak_maillist::models::email_message::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(msgs.len(), 0);
}

#[tokio::test]
async fn test_mail_pipeline_open_list_no_subscriber() {
    let state = setup_db().await;
    let db = state.db.clone();

    let domain_id = uuid::Uuid::new_v4();
    let _domain = oak_maillist::models::domain::ActiveModel {
        id: Set(domain_id),
        name: Set("example.com".to_string()),
        smtp_host: Set(None),
        smtp_port: Set(None),
        smtp_username: Set(None),
        smtp_password: Set(None),
        dkim_selector: Set(None),
        dkim_private_key: Set(None),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let list_id = uuid::Uuid::new_v4();
    let _list = oak_maillist::models::mailing_list::ActiveModel {
        id: Set(list_id),
        domain_id: Set(domain_id),
        name: Set("Open".to_string()),
        display_name: Set(None),
        email_local_part: Set("open-list".to_string()),
        description: Set(None),
        visibility: Set("public".to_string()),
        subscription_policy: Set("open".to_string()),
        post_policy: Set("open".to_string()),
        reply_to: Set("list".to_string()),
        archive_enabled: Set(true),
        archive_visibility: Set("public".to_string()),
        max_message_size_kb: Set(1024),
        digest_enabled: Set(false),
        header_template: Set(None),
        footer_template: Set(None),
        ai_moderation_enabled: Set(false),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let raw = make_raw_email(
        "open-list@example.com",
        "anyone@example.com",
        "Open Post",
        "Anyone can post here.",
    );
    let raw_for_parse = raw.clone();
    let parsed = mailparse::parse_mail(&raw_for_parse).unwrap();
    let email = IncomingEmail {
        from: "anyone@example.com".to_string(),
        to: vec!["open-list@example.com".to_string()],
        raw_data: raw,
        remote_addr: "127.0.0.1".to_string(),
    };

    let pipeline = MailPipeline::new(state);
    let result = pipeline.process(email, parsed).await;
    assert!(result.is_ok());

    let msgs = oak_maillist::models::email_message::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(msgs.len(), 1);
}

#[tokio::test]
async fn test_mail_pipeline_multipart_email() {
    let state = setup_db().await;
    let db = state.db.clone();

    let domain_id = uuid::Uuid::new_v4();
    let _domain = oak_maillist::models::domain::ActiveModel {
        id: Set(domain_id),
        name: Set("example.com".to_string()),
        smtp_host: Set(None),
        smtp_port: Set(None),
        smtp_username: Set(None),
        smtp_password: Set(None),
        dkim_selector: Set(None),
        dkim_private_key: Set(None),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let list_id = uuid::Uuid::new_v4();
    let _list = oak_maillist::models::mailing_list::ActiveModel {
        id: Set(list_id),
        domain_id: Set(domain_id),
        name: Set("Multi".to_string()),
        display_name: Set(None),
        email_local_part: Set("multi-list".to_string()),
        description: Set(None),
        visibility: Set("public".to_string()),
        subscription_policy: Set("open".to_string()),
        post_policy: Set("open".to_string()),
        reply_to: Set("list".to_string()),
        archive_enabled: Set(true),
        archive_visibility: Set("public".to_string()),
        max_message_size_kb: Set(1024),
        digest_enabled: Set(false),
        header_template: Set(None),
        footer_template: Set(None),
        ai_moderation_enabled: Set(false),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let raw = b"From: sender@example.com\r\nTo: multi-list@example.com\r\nSubject: Multipart\r\nContent-Type: multipart/alternative; boundary=\"abc\"\r\n\r\n--abc\r\nContent-Type: text/plain\r\n\r\nPlain text body\r\n--abc\r\nContent-Type: text/html\r\n\r\n<html><body>HTML body</body></html>\r\n--abc--".to_vec();
    let raw_for_parse = raw.clone();
    let parsed = mailparse::parse_mail(&raw_for_parse).unwrap();
    let email = IncomingEmail {
        from: "sender@example.com".to_string(),
        to: vec!["multi-list@example.com".to_string()],
        raw_data: raw,
        remote_addr: "127.0.0.1".to_string(),
    };

    let pipeline = MailPipeline::new(state);
    let result = pipeline.process(email, parsed).await;
    assert!(result.is_ok());

    let msgs = oak_maillist::models::email_message::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].body_text, Some("Plain text body\r\n".to_string()));
    assert_eq!(
        msgs[0].body_html,
        Some("<html><body>HTML body</body></html>\r\n".to_string())
    );
}
