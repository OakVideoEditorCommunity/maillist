mod common;
use common::setup_db;
use oak_maillist::services::auth_service::AuthService;
use oak_maillist::services::mfa_service::MfaService;

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

#[tokio::test]
async fn test_setup_totp_generates_secret_and_qr() {
    let state = setup_db().await;
    let auth_svc = AuthService::new(state.db.clone(), state.config.clone());
    let mfa_svc = MfaService::new(state.db.clone(), state.config.clone());
    let user = auth_svc.register("totp1@example.com", "password123", None).await.unwrap();
    
    let (secret, qr) = mfa_svc.setup_totp(&user.id.to_string(), "TestIssuer").await.unwrap();
    assert!(!secret.is_empty());
    assert!(qr.contains("otpauth://"));
}

#[tokio::test]
async fn test_verify_totp_setup_wrong_code() {
    let state = setup_db().await;
    let auth_svc = AuthService::new(state.db.clone(), state.config.clone());
    let mfa_svc = MfaService::new(state.db.clone(), state.config.clone());
    let user = auth_svc.register("totp2@example.com", "password123", None).await.unwrap();
    
    mfa_svc.setup_totp(&user.id.to_string(), "Test").await.unwrap();
    let result = mfa_svc.verify_totp_setup(&user.id.to_string(), "000000").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_totp_wrong_code() {
    let state = setup_db().await;
    let auth_svc = AuthService::new(state.db.clone(), state.config.clone());
    let mfa_svc = MfaService::new(state.db.clone(), state.config.clone());
    let user = auth_svc.register("totp3@example.com", "password123", None).await.unwrap();
    
    let (secret, _) = mfa_svc.setup_totp(&user.id.to_string(), "Test").await.unwrap();
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1, 6, 1, 30,
        base32_decode(&secret).unwrap(),
        Some("Test".to_string()),
        "totp3@example.com".to_string(),
    ).unwrap();
    let code = totp.generate_current().unwrap();
    mfa_svc.verify_totp_setup(&user.id.to_string(), &code).await.unwrap();
    
    let valid = mfa_svc.verify_totp(&user.id.to_string(), "000000").await.unwrap();
    assert!(!valid);
}

#[tokio::test]
async fn test_disable_totp_wrong_code() {
    let state = setup_db().await;
    let auth_svc = AuthService::new(state.db.clone(), state.config.clone());
    let mfa_svc = MfaService::new(state.db.clone(), state.config.clone());
    let user = auth_svc.register("totp4@example.com", "password123", None).await.unwrap();
    
    let (secret, _) = mfa_svc.setup_totp(&user.id.to_string(), "Test").await.unwrap();
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1, 6, 1, 30,
        base32_decode(&secret).unwrap(),
        Some("Test".to_string()),
        "totp4@example.com".to_string(),
    ).unwrap();
    let code = totp.generate_current().unwrap();
    mfa_svc.verify_totp_setup(&user.id.to_string(), &code).await.unwrap();
    
    let result = mfa_svc.disable_totp(&user.id.to_string(), "000000").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_regenerate_backup_codes_count() {
    let state = setup_db().await;
    let auth_svc = AuthService::new(state.db.clone(), state.config.clone());
    let mfa_svc = MfaService::new(state.db.clone(), state.config.clone());
    let user = auth_svc.register("totp5@example.com", "password123", None).await.unwrap();
    
    let (secret, _) = mfa_svc.setup_totp(&user.id.to_string(), "Test").await.unwrap();
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1, 6, 1, 30,
        base32_decode(&secret).unwrap(),
        Some("Test".to_string()),
        "totp5@example.com".to_string(),
    ).unwrap();
    let code = totp.generate_current().unwrap();
    mfa_svc.verify_totp_setup(&user.id.to_string(), &code).await.unwrap();
    
    let codes = mfa_svc.regenerate_backup_codes(&user.id.to_string()).await.unwrap();
    assert_eq!(codes.len(), 10);
    
    let count = mfa_svc.get_backup_codes_count(&user.id.to_string()).await.unwrap();
    assert_eq!(count, 10);
}

#[tokio::test]
async fn test_get_backup_codes_count_no_totp() {
    let state = setup_db().await;
    let auth_svc = AuthService::new(state.db.clone(), state.config.clone());
    let mfa_svc = MfaService::new(state.db.clone(), state.config.clone());
    let user = auth_svc.register("totp6@example.com", "password123", None).await.unwrap();
    
    let result = mfa_svc.get_backup_codes_count(&user.id.to_string()).await;
    assert!(result.is_err());
}
