mod common;
use common::setup_db;
use oak_maillist::services::auth_service::AuthService;

#[tokio::test]
async fn test_register_duplicate_email() {
    let state = setup_db().await;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    svc.register("dup@example.com", "password123", None)
        .await
        .unwrap();
    let result = svc.register("dup@example.com", "password123", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_login_wrong_email() {
    let state = setup_db().await;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    let result = svc.login("nope@example.com", "password123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_login_inactive_user() {
    let state = setup_db().await;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    let user = svc
        .register("inactive@example.com", "password123", None)
        .await
        .unwrap();

    use sea_orm::{ActiveModelTrait, Set};
    let mut active: oak_maillist::models::user::ActiveModel = user.into();
    active.is_active = Set(false);
    active.update(&state.db).await.unwrap();

    let result = svc.login("inactive@example.com", "password123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_access_token_roundtrip() {
    let state = setup_db().await;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    let user = svc
        .register("token@example.com", "password123", None)
        .await
        .unwrap();

    let token = svc
        .generate_access_token(&user.id.to_string(), &user.email, "subscriber")
        .unwrap();
    assert!(!token.is_empty());

    let claims = svc.verify_access_token(&token).unwrap();
    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.email, user.email);
    assert_eq!(claims.role, "subscriber");
}

#[tokio::test]
async fn test_verify_access_token_invalid() {
    let state = setup_db().await;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    let result = svc.verify_access_token("not.a.valid.token");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_refresh_token_revoke_and_verify() {
    let state = setup_db().await;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    let user = svc
        .register("refresh@example.com", "password123", None)
        .await
        .unwrap();

    let token = svc
        .create_refresh_token(&user.id.to_string(), None)
        .await
        .unwrap();
    let verified = svc.verify_refresh_token(&token).await.unwrap();
    assert_eq!(verified.id, user.id);

    svc.revoke_refresh_token(&token).await.unwrap();
    let result = svc.verify_refresh_token(&token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_revoke_all_user_tokens() {
    let state = setup_db().await;
    let svc = AuthService::new(state.db.clone(), state.config.clone());
    let user = svc
        .register("revokeall@example.com", "password123", None)
        .await
        .unwrap();

    let t1 = svc
        .create_refresh_token(&user.id.to_string(), None)
        .await
        .unwrap();
    let t2 = svc
        .create_refresh_token(&user.id.to_string(), None)
        .await
        .unwrap();

    svc.revoke_all_user_tokens(&user.id.to_string())
        .await
        .unwrap();

    assert!(svc.verify_refresh_token(&t1).await.is_err());
    assert!(svc.verify_refresh_token(&t2).await.is_err());
}
