mod common;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::setup_app;
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
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_list_domains_requires_auth() {
    let app = setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/domains")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_domain_success() {
    let app = setup_app().await;
    let token = get_token(&app, "domain@example.com").await;
    let body = serde_json::json!({"name": "example.com"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/domains")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_domain_missing_name() {
    let app = setup_app().await;
    let token = get_token(&app, "domain2@example.com").await;
    let body = serde_json::json!({});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/domains")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_domain_not_found() {
    let app = setup_app().await;
    let token = get_token(&app, "domain3@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/domains/550e8400-e29b-41d4-a716-446655440000")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_domain_invalid_uuid() {
    let app = setup_app().await;
    let token = get_token(&app, "domain4@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/domains/not-a-uuid")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_domain_crud() {
    let app = setup_app().await;
    let token = get_token(&app, "domain5@example.com").await;

    // Create
    let body = serde_json::json!({"name": "crud.com"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/domains")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let domain_id = json["data"]["id"].as_str().unwrap();

    // Get
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/domains/{}", domain_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Update
    let body = serde_json::json!({"smtp_host": "smtp.crud.com"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/domains/{}", domain_id))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Delete
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/domains/{}", domain_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_verify_dkim_stub() {
    let app = setup_app().await;
    let token = get_token(&app, "dkim@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/domains/550e8400-e29b-41d4-a716-446655440000/verify-dkim")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
