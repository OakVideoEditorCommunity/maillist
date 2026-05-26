use oak_maillist::utils::{crypto, email, response};

#[test]
fn test_generate_random_token() {
    let t1 = crypto::generate_random_token(16);
    let t2 = crypto::generate_random_token(16);
    assert_eq!(t1.len(), 16);
    assert_eq!(t2.len(), 16);
    assert_ne!(t1, t2);
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
fn test_extract_local_part() {
    assert_eq!(email::extract_local_part("user@example.com"), Some("user"));
    assert_eq!(email::extract_local_part("invalid"), Some("invalid"));
}

#[test]
fn test_extract_domain() {
    assert_eq!(email::extract_domain("user@example.com"), Some("example.com"));
    assert_eq!(email::extract_domain("invalid"), None);
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
fn test_api_response_new() {
    let resp = response::ApiResponse::new("hello");
    assert!(resp.meta.is_none());
}

#[test]
fn test_api_response_with_meta() {
    let resp = response::ApiResponse::with_meta("hello", serde_json::json!({"count": 1}));
    assert!(resp.meta.is_some());
}
