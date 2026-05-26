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

async fn create_domain_and_list(app: &axum::Router, token: &str) -> (String, String) {
    let body = serde_json::json!({"name": "lists.example.com"});
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
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let domain_id = json["data"]["id"].as_str().unwrap().to_string();

    let body = serde_json::json!({"domain_id": domain_id, "name": "my-list", "email_local_part": "my", "display_name": "My List"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/lists")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let list_id = json["data"]["id"].as_str().unwrap().to_string();

    (domain_id, list_id)
}

#[tokio::test]
async fn test_list_public_lists() {
    let app = setup_app().await;
    let token = get_token(&app, "listpub@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/lists")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_list_requires_auth() {
    let app = setup_app().await;
    let body = serde_json::json!({"name": "l", "email_local_part": "l"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/lists")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_crud() {
    let app = setup_app().await;
    let token = get_token(&app, "listcrud@example.com").await;
    let (_domain_id, list_id) = create_domain_and_list(&app, &token).await;

    // Get
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/lists/{}", list_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Update settings
    let body = serde_json::json!({"display_name": "Updated List"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/lists/{}", list_id))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Get settings
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/lists/{}/settings", list_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
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
                .uri(format!("/api/v1/lists/{}", list_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_subscribe_and_confirm() {
    let app = setup_app().await;
    let token = get_token(&app, "listsub@example.com").await;
    let (_domain_id, list_id) = create_domain_and_list(&app, &token).await;

    // Subscribe
    let body = serde_json::json!({"email": "subscriber@example.com", "name": "Sub"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/lists/{}/subscribe", list_id))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let sub_token = json["data"]["token"].as_str().unwrap();

    // Confirm
    let body = serde_json::json!({"token": sub_token});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/lists/{}/subscribers/confirm", list_id))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Unsubscribe
    let body = serde_json::json!({"token": sub_token});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/lists/{}/unsubscribe", list_id))
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
async fn test_list_archive() {
    let app = setup_app().await;
    let token = get_token(&app, "listarch@example.com").await;
    let (_domain_id, list_id) = create_domain_and_list(&app, &token).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/lists/{}/archive", list_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_stats() {
    let app = setup_app().await;
    let token = get_token(&app, "liststat@example.com").await;
    let (_domain_id, list_id) = create_domain_and_list(&app, &token).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/lists/{}/stats", list_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_search() {
    let app = setup_app().await;
    let token = get_token(&app, "listsearch@example.com").await;
    let (_domain_id, list_id) = create_domain_and_list(&app, &token).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/lists/{}/search?q=test", list_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
