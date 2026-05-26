mod common;
use common::setup_db;
use oak_maillist::services::domain_service::DomainService;
use oak_maillist::services::list_service::ListService;
use oak_maillist::services::moderation_service::ModerationService;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

async fn create_test_moderation_item(
    state: &oak_maillist::models::AppState,
) -> oak_maillist::models::moderation_queue::Model {
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("mod.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc
        .create(&domain.id.to_string(), "ml", "ml", None, None)
        .await
        .unwrap();

    let item = oak_maillist::models::moderation_queue::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        list_id: Set(list.id),
        message_id: Set(None),
        from_addr: Set("sender@example.com".to_string()),
        subject: Set(Some("Test".to_string())),
        reason: Set("ai_moderation".to_string()),
        status: Set("pending".to_string()),
        source: Set("smtp".to_string()),
        ai_risk_score: Set(Some(90)),
        ai_labels: Set(None),
        ai_raw_response: Set(None),
        ai_reviewed: Set(false),
        moderated_by: Set(None),
        moderated_at: Set(None),
        moderation_note: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    };
    item.insert(&state.db).await.unwrap()
}

#[tokio::test]
async fn test_moderation_approve_not_found() {
    let state = setup_db().await;
    let svc = ModerationService::new(state.db.clone());
    let result = svc
        .approve("550e8400-e29b-41d4-a716-446655440000", None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_moderation_reject_not_found() {
    let state = setup_db().await;
    let svc = ModerationService::new(state.db.clone());
    let result = svc
        .reject("550e8400-e29b-41d4-a716-446655440000", None, Some("spam"))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_moderation_discard_not_found() {
    let state = setup_db().await;
    let svc = ModerationService::new(state.db.clone());
    let result = svc
        .discard("550e8400-e29b-41d4-a716-446655440000", None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_moderation_whitelist_creates_policy() {
    let state = setup_db().await;
    let item = create_test_moderation_item(&state).await;
    let svc = ModerationService::new(state.db.clone());

    svc.whitelist_sender(&item.id.to_string(), None)
        .await
        .unwrap();

    let updated = oak_maillist::models::moderation_queue::Entity::find_by_id(item.id)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, "approved");

    let policies = oak_maillist::models::sender_policy::Entity::find()
        .all(&state.db)
        .await
        .unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0].policy_type, "whitelist");
}

#[tokio::test]
async fn test_moderation_blacklist_creates_policy() {
    let state = setup_db().await;
    let item = create_test_moderation_item(&state).await;
    let svc = ModerationService::new(state.db.clone());

    svc.blacklist_sender(&item.id.to_string(), None)
        .await
        .unwrap();

    let updated = oak_maillist::models::moderation_queue::Entity::find_by_id(item.id)
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, "rejected");

    let policies = oak_maillist::models::sender_policy::Entity::find()
        .all(&state.db)
        .await
        .unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0].policy_type, "blacklist");
}
