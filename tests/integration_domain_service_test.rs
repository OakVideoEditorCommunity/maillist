mod common;
use common::setup_db;
use oak_maillist::services::domain_service::DomainService;

#[tokio::test]
async fn test_domain_find_by_id_not_found() {
    let state = setup_db().await;
    let svc = DomainService::new(state.db.clone());
    let found = svc.find_by_id("550e8400-e29b-41d4-a716-446655440000").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_domain_update_partial() {
    let state = setup_db().await;
    let svc = DomainService::new(state.db.clone());
    let domain = svc.create("partial.com").await.unwrap();
    
    let updated = svc.update(&domain.id.to_string(), serde_json::json!({"smtp_host": "smtp.partial.com"})).await.unwrap();
    assert_eq!(updated.smtp_host, Some("smtp.partial.com".to_string()));
    assert_eq!(updated.name, "partial.com"); // unchanged
}

#[tokio::test]
async fn test_domain_update_not_found() {
    let state = setup_db().await;
    let svc = DomainService::new(state.db.clone());
    let result = svc.update("550e8400-e29b-41d4-a716-446655440000", serde_json::json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_domain_delete_not_found() {
    let state = setup_db().await;
    let svc = DomainService::new(state.db.clone());
    let result = svc.delete("550e8400-e29b-41d4-a716-446655440000").await;
    // SeaORM delete_by_id on non-existent returns Ok(0)
    assert!(result.is_ok());
}
