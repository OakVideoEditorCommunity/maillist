mod common;
use common::setup_db;
use oak_maillist::services::domain_service::DomainService;
use oak_maillist::services::list_service::ListService;
use sea_orm::Set;

#[tokio::test]
async fn test_list_create_defaults() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("lists.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    
    let list = list_svc.create(&domain.id.to_string(), "my-list", "my", Some("My List"), None).await.unwrap();
    assert_eq!(list.name, "my-list");
    assert_eq!(list.visibility, "public");
    assert_eq!(list.subscription_policy, "confirm");
    assert_eq!(list.post_policy, "subscriber_only");
    assert_eq!(list.reply_to, "list");
    assert!(list.archive_enabled);
    assert!(list.ai_moderation_enabled);
    assert!(list.is_active);
}

#[tokio::test]
async fn test_list_create_invalid_domain() {
    let state = setup_db().await;
    let list_svc = ListService::new(state.db.clone());
    let result = list_svc.create("not-a-uuid", "list", "l", None, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_public_pagination() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("pag.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    
    list_svc.create(&domain.id.to_string(), "l1", "l1", None, None).await.unwrap();
    list_svc.create(&domain.id.to_string(), "l2", "l2", None, None).await.unwrap();
    
    let (items, total) = list_svc.list_public(1, 10).await.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(total, 2);
}

#[tokio::test]
async fn test_list_public_excludes_inactive() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("excl.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    
    let list = list_svc.create(&domain.id.to_string(), "del", "del", None, None).await.unwrap();
    list_svc.delete(&list.id.to_string()).await.unwrap();
    
    let (items, total) = list_svc.list_public(1, 10).await.unwrap();
    assert_eq!(total, 0);
}

#[tokio::test]
async fn test_list_update_partial() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("upd.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "upd", "upd", None, None).await.unwrap();
    
    let updated = list_svc.update(&list.id.to_string(), serde_json::json!({"display_name": "Updated"})).await.unwrap();
    assert_eq!(updated.display_name, Some("Updated".to_string()));
    assert_eq!(updated.name, "upd"); // unchanged
}

#[tokio::test]
async fn test_list_delete_soft_delete() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("soft.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "soft", "soft", None, None).await.unwrap();
    
    list_svc.delete(&list.id.to_string()).await.unwrap();
    let found = list_svc.find_by_id(&list.id.to_string()).await.unwrap();
    assert!(found.is_some());
    assert!(!found.unwrap().is_active);
}
