mod common;
use common::setup_app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

async fn get_token(app: &axum::Router, email: &str) -> String {
    let body = serde_json::json!({"email": email, "password": "password123", "name": "Test"});
    app.clone()
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

    let body = serde_json::json!({"email": email, "password": "password123"});
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
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_moderation_get_not_found() {
    let app = setup_app().await;
    let token = get_token(&app, "mod@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/moderation/550e8400-e29b-41d4-a716-446655440000")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_moderation_approve_requires_auth() {
    let app = setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/moderation/550e8400-e29b-41d4-a716-446655440000/approve")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_moderation_ai_feedback() {
    let app = setup_app().await;
    let token = get_token(&app, "modai@example.com").await;
    let body = serde_json::json!({"feedback": "good"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/moderation/550e8400-e29b-41d4-a716-446655440000/ai-feedback")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
