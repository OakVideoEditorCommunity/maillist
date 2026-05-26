mod common;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::setup_app;
use tower::ServiceExt;

async fn register_and_login(app: &axum::Router, email: &str, password: &str) -> String {
    let body = serde_json::json!({"email": email, "password": password, "name": "Test"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = serde_json::json!({"email": email, "password": password});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_register_success() {
    let app = setup_app().await;
    let body = serde_json::json!({"email": "reg@example.com", "password": "password123"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let app = setup_app().await;
    let body = serde_json::json!({"email": "dup@example.com", "password": "password123"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_register_password_too_short() {
    let app = setup_app().await;
    let body = serde_json::json!({"email": "short@example.com", "password": "123"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_success() {
    let app = setup_app().await;
    let token = register_and_login(&app, "login@example.com", "password123").await;
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let app = setup_app().await;
    register_and_login(&app, "wrong@example.com", "password123").await;

    let body = serde_json::json!({"email": "wrong@example.com", "password": "badpass"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let app = setup_app().await;
    let body = serde_json::json!({"email": "nope@example.com", "password": "password123"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_all_requires_auth() {
    let app = setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout-all")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_refresh_invalid_token() {
    let app = setup_app().await;
    let body = serde_json::json!({"refresh_token": "invalid-token"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_forgot_password_stub() {
    let app = setup_app().await;
    let body = serde_json::json!({"email": "test@example.com"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/forgot-password")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_magic_link_callback_stub() {
    let app = setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/magic-link/callback")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_passkey_register_requires_auth() {
    let app = setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/passkey/register")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"challenge_id":"x","credential":{"id":"x","rawId":"eQ","response":{"attestationObject":"eQ","clientDataJSON":"eQ"},"type":"public-key"}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_passkey_login_validation_error() {
    let app = setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/passkey/login")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    // Missing required fields causes JSON deserialization error (422 in axum 0.8)
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
