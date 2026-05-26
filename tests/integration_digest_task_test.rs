mod common;
use common::setup_db;
use oak_maillist::tasks::digest::DigestTask;
use oak_maillist::services::domain_service::DomainService;
use oak_maillist::services::list_service::ListService;
use oak_maillist::services::subscriber_service::SubscriberService;
use sea_orm::{ActiveModelTrait, Set};

#[tokio::test]
async fn test_digest_no_subscribers() {
    let state: oak_maillist::models::AppState = setup_db().await;
    let task = DigestTask::new(state.clone());
    let result = task.run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_digest_no_messages() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("dig.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());
    
    let sub = sub_svc.subscribe(&list.id.to_string(), "user@example.com", None, "http://localhost").await.unwrap();
    sub_svc.confirm(&list.id.to_string(), &sub.token).await.unwrap();
    sub_svc.update_digest_mode(&sub.id.to_string(), "daily").await.unwrap();
    
    let task = DigestTask::new(state.clone());
    let result = task.run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_digest_skips_when_smtp_not_configured() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("dig2.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());
    
    let sub = sub_svc.subscribe(&list.id.to_string(), "user@example.com", None, "http://localhost").await.unwrap();
    sub_svc.confirm(&list.id.to_string(), &sub.token).await.unwrap();
    sub_svc.update_digest_mode(&sub.id.to_string(), "daily").await.unwrap();
    
    let msg = oak_maillist::models::email_message::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        list_id: Set(list.id),
        message_id: Set("msg-id".to_string()),
        in_reply_to: Set(None),
        references: Set(None),
        from_name: Set(None),
        from_addr: Set("sender@example.com".to_string()),
        to_addr: Set(None),
        subject: Set(Some("Hello".to_string())),
        subject_normalized: Set(None),
        body_text: Set(Some("Body".to_string())),
        body_html: Set(None),
        raw_content: Set(None),
        size_bytes: Set(Some(0)),
        has_attachments: Set(false),
        received_at: Set(chrono::Utc::now().into()),
        thread_id: Set(None),
        is_deleted: Set(false),
        deleted_at: Set(None),
        deleted_by: Set(None),
        deleted_reason: Set(None),
    };
    msg.insert(&state.db).await.unwrap();
    
    let task = DigestTask::new(state.clone());
    let result = task.run().await;
    assert!(result.is_ok());
}
