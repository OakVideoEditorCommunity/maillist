use oak_maillist::ai::aliyun_signer::AliyunV3Signer;
use oak_maillist::ai::parser::parse_ai_response;
use oak_maillist::ai::policy::ModerationPolicy;

#[test]
fn test_moderation_policy_verdict_flagged() {
    let policy = ModerationPolicy {
        high_risk_threshold: 80,
        medium_risk_threshold: 50,
    };
    assert_eq!(policy.verdict(80), "flagged");
    assert_eq!(policy.verdict(100), "flagged");
}

#[test]
fn test_moderation_policy_verdict_caution() {
    let policy = ModerationPolicy {
        high_risk_threshold: 80,
        medium_risk_threshold: 50,
    };
    assert_eq!(policy.verdict(50), "caution");
    assert_eq!(policy.verdict(79), "caution");
}

#[test]
fn test_moderation_policy_verdict_clean() {
    let policy = ModerationPolicy {
        high_risk_threshold: 80,
        medium_risk_threshold: 50,
    };
    assert_eq!(policy.verdict(0), "clean");
    assert_eq!(policy.verdict(49), "clean");
}

#[test]
fn test_parse_ai_response_returns_clean_default() {
    let result = parse_ai_response("any content").unwrap();
    assert_eq!(result.overall_score, 0);
    assert_eq!(result.verdict, "clean");
    assert_eq!(result.risk_level, "none");
    assert!(result.flagged_categories.is_empty());
}

#[test]
fn test_aliyun_v3_signer_produces_required_headers() {
    let signer = AliyunV3Signer::new(
        "ak_id".to_string(),
        "ak_secret".to_string(),
        "cn-shanghai".to_string(),
        "nlp".to_string(),
    );

    let headers = signer.sign_request(
        "POST",
        "/api/test",
        "",
        &[("Content-Type".to_string(), "application/json".to_string())],
        b"{}",
    );

    let header_names: Vec<&str> = headers.iter().map(|(k, _)| k.as_str()).collect();
    assert!(header_names.contains(&"x-acs-content-sha256"));
    assert!(header_names.contains(&"x-acs-date"));
    assert!(header_names.contains(&"x-acs-signature-nonce"));
    assert!(header_names.contains(&"x-acs-version"));
    assert!(header_names.contains(&"Authorization"));

    let auth = headers
        .iter()
        .find(|(k, _)| k == "Authorization")
        .map(|(_, v)| v)
        .unwrap();
    assert!(auth.starts_with("ACS3-HMAC-SHA256 Credential=ak_id"));
    assert!(auth.contains("Signature="));
}

#[test]
fn test_aliyun_v3_signer_empty_body() {
    let signer = AliyunV3Signer::new(
        "ak".to_string(),
        "secret".to_string(),
        "ap-southeast-1".to_string(),
        "vision".to_string(),
    );

    let headers = signer.sign_request("GET", "/", "", &[], b"");
    assert!(headers.iter().any(|(k, _)| k == "Authorization"));
}

#[test]
fn test_aliyun_v3_signer_with_query() {
    let signer = AliyunV3Signer::new(
        "ak".to_string(),
        "secret".to_string(),
        "cn-beijing".to_string(),
        "ocr".to_string(),
    );

    let headers = signer.sign_request("GET", "/api/resource", "key=value&foo=bar", &[], b"");
    assert!(headers.iter().any(|(k, _)| k == "Authorization"));
}
