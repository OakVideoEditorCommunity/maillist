use chrono::Utc;
use lettre::Transport;
use migration::MigratorTrait;
use oak_maillist::config::AppConfig;
use oak_maillist::models::AppState;
use oak_maillist::smtp::server::SmtpServer;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, EntityTrait, Set};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::timeout;

#[allow(dead_code)]
fn make_raw_email(to: &str, from: &str, subject: &str, body: &str) -> Vec<u8> {
    format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nMessage-Id: <test-123@example.com>\r\nContent-Type: text/plain\r\n\r\n{}",
        from, to, subject, body
    )
    .into_bytes()
}

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
                "logging": {"level":"error","format":"pretty"},
            "branding": {"site_name":"Oak MailList","primary_color":"409EFF","logo_url":""}
            }
            "#,
        )
        .unwrap()
    });
    AppState::new(
        db,
        config,
        Arc::new(Notify::new()),
        Arc::new(AtomicBool::new(false)),
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_smtp_e2e_localhost_to_localhost() {
    let state = setup_db().await;
    let db = state.db.clone();

    // Seed domain
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
        dkim_public_key: Set(None),
        spf_record: Set(None),
        dmarc_record: Set(None),
        spf_verified: Set(false),
        dkim_verified: Set(false),
        dmarc_verified: Set(false),
        dkim_enabled: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    // Seed mailing list
    let list_id = uuid::Uuid::new_v4();
    let _list = oak_maillist::models::mailing_list::ActiveModel {
        id: Set(list_id),
        domain_id: Set(domain_id),
        name: Set("E2E List".to_string()),
        display_name: Set(Some("E2E List Display".to_string())),
        email_local_part: Set("e2e-list".to_string()),
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

    // Seed subscriber
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

    // Find a free port manually
    let free_port = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    };

    // Start SMTP server on the free port
    let server = SmtpServer::new("127.0.0.1".to_string(), free_port, state.clone());
    let server_task = tokio::spawn({
        let server = server.clone();
        async move {
            server.run().await.unwrap();
        }
    });

    // Wait until the server has bound the port
    let mut bound = false;
    for _ in 0..50 {
        if server.bound_port().await.is_some() {
            bound = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(bound, "SMTP server should have bound to a port");

    // Build and send email via lettre to localhost
    let email = lettre::Message::builder()
        .from("sender@example.com".parse().unwrap())
        .to("e2e-list@example.com".parse().unwrap())
        .subject("E2E Test Subject")
        .body("This is an end-to-end test body.".to_string())
        .unwrap();

    let mailer = lettre::SmtpTransport::builder_dangerous("127.0.0.1")
        .port(free_port)
        .timeout(Some(Duration::from_secs(5)))
        .build();

    let send_result = timeout(Duration::from_secs(10), async { mailer.send(&email) }).await;

    assert!(send_result.is_ok(), "SMTP send should not time out");
    let send_result = send_result.unwrap();
    assert!(
        send_result.is_ok(),
        "SMTP send should succeed: {:?}",
        send_result.err()
    );

    // Poll the DB with timeout to verify the message was saved
    let mut found = false;
    let db_check = timeout(Duration::from_secs(5), async {
        for _ in 0..50 {
            let msgs = oak_maillist::models::email_message::Entity::find()
                .all(&db)
                .await
                .unwrap();
            if !msgs.is_empty() {
                assert_eq!(msgs.len(), 1);
                assert_eq!(msgs[0].subject, Some("E2E Test Subject".to_string()));
                assert_eq!(msgs[0].from_addr, "sender@example.com");
                found = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await;

    assert!(db_check.is_ok(), "DB check should not time out");
    assert!(found, "Email should have been saved to email_message table");

    // Clean up server task
    server_task.abort();
}
