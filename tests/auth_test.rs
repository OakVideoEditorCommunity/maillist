use migration::MigratorTrait;
use oak_maillist::config::AppConfig;
use oak_maillist::models::AppState;
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
async fn test_auth_register_and_login() {
    let state = setup_db().await;
    let svc = oak_maillist::services::auth_service::AuthService::new(
        state.db.clone(),
        state.config.clone(),
    );

    let user = svc
        .register("test@example.com", "password123", Some("Test User"))
        .await
        .unwrap();
    assert_eq!(user.email, "test@example.com");

    let logged_in = svc.login("test@example.com", "password123").await.unwrap();
    assert_eq!(logged_in.email, "test@example.com");

    let token = svc
        .generate_access_token(&user.id.to_string(), &user.email, "subscriber")
        .unwrap();
    assert!(!token.is_empty());

    let refresh = svc
        .create_refresh_token(&user.id.to_string(), None)
        .await
        .unwrap();
    assert!(!refresh.is_empty());

    let verified = svc.verify_refresh_token(&refresh).await.unwrap();
    assert_eq!(verified.id, user.id);

    svc.revoke_refresh_token(&refresh).await.unwrap();
}

#[tokio::test]
async fn test_auth_login_wrong_password() {
    let state = setup_db().await;
    let svc = oak_maillist::services::auth_service::AuthService::new(
        state.db.clone(),
        state.config.clone(),
    );

    svc.register("test2@example.com", "password123", None)
        .await
        .unwrap();
    let result = svc.login("test2@example.com", "wrongpassword").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mfa_totp_lifecycle() {
    let state = setup_db().await;
    let auth_svc = oak_maillist::services::auth_service::AuthService::new(
        state.db.clone(),
        state.config.clone(),
    );
    let mfa_svc = oak_maillist::services::mfa_service::MfaService::new(
        state.db.clone(),
        state.config.clone(),
    );

    let user = auth_svc
        .register("totp@example.com", "password123", None)
        .await
        .unwrap();

    let (secret, _qr) = mfa_svc
        .setup_totp(&user.id.to_string(), "Test")
        .await
        .unwrap();
    assert!(!secret.is_empty());

    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        base32_decode(&secret).unwrap(),
        Some("Test".to_string()),
        "totp@example.com".to_string(),
    )
    .unwrap();
    let code = totp.generate_current().unwrap();

    let backups = mfa_svc
        .verify_totp_setup(&user.id.to_string(), &code)
        .await
        .unwrap();
    assert_eq!(backups.len(), 10);

    let new_code = totp.generate_current().unwrap();
    let valid2 = mfa_svc
        .verify_totp(&user.id.to_string(), &new_code)
        .await
        .unwrap();
    assert!(valid2);

    let count = mfa_svc
        .get_backup_codes_count(&user.id.to_string())
        .await
        .unwrap();
    assert_eq!(count, 10);

    mfa_svc
        .disable_totp(&user.id.to_string(), &new_code)
        .await
        .unwrap();
}

fn base32_decode(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = Vec::new();
    let mut bits = 0u32;
    let mut bit_count = 0;
    for ch in input.to_uppercase().chars() {
        let val = ALPHABET.iter().position(|&b| b as char == ch)? as u32;
        bits = (bits << 5) | val;
        bit_count += 5;
        if bit_count >= 8 {
            result.push(((bits >> (bit_count - 8)) & 0xFF) as u8);
            bit_count -= 8;
        }
    }
    Some(result)
}
