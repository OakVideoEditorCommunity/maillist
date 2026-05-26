mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
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

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_admin_dashboard() {
    let app = common::setup_app().await;
    let token = register_and_login(&app, "admin1@example.com", "password123").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/dashboard")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["data"]["users"].is_u64());
    assert!(json["data"]["lists"].is_u64());
    assert!(json["data"]["subscribers"].is_u64());
    assert!(json["data"]["messages"].is_u64());
}

#[tokio::test]
async fn test_admin_stats() {
    let app = common::setup_app().await;
    let token = register_and_login(&app, "admin2@example.com", "password123").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/stats")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["data"]["pending_moderation"].is_u64());
}

#[tokio::test]
async fn test_admin_activity_log() {
    let app = common::setup_app().await;
    let token = register_and_login(&app, "admin3@example.com", "password123").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/activity-log")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_get_settings() {
    let app = common::setup_app().await;
    let token = register_and_login(&app, "admin4@example.com", "password123").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/settings")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["data"]["server"].is_object());
    assert!(json["data"]["smtp"].is_object());
    assert!(json["data"]["ai_moderation"].is_object());
}

#[tokio::test]
async fn test_admin_update_settings_stub() {
    let app = common::setup_app().await;
    let token = register_and_login(&app, "admin5@example.com", "password123").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/settings")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"smtp_host":"smtp.example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_ai_moderation_stats() {
    let app = common::setup_app().await;
    let token = register_and_login(&app, "admin6@example.com", "password123").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/ai-moderation/stats")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["data"]["total"].is_u64());
    assert!(json["data"]["pending"].is_u64());
    assert!(json["data"]["approved"].is_u64());
    assert!(json["data"]["rejected"].is_u64());
}

#[tokio::test]
async fn test_admin_requires_auth() {
    let app = common::setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/dashboard")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
