mod common;
use common::test_config;
use oak_maillist::services::ai_service::AiService;

#[test]
fn test_ai_service_should_flag() {
    let config = test_config();
    let svc = AiService::new(config.ai_moderation.clone());
    assert!(svc.should_flag(80));
    assert!(svc.should_flag(100));
    assert!(!svc.should_flag(79));
    assert!(!svc.should_flag(50));
}

#[test]
fn test_ai_service_should_caution() {
    let config = test_config();
    let svc = AiService::new(config.ai_moderation.clone());
    assert!(svc.should_caution(50));
    assert!(svc.should_caution(79));
    assert!(!svc.should_caution(80));
    assert!(!svc.should_caution(49));
}

#[tokio::test]
async fn test_ai_service_moderate_disabled() {
    let mut config = test_config();
    config.ai_moderation.enabled = false;
    let svc = AiService::new(config.ai_moderation);
    let result = svc.moderate_email("subject", "body").await.unwrap();
    assert_eq!(result.verdict, "clean");
    assert_eq!(result.risk_level, "none");
}
