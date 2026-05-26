use axum::response::IntoResponse;
use oak_maillist::utils::{crypto, email, response, validation};

#[test]
fn test_generate_random_token() {
    let t1 = crypto::generate_random_token(16);
    let t2 = crypto::generate_random_token(16);
    assert_eq!(t1.len(), 16);
    assert_eq!(t2.len(), 16);
    assert_ne!(t1, t2);
}

#[test]
fn test_generate_random_token_zero_length() {
    let t = crypto::generate_random_token(0);
    assert_eq!(t.len(), 0);
}

#[test]
fn test_generate_uuid() {
    let u1 = crypto::generate_uuid();
    let u2 = crypto::generate_uuid();
    assert_ne!(u1, u2);
}

#[test]
fn test_hash_and_verify_password() {
    let password = "my_secret_password_123";
    let hash = crypto::hash_password(password).unwrap();
    assert!(crypto::verify_password(password, &hash).unwrap());
    assert!(!crypto::verify_password("wrong_password", &hash).unwrap());
}

#[test]
fn test_verify_password_invalid_hash() {
    let result = crypto::verify_password("password", "not-a-valid-hash");
    assert!(result.is_err());
}

#[test]
fn test_extract_local_part() {
    assert_eq!(email::extract_local_part("user@example.com"), Some("user"));
    assert_eq!(email::extract_local_part("invalid"), Some("invalid"));
}

#[test]
fn test_extract_local_part_empty() {
    assert_eq!(email::extract_local_part(""), Some(""));
}

#[test]
fn test_extract_local_part_multiple_at() {
    assert_eq!(email::extract_local_part("a@b@c.com"), Some("a"));
}

#[test]
fn test_extract_domain() {
    assert_eq!(
        email::extract_domain("user@example.com"),
        Some("example.com")
    );
    assert_eq!(email::extract_domain("invalid"), None);
}

#[test]
fn test_extract_domain_empty() {
    assert_eq!(email::extract_domain(""), None);
}

#[test]
fn test_extract_domain_multiple_at() {
    assert_eq!(email::extract_domain("a@b@c.com"), Some("b"));
}

#[test]
fn test_build_list_email() {
    assert_eq!(
        email::build_list_email("list", "example.com"),
        "list@example.com"
    );
}

#[test]
fn test_build_verp_address() {
    assert_eq!(
        email::build_verp_address("list", "example.com", "abc123"),
        "list-bounces+abc123@example.com"
    );
}

#[test]
fn test_build_verp_address_special_token() {
    assert_eq!(
        email::build_verp_address("list", "example.com", "token-with-dash_123"),
        "list-bounces+token-with-dash_123@example.com"
    );
}

#[test]
fn test_api_response_new() {
    let resp = response::ApiResponse::new("hello");
    assert!(resp.meta.is_none());
}

#[test]
fn test_api_response_with_meta() {
    let resp = response::ApiResponse::with_meta("hello", serde_json::json!({"count": 1}));
    assert!(resp.meta.is_some());
}

#[test]
fn test_api_error_into_response_validation_error() {
    let err = response::ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "bad".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[test]
fn test_api_error_into_response_unauthorized() {
    let err = response::ApiError {
        code: "UNAUTHORIZED".to_string(),
        message: "no".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[test]
fn test_api_error_into_response_forbidden() {
    let err = response::ApiError {
        code: "FORBIDDEN".to_string(),
        message: "no".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
}

#[test]
fn test_api_error_into_response_mfa_required() {
    let err = response::ApiError {
        code: "MFA_REQUIRED".to_string(),
        message: "mfa".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
}

#[test]
fn test_api_error_into_response_not_found() {
    let err = response::ApiError {
        code: "NOT_FOUND".to_string(),
        message: "gone".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
}

#[test]
fn test_api_error_into_response_conflict() {
    let err = response::ApiError {
        code: "CONFLICT".to_string(),
        message: "dup".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::CONFLICT);
}

#[test]
fn test_api_error_into_response_rate_limited() {
    let err = response::ApiError {
        code: "RATE_LIMITED".to_string(),
        message: "slow".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn test_api_error_into_response_ai_moderation_error() {
    let err = response::ApiError {
        code: "AI_MODERATION_ERROR".to_string(),
        message: "ai down".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn test_api_error_into_response_default_500() {
    let err = response::ApiError {
        code: "UNKNOWN".to_string(),
        message: "oops".to_string(),
        details: None,
        request_id: None,
    };
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_is_valid_email_standard() {
    assert!(validation::is_valid_email("user@example.com"));
    assert!(validation::is_valid_email("user+tag@example.com"));
    assert!(validation::is_valid_email("first.last@sub.example.co.uk"));
}

#[test]
fn test_is_valid_email_invalid() {
    assert!(!validation::is_valid_email(""));
    assert!(!validation::is_valid_email("no-at-sign"));
    assert!(!validation::is_valid_email("@example.com"));
    assert!(!validation::is_valid_email("user@"));
}

#[test]
fn test_is_valid_email_too_long() {
    let local = "a".repeat(250);
    let email = format!("{}@example.com", local);
    assert!(!validation::is_valid_email(&email));
}

#[test]
fn test_is_valid_domain_standard() {
    assert!(validation::is_valid_domain("example.com"));
    assert!(validation::is_valid_domain("sub.example.co.uk"));
    assert!(validation::is_valid_domain("localhost"));
}

#[test]
fn test_is_valid_domain_invalid() {
    assert!(!validation::is_valid_domain(""));
    assert!(!validation::is_valid_domain("example..com"));
    assert!(!validation::is_valid_domain("-example.com"));
    assert!(!validation::is_valid_domain("example.com:8080"));
}

#[test]
fn test_is_valid_domain_too_long() {
    let domain = "a".repeat(254);
    assert!(!validation::is_valid_domain(&domain));
}

#[test]
fn test_normalize_email() {
    assert_eq!(
        validation::normalize_email("User@Example.COM"),
        "user@example.com"
    );
    assert_eq!(
        validation::normalize_email("  User@Example.COM  "),
        "user@example.com"
    );
}
