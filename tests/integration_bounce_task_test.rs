mod common;
use common::setup_db;
use oak_maillist::tasks::bounce::BounceProcessor;
use sea_orm::EntityTrait;
use oak_maillist::services::domain_service::DomainService;
use oak_maillist::services::list_service::ListService;
use oak_maillist::services::subscriber_service::SubscriberService;

#[tokio::test]
async fn test_bounce_unknown_token() {
    let state = setup_db().await;
    let processor = BounceProcessor::new(state.db.clone());
    let result = processor.process_bounce("unknown-token", "hard", "bounce").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_bounce_hard_threshold() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("bounce.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());
    
    let sub = sub_svc.subscribe(&list.id.to_string(), "user@example.com", None, "http://localhost").await.unwrap();
    sub_svc.confirm(&list.id.to_string(), &sub.token).await.unwrap();
    
    let processor = BounceProcessor::new(state.db.clone());
    processor.process_bounce(&sub.token, "hard", "bounce1").await.unwrap();
    processor.process_bounce(&sub.token, "hard", "bounce2").await.unwrap();
    
    let sub2 = oak_maillist::models::subscriber::Entity::find_by_id(sub.id)
        .one(&state.db).await.unwrap().unwrap();
    assert_eq!(sub2.bounce_count, 2);
    assert_eq!(sub2.status, "active");
    
    processor.process_bounce(&sub.token, "hard", "bounce3").await.unwrap();
    let sub3 = oak_maillist::models::subscriber::Entity::find_by_id(sub.id)
        .one(&state.db).await.unwrap().unwrap();
    assert_eq!(sub3.bounce_count, 3);
    assert_eq!(sub3.status, "unsubscribed");
}

#[tokio::test]
async fn test_bounce_soft_no_unsubscribe() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("soft.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());
    
    let sub = sub_svc.subscribe(&list.id.to_string(), "user@example.com", None, "http://localhost").await.unwrap();
    sub_svc.confirm(&list.id.to_string(), &sub.token).await.unwrap();
    
    let processor = BounceProcessor::new(state.db.clone());
    for _ in 0..5 {
        processor.process_bounce(&sub.token, "soft", "soft_bounce").await.unwrap();
    }
    
    let sub2 = oak_maillist::models::subscriber::Entity::find_by_id(sub.id)
        .one(&state.db).await.unwrap().unwrap();
    assert_eq!(sub2.bounce_count, 5);
    assert_eq!(sub2.status, "active");
}

#[tokio::test]
async fn test_bounce_unknown_type() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("unk.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());
    
    let sub = sub_svc.subscribe(&list.id.to_string(), "user@example.com", None, "http://localhost").await.unwrap();
    sub_svc.confirm(&list.id.to_string(), &sub.token).await.unwrap();
    
    let processor = BounceProcessor::new(state.db.clone());
    let result = processor.process_bounce(&sub.token, "unknown_type", "reason").await;
    assert!(result.is_ok());
}
