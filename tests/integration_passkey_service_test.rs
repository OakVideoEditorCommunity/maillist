use migration::MigratorTrait;
use oak_maillist::config::AppConfig;
use oak_maillist::models::AppState;
use oak_maillist::services::passkey_service::PasskeyService;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database};

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

#[tokio::test]
async fn test_passkey_service_start_registration() {
    let state = setup_db().await;
    let svc = PasskeyService::from_state(
        state.db.clone(),
        state.webauthn.clone().unwrap(),
        state.passkey_challenges.clone(),
    );

    let user_id = uuid::Uuid::new_v4();
    let (ccr, challenge_id) = svc
        .start_registration(user_id, "test@example.com", Some("Test User"))
        .await
        .unwrap();

    assert!(!challenge_id.is_empty());
    assert_eq!(ccr.public_key.user.name, "test@example.com");
    assert_eq!(ccr.public_key.user.display_name, "Test User");

    // Verify challenge is stored
    let challenges = state.passkey_challenges.read().await;
    assert!(challenges.contains_key(&challenge_id));
}

#[tokio::test]
async fn test_passkey_service_finish_registration_invalid_challenge() {
    let state = setup_db().await;
    let svc = PasskeyService::from_state(
        state.db.clone(),
        state.webauthn.clone().unwrap(),
        state.passkey_challenges.clone(),
    );

    let credential = webauthn_rs::prelude::RegisterPublicKeyCredential {
        id: "test".to_string(),
        raw_id: webauthn_rs::prelude::Base64UrlSafeData::from(vec![1, 2, 3]),
        response: webauthn_rs_proto::AuthenticatorAttestationResponseRaw {
            attestation_object: webauthn_rs::prelude::Base64UrlSafeData::from(vec![4, 5, 6]),
            client_data_json: webauthn_rs::prelude::Base64UrlSafeData::from(vec![7, 8, 9]),
            transports: None,
        },
        extensions: webauthn_rs_proto::extensions::RegistrationExtensionsClientOutputs::default(),
        type_: "public-key".to_string(),
    };

    let result = svc.finish_registration("nonexistent", &credential).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid or expired challenge"));
}

#[tokio::test]
async fn test_passkey_service_start_authentication_user_not_found() {
    let state = setup_db().await;
    let svc = PasskeyService::from_state(
        state.db.clone(),
        state.webauthn.clone().unwrap(),
        state.passkey_challenges.clone(),
    );

    let result = svc.start_authentication(Some("nobody@example.com")).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("User not found"));
}

#[tokio::test]
async fn test_passkey_service_start_authentication_no_passkey() {
    let state = setup_db().await;
    let svc = PasskeyService::from_state(
        state.db.clone(),
        state.webauthn.clone().unwrap(),
        state.passkey_challenges.clone(),
    );

    // Create a user without passkey credentials
    let user = oak_maillist::services::auth_service::AuthService::new(
        state.db.clone(),
        state.config.clone(),
    )
    .register("user@example.com", "password123", Some("User"))
    .await
    .unwrap();

    let result = svc.start_authentication(Some(&user.email)).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("No passkey registered"));
}

#[tokio::test]
async fn test_passkey_service_finish_authentication_invalid_challenge() {
    let state = setup_db().await;
    let svc = PasskeyService::from_state(
        state.db.clone(),
        state.webauthn.clone().unwrap(),
        state.passkey_challenges.clone(),
    );

    let credential = webauthn_rs::prelude::PublicKeyCredential {
        id: "test".to_string(),
        raw_id: webauthn_rs::prelude::Base64UrlSafeData::from(vec![1, 2, 3]),
        response: webauthn_rs_proto::AuthenticatorAssertionResponseRaw {
            authenticator_data: webauthn_rs::prelude::Base64UrlSafeData::from(vec![4, 5, 6]),
            client_data_json: webauthn_rs::prelude::Base64UrlSafeData::from(vec![7, 8, 9]),
            signature: webauthn_rs::prelude::Base64UrlSafeData::from(vec![10, 11, 12]),
            user_handle: None,
        },
        extensions: webauthn_rs_proto::extensions::AuthenticationExtensionsClientOutputs::default(),
        type_: "public-key".to_string(),
    };

    let result = svc.finish_authentication("nonexistent", &credential).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid or expired challenge"));
}

#[tokio::test]
async fn test_passkey_service_start_registration_excludes_existing() {
    let state = setup_db().await;
    let svc = PasskeyService::from_state(
        state.db.clone(),
        state.webauthn.clone().unwrap(),
        state.passkey_challenges.clone(),
    );

    let user_id = uuid::Uuid::new_v4();

    // First registration attempt
    let (ccr1, challenge_id1) = svc
        .start_registration(user_id, "test@example.com", None)
        .await
        .unwrap();
    assert!(!challenge_id1.is_empty());
    assert!(ccr1.public_key.exclude_credentials.is_none());

    // Insert a fake credential so the next registration excludes it
    let fake_cred_id = vec![1u8, 2, 3, 4];
    let fake_passkey = serde_json::json!({
        "cred": {
            "cred_id": fake_cred_id.clone(),
            "cred": {"type_": "ES256", "key": {}},
            "counter": 0,
            "transports": null,
            "user_verified": true,
            "backup_eligible": false,
            "backup_state": false
        }
    });
    let _cred: oak_maillist::models::passkey_credential::Model =
        oak_maillist::models::passkey_credential::ActiveModel {
            id: sea_orm::Set(uuid::Uuid::new_v4()),
            user_id: sea_orm::Set(user_id),
            credential_id: sea_orm::Set(fake_cred_id),
            public_key: sea_orm::Set(fake_passkey.to_string().into_bytes()),
            sign_count: sea_orm::Set(0),
            aaguid: sea_orm::Set(None),
            device_name: sea_orm::Set(None),
            transports: sea_orm::Set(None),
            is_backup_eligible: sea_orm::Set(false),
            is_backup: sea_orm::Set(false),
            last_used_at: sea_orm::Set(None),
            created_at: sea_orm::Set(chrono::Utc::now().into()),
        }
        .insert(&state.db)
        .await
        .unwrap();

    // Second registration should exclude existing credential
    let (ccr2, challenge_id2) = svc
        .start_registration(user_id, "test@example.com", None)
        .await
        .unwrap();
    assert!(!challenge_id2.is_empty());
    assert!(ccr2.public_key.exclude_credentials.is_some());
    let excluded = ccr2.public_key.exclude_credentials.unwrap();
    assert_eq!(excluded.len(), 1);
}
