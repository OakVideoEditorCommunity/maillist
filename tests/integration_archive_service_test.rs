mod common;
use common::setup_db;
use oak_maillist::services::archive_service::ArchiveService;
use sea_orm::EntityTrait;
use oak_maillist::services::domain_service::DomainService;
use oak_maillist::services::list_service::ListService;
use sea_orm::{ActiveModelTrait, Set};

async fn create_test_message(
    db: &sea_orm::DatabaseConnection,
    list_id: uuid::Uuid,
    message_id: Option<&str>,
    in_reply_to: Option<&str>,
    subject: &str,
    body: &str,
) -> oak_maillist::models::email_message::Model {
    let msg = oak_maillist::models::email_message::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        list_id: Set(list_id),
        message_id: Set(message_id.unwrap_or("msg-id").to_string()),
        in_reply_to: Set(in_reply_to.map(|s| s.to_string())),
        references: Set(None),
        from_name: Set(None),
        from_addr: Set("sender@example.com".to_string()),
        to_addr: Set(None),
        subject: Set(Some(subject.to_string())),
        subject_normalized: Set(None),
        body_text: Set(Some(body.to_string())),
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
    msg.insert(db).await.unwrap()
}

#[tokio::test]
async fn test_build_threads_no_reply_headers() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("arch.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let archive_svc = ArchiveService::new(state.db.clone());
    
    let msg = create_test_message(&state.db, list.id, Some("msg-1"), None, "Hello", "Body").await;
    archive_svc.build_threads(&list.id.to_string()).await.unwrap();
    
    let updated = oak_maillist::models::email_message::Entity::find_by_id(msg.id)
        .one(&state.db).await.unwrap().unwrap();
    assert_eq!(updated.thread_id, Some(msg.id));
}

#[tokio::test]
async fn test_build_threads_with_in_reply_to() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("arch2.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let archive_svc = ArchiveService::new(state.db.clone());
    
    let parent = create_test_message(&state.db, list.id, Some("parent-id"), None, "Parent", "Body").await;
    archive_svc.build_threads(&list.id.to_string()).await.unwrap();
    let child = create_test_message(&state.db, list.id, Some("child-id"), Some("parent-id"), "Re: Parent", "Reply").await;
    archive_svc.build_threads(&list.id.to_string()).await.unwrap();
    
    let updated = oak_maillist::models::email_message::Entity::find_by_id(child.id)
        .one(&state.db).await.unwrap().unwrap();
    assert_eq!(updated.thread_id, Some(parent.id));
}

#[tokio::test]
async fn test_search_by_keyword() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("search.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let archive_svc = ArchiveService::new(state.db.clone());
    
    create_test_message(&state.db, list.id, None, None, "Rust topic", "Rust is great").await;
    create_test_message(&state.db, list.id, None, None, "Other", "Something else").await;
    
    let results = archive_svc.search(&list.id.to_string(), "Rust", None).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].subject, Some("Rust topic".to_string()));
}

#[tokio::test]
async fn test_search_from_filter() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("search2.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let archive_svc = ArchiveService::new(state.db.clone());
    
    let mut msg = oak_maillist::models::email_message::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        list_id: Set(list.id),
        message_id: Set("msg-id".to_string()),
        in_reply_to: Set(None),
        references: Set(None),
        from_name: Set(None),
        from_addr: Set("alice@example.com".to_string()),
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
    
    let results = archive_svc.search(&list.id.to_string(), "Hello", Some("alice@example.com")).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_get_thread_messages_excludes_deleted() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("thread.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc.create(&domain.id.to_string(), "l", "l", None, None).await.unwrap();
    let archive_svc = ArchiveService::new(state.db.clone());
    
    let msg = create_test_message(&state.db, list.id, None, None, "T", "B").await;
    let msg_id = msg.id;
    let mut active: oak_maillist::models::email_message::ActiveModel = msg.into();
    active.thread_id = Set(Some(msg_id));
    active.update(&state.db).await.unwrap();
    
    let results = archive_svc.get_thread_messages(&msg_id.to_string()).await.unwrap();
    assert_eq!(results.len(), 1);
    
    let mut del: oak_maillist::models::email_message::ActiveModel = oak_maillist::models::email_message::Entity::find_by_id(msg_id)
        .one(&state.db).await.unwrap().unwrap().into();
    del.is_deleted = Set(true);
    del.update(&state.db).await.unwrap();
    
    let results = archive_svc.get_thread_messages(&msg_id.to_string()).await.unwrap();
    assert_eq!(results.len(), 0);
}
