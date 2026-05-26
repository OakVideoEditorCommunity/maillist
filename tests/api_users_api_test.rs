mod common;
use common::setup_app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

async fn get_token(app: &axum::Router, email: &str) -> String {
    let body = serde_json::json!({"email": email, "password": "password123", "name": "Test"});
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
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_get_me_requires_auth() {
    let app = setup_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_me_success() {
    let app = setup_app().await;
    let token = get_token(&app, "me@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/me")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    if status != StatusCode::OK {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8_lossy(&bytes);
        panic!("Unexpected status: {:?}, body: {}", status, body);
    }
}

#[tokio::test]
async fn test_update_me_success() {
    let app = setup_app().await;
    let token = get_token(&app, "me2@example.com").await;
    let body = serde_json::json!({"name": "New Name"});
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/users/me")
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
async fn test_change_password_wrong_old() {
    let app = setup_app().await;
    let token = get_token(&app, "pwd@example.com").await;
    let body = serde_json::json!({"old_password": "wrong", "new_password": "newpassword123"});
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/users/me/password")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_change_password_too_short() {
    let app = setup_app().await;
    let token = get_token(&app, "pwd2@example.com").await;
    let body = serde_json::json!({"old_password": "password123", "new_password": "123"});
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/users/me/password")
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
async fn test_list_sessions() {
    let app = setup_app().await;
    let token = get_token(&app, "sess@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/me/sessions")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_mfa_status() {
    let app = setup_app().await;
    let token = get_token(&app, "mfastat@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/me/mfa")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_passkeys() {
    let app = setup_app().await;
    let token = get_token(&app, "pk@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/me/passkeys")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_user() {
    let app = setup_app().await;
    let token = get_token(&app, "userget@example.com").await;
    
    // Get current user to know ID
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/me")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let user_id = json["data"]["id"].as_str().unwrap();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/users/{}", user_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_users() {
    let app = setup_app().await;
    let token = get_token(&app, "listu@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
