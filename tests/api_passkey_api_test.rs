use axum::{body::Body, extract::Request, http::StatusCode};
use tower::ServiceExt;

mod common;

async fn register_and_login(app: &axum::Router) -> String {
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"passkey@example.com","password":"password123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"passkey@example.com","password":"password123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_passkey_register_options_success() {
    let app = common::setup_app().await;
    let token = register_and_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/passkey/register-options")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["data"]["challenge_id"].as_str().is_some());
    assert!(json["data"]["options"].is_object());
}

#[tokio::test]
async fn test_passkey_register_options_requires_auth() {
    let app = common::setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/passkey/register-options")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_passkey_register_invalid_challenge() {
    let app = common::setup_app().await;
    let token = register_and_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/passkey/register")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(r#"{"challenge_id":"invalid-challenge","credential":{"id":"x","rawId":"eQ","response":{"attestationObject":"eQ","clientDataJSON":"eQ"},"type":"public-key"}}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_passkey_auth_options_user_not_found() {
    let app = common::setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/passkey/auth-options")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"nobody@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_passkey_auth_options_no_passkey_registered() {
    let app = common::setup_app().await;
    let _token = register_and_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/passkey/auth-options")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"passkey@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_passkey_login_invalid_challenge() {
    let app = common::setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/passkey/login")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"challenge_id":"invalid-challenge","credential":{"id":"x","rawId":"eQ","response":{"authenticatorData":"eQ","clientDataJSON":"eQ","signature":"eQ"},"type":"public-key"}}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
