mod common;
use common::setup_db;
use oak_maillist::services::domain_service::DomainService;
use oak_maillist::services::list_service::ListService;
use oak_maillist::services::subscriber_service::SubscriberService;

#[tokio::test]
async fn test_subscribe_already_active() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("sub.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc
        .create(&domain.id.to_string(), "l", "l", None, None)
        .await
        .unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());

    let sub = sub_svc
        .subscribe(
            &list.id.to_string(),
            "user@example.com",
            Some("User"),
            "http://localhost",
        )
        .await
        .unwrap();
    sub_svc
        .confirm(&list.id.to_string(), &sub.token)
        .await
        .unwrap();

    let result = sub_svc
        .subscribe(
            &list.id.to_string(),
            "user@example.com",
            Some("User"),
            "http://localhost",
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_subscribe_existing_pending_returns_existing() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("sub2.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc
        .create(&domain.id.to_string(), "l", "l", None, None)
        .await
        .unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());

    let sub1 = sub_svc
        .subscribe(
            &list.id.to_string(),
            "user@example.com",
            Some("User"),
            "http://localhost",
        )
        .await
        .unwrap();
    let sub2 = sub_svc
        .subscribe(
            &list.id.to_string(),
            "user@example.com",
            Some("User"),
            "http://localhost",
        )
        .await
        .unwrap();
    assert_eq!(sub1.id, sub2.id);
}

#[tokio::test]
async fn test_subscribe_invalid_list() {
    let state = setup_db().await;
    let sub_svc = SubscriberService::new(state.db.clone());
    let result = sub_svc
        .subscribe(
            "550e8400-e29b-41d4-a716-446655440000",
            "user@example.com",
            None,
            "http://localhost",
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_confirm_invalid_token() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("conf.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc
        .create(&domain.id.to_string(), "l", "l", None, None)
        .await
        .unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());

    let result = sub_svc.confirm(&list.id.to_string(), "bad-token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_confirm_already_active() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("conf2.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc
        .create(&domain.id.to_string(), "l", "l", None, None)
        .await
        .unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());

    let sub = sub_svc
        .subscribe(
            &list.id.to_string(),
            "user@example.com",
            None,
            "http://localhost",
        )
        .await
        .unwrap();
    let confirmed1 = sub_svc
        .confirm(&list.id.to_string(), &sub.token)
        .await
        .unwrap();
    let confirmed2 = sub_svc
        .confirm(&list.id.to_string(), &sub.token)
        .await
        .unwrap();
    assert_eq!(confirmed1.id, confirmed2.id);
}

#[tokio::test]
async fn test_unsubscribe_invalid_token() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("unsub.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc
        .create(&domain.id.to_string(), "l", "l", None, None)
        .await
        .unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());

    let result = sub_svc.unsubscribe(&list.id.to_string(), "bad-token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_by_list_pagination() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("pag.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc
        .create(&domain.id.to_string(), "l", "l", None, None)
        .await
        .unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());

    for i in 0..5 {
        let sub = sub_svc
            .subscribe(
                &list.id.to_string(),
                &format!("user{}@example.com", i),
                None,
                "http://localhost",
            )
            .await
            .unwrap();
        sub_svc
            .confirm(&list.id.to_string(), &sub.token)
            .await
            .unwrap();
    }

    let (items, total) = sub_svc
        .list_by_list(&list.id.to_string(), 1, 10)
        .await
        .unwrap();
    assert_eq!(items.len(), 5);
    assert_eq!(total, 5);
}

#[tokio::test]
async fn test_update_digest_mode() {
    let state = setup_db().await;
    let domain_svc = DomainService::new(state.db.clone());
    let domain = domain_svc.create("dig.com").await.unwrap();
    let list_svc = ListService::new(state.db.clone());
    let list = list_svc
        .create(&domain.id.to_string(), "l", "l", None, None)
        .await
        .unwrap();
    let sub_svc = SubscriberService::new(state.db.clone());

    let sub = sub_svc
        .subscribe(
            &list.id.to_string(),
            "user@example.com",
            None,
            "http://localhost",
        )
        .await
        .unwrap();
    let updated = sub_svc
        .update_digest_mode(&sub.id.to_string(), "daily")
        .await
        .unwrap();
    assert_eq!(updated.digest_mode, "daily");
}

#[tokio::test]
async fn test_update_digest_mode_not_found() {
    let state = setup_db().await;
    let sub_svc = SubscriberService::new(state.db.clone());
    let result = sub_svc
        .update_digest_mode("550e8400-e29b-41d4-a716-446655440000", "daily")
        .await;
    assert!(result.is_err());
}
