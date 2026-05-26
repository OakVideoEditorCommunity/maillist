use crate::models::AppState;
use crate::services::list_service::ListService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post, put},
};
use sea_orm::ActiveModelTrait;
use serde::Deserialize;
use std::collections::HashMap;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_lists).post(create_list))
        .route("/{id}", get(get_list).put(update_list).delete(delete_list))
        .route("/{id}/archive", post(clear_archive))
        .route("/{id}/stats", get(get_list_stats))
        .route(
            "/{id}/settings",
            get(get_list_settings).put(update_list_settings),
        )
        .route("/{id}/subscribe", post(subscribe))
        .route("/{id}/unsubscribe", post(unsubscribe))
        .route(
            "/{id}/subscribers",
            get(list_subscribers).post(add_subscriber),
        )
        .route("/{id}/subscribers/confirm", post(confirm_subscription))
        .route(
            "/{id}/subscribers/{sub_id}",
            get(get_subscriber)
                .put(update_subscriber)
                .delete(remove_subscriber),
        )
        .route("/{id}/subscribers/import", post(import_subscribers))
        .route("/{id}/subscribers/export", get(export_subscribers))
        .route(
            "/{id}/subscribers/bulk-update",
            post(bulk_update_subscribers),
        )
        .route("/{id}/messages", get(list_messages))
        .route(
            "/{id}/messages/{msg_id}",
            get(get_message).delete(delete_message),
        )
        .route("/{id}/messages/{msg_id}/raw", get(download_raw_message))
        .route("/{id}/messages/{msg_id}/attachments", get(list_attachments))
        .route(
            "/{id}/messages/{msg_id}/attachments/{att_id}",
            get(download_attachment),
        )
        .route("/{id}/threads", get(list_threads))
        .route("/{id}/threads/{thread_id}", get(get_thread))
        .route("/{id}/search", get(search_archive))
        .route("/{id}/moderation", get(list_moderation_queue))
        .route("/{id}/policies", get(list_policies).post(add_policy))
        .route("/{id}/policies/{policy_id}", delete(delete_policy))
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub q: Option<String>,
    pub visibility: Option<String>,
}

async fn list_lists(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> ApiResult<serde_json::Value> {
    let service = ListService::new(state.db.clone());
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let (lists, total) = service
        .list_public(page, per_page)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let items: Vec<_> = lists
        .into_iter()
        .map(|l| {
            serde_json::json!({
                "id": l.id,
                "name": l.name,
                "display_name": l.display_name,
                "email": format!("{}@...", l.email_local_part),
                "description": l.description,
                "visibility": l.visibility,
                "created_at": l.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::with_meta(
        serde_json::json!({ "items": items }),
        serde_json::json!({
            "page": page,
            "per_page": per_page,
            "total": total,
        }),
    )))
}

#[derive(Deserialize)]
pub struct CreateListRequest {
    pub domain_id: String,
    pub name: String,
    pub email_local_part: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

async fn create_list(
    State(state): State<AppState>,
    Json(req): Json<CreateListRequest>,
) -> ApiResult<serde_json::Value> {
    let service = ListService::new(state.db.clone());
    let list = service
        .create(
            &req.domain_id,
            &req.name,
            &req.email_local_part,
            req.display_name.as_deref(),
            req.description.as_deref(),
        )
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": list.id,
        "name": list.name,
        "display_name": list.display_name,
        "email_local_part": list.email_local_part,
        "created_at": list.created_at,
    }))))
}

async fn get_list(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let service = ListService::new(state.db.clone());
    let list = service
        .find_by_id(&id)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "List not found".to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": list.id,
        "name": list.name,
        "display_name": list.display_name,
        "email_local_part": list.email_local_part,
        "description": list.description,
        "visibility": list.visibility,
        "subscription_policy": list.subscription_policy,
        "post_policy": list.post_policy,
        "reply_to": list.reply_to,
        "archive_enabled": list.archive_enabled,
        "archive_visibility": list.archive_visibility,
        "max_message_size_kb": list.max_message_size_kb,
        "digest_enabled": list.digest_enabled,
        "ai_moderation_enabled": list.ai_moderation_enabled,
        "is_active": list.is_active,
        "created_at": list.created_at,
        "updated_at": list.updated_at,
    }))))
}

async fn update_list(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let service = ListService::new(state.db.clone());
    let list = service.update(&id, req).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": list.id,
        "name": list.name,
        "updated_at": list.updated_at,
    }))))
}

async fn delete_list(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let service = ListService::new(state.db.clone());
    service.delete(&id).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "List deleted successfully"
    }))))
}

async fn clear_archive(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let messages = crate::models::email_message::Entity::find()
        .filter(crate::models::email_message::Column::ListId.eq(list_uuid))
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    for msg in messages {
        let mut active: crate::models::email_message::ActiveModel = msg.into();
        active.is_deleted = Set(true);
        active.deleted_at = Set(Some(chrono::Utc::now().into()));
        active.deleted_reason = Set(Some("Archive cleared".to_string()));
        let _ = active.update(&state.db).await;
    }

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Archive cleared"
    }))))
}

async fn get_list_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let subscriber_count = crate::models::subscriber::Entity::find()
        .filter(crate::models::subscriber::Column::ListId.eq(list_uuid))
        .filter(crate::models::subscriber::Column::Status.eq("active"))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;

    let message_count = crate::models::email_message::Entity::find()
        .filter(crate::models::email_message::Column::ListId.eq(list_uuid))
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;

    let pending_moderation = crate::models::moderation_queue::Entity::find()
        .filter(crate::models::moderation_queue::Column::ListId.eq(list_uuid))
        .filter(crate::models::moderation_queue::Column::Status.eq("pending"))
        .all(&state.db)
        .await
        .unwrap_or_default()
        .len() as u64;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "subscriber_count": subscriber_count,
        "message_count": message_count,
        "pending_moderation": pending_moderation,
    }))))
}

async fn get_list_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    get_list(State(state), Path(id)).await
}

async fn update_list_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    update_list(State(state), Path(id), Json(req)).await
}

#[derive(Deserialize)]
struct SubscribeRequest {
    email: String,
    name: Option<String>,
}

async fn subscribe(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SubscribeRequest>,
) -> ApiResult<serde_json::Value> {
    let svc = crate::services::subscriber_service::SubscriberService::new(state.db.clone());
    let sub = svc
        .subscribe(
            &id,
            &req.email,
            req.name.as_deref(),
            &state.config.server.base_url,
        )
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": sub.id,
        "email": sub.email,
        "status": sub.status,
        "token": sub.token,
    }))))
}

#[derive(Deserialize)]
struct UnsubscribeRequest {
    token: String,
}

async fn unsubscribe(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UnsubscribeRequest>,
) -> ApiResult<serde_json::Value> {
    let svc = crate::services::subscriber_service::SubscriberService::new(state.db.clone());
    svc.unsubscribe(&id, &req.token)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Unsubscribed successfully"
    }))))
}

#[derive(Deserialize)]
struct ConfirmRequest {
    token: String,
}

async fn confirm_subscription(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ConfirmRequest>,
) -> ApiResult<serde_json::Value> {
    let svc = crate::services::subscriber_service::SubscriberService::new(state.db.clone());
    let sub = svc.confirm(&id, &req.token).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": sub.id,
        "email": sub.email,
        "status": sub.status,
    }))))
}

async fn list_subscribers(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::PaginatorTrait;
    let svc = crate::services::subscriber_service::SubscriberService::new(state.db.clone());
    let page: u64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let per_page: u64 = params
        .get("per_page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
        .min(100);

    let (items, total) = svc
        .list_by_list(&id, page, per_page)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let result: Vec<_> = items
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "email": s.email,
                "name": s.name,
                "status": s.status,
                "digest_mode": s.digest_mode,
                "confirmed_at": s.confirmed_at,
                "created_at": s.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::with_meta(
        serde_json::json!({ "items": result }),
        serde_json::json!({ "page": page, "per_page": per_page, "total": total }),
    )))
}

async fn add_subscriber(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let email = req.get("email").and_then(|v| v.as_str()).ok_or(ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "email is required".to_string(),
        details: None,
        request_id: None,
    })?;
    let name = req.get("name").and_then(|v| v.as_str());

    let svc = crate::services::subscriber_service::SubscriberService::new(state.db.clone());
    let sub = svc
        .subscribe(&id, email, name, &state.config.server.base_url)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let mut active: crate::models::subscriber::ActiveModel = sub.into();
    active.status = sea_orm::Set("active".to_string());
    active.confirmed_at = sea_orm::Set(Some(chrono::Utc::now().into()));
    let sub = active.update(&state.db).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": sub.id,
        "email": sub.email,
        "status": sub.status,
    }))))
}

async fn get_subscriber(
    State(state): State<AppState>,
    Path((_id, sub_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::EntityTrait;
    let uuid = uuid::Uuid::parse_str(&sub_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let sub = crate::models::subscriber::Entity::find_by_id(uuid)
        .one(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "Subscriber not found".to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": sub.id,
        "email": sub.email,
        "name": sub.name,
        "status": sub.status,
        "digest_mode": sub.digest_mode,
        "bounce_count": sub.bounce_count,
    }))))
}

async fn update_subscriber(
    State(state): State<AppState>,
    Path((_id, sub_id)): Path<(String, String)>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};
    let uuid = uuid::Uuid::parse_str(&sub_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let sub = crate::models::subscriber::Entity::find_by_id(uuid)
        .one(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "Subscriber not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let mut active: crate::models::subscriber::ActiveModel = sub.into();
    if let Some(v) = req.get("status").and_then(|v| v.as_str()) {
        active.status = Set(v.to_string());
    }
    if let Some(v) = req.get("digest_mode").and_then(|v| v.as_str()) {
        active.digest_mode = Set(v.to_string());
    }
    if let Some(v) = req.get("name").and_then(|v| v.as_str()) {
        active.name = Set(Some(v.to_string()));
    }
    active.updated_at = Set(chrono::Utc::now().into());

    let updated = active.update(&state.db).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": updated.id,
        "status": updated.status,
        "digest_mode": updated.digest_mode,
    }))))
}

async fn remove_subscriber(
    State(state): State<AppState>,
    Path((_id, sub_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::EntityTrait;
    let uuid = uuid::Uuid::parse_str(&sub_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    crate::models::subscriber::Entity::delete_by_id(uuid)
        .exec(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Subscriber removed"
    }))))
}

async fn import_subscribers(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let emails = req
        .get("emails")
        .and_then(|v| v.as_array())
        .ok_or(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "emails array is required".to_string(),
            details: None,
            request_id: None,
        })?;

    let svc = crate::services::subscriber_service::SubscriberService::new(state.db.clone());
    let mut count = 0;
    for e in emails {
        if let Some(email) = e.as_str() {
            if svc
                .subscribe(&id, email, None, &state.config.server.base_url)
                .await
                .is_ok()
            {
                count += 1;
            }
        }
    }

    Ok(Json(ApiResponse::new(serde_json::json!({
        "imported": count,
    }))))
}

async fn export_subscribers(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<String> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let items = crate::models::subscriber::Entity::find()
        .filter(crate::models::subscriber::Column::ListId.eq(list_uuid))
        .order_by_asc(crate::models::subscriber::Column::Email)
        .all(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let mut csv = "email,name,status\n".to_string();
    for s in items {
        csv.push_str(&format!(
            "{},{},{}\n",
            s.email,
            s.name.as_deref().unwrap_or(""),
            s.status
        ));
    }

    Ok(Json(ApiResponse::new(csv)))
}

async fn bulk_update_subscribers(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let status = req.get("status").and_then(|v| v.as_str());
    let ids = req.get("ids").and_then(|v| v.as_array());

    if let Some(ids) = ids {
        for id_val in ids {
            if let Some(sid) = id_val.as_str() {
                if let Ok(uid) = uuid::Uuid::parse_str(sid) {
                    if let Ok(Some(sub)) = crate::models::subscriber::Entity::find_by_id(uid)
                        .one(&state.db)
                        .await
                    {
                        let mut active: crate::models::subscriber::ActiveModel = sub.into();
                        if let Some(s) = status {
                            active.status = Set(s.to_string());
                        }
                        active.updated_at = Set(chrono::Utc::now().into());
                        let _ = active.update(&state.db).await;
                    }
                }
            }
        }
    }

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Bulk update completed"
    }))))
}

async fn list_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let page: u64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let per_page: u64 = params
        .get("per_page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
        .min(100);

    let paginator = crate::models::email_message::Entity::find()
        .filter(crate::models::email_message::Column::ListId.eq(list_uuid))
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .order_by_desc(crate::models::email_message::Column::ReceivedAt)
        .paginate(&state.db, per_page);

    let messages = paginator.fetch_page(page - 1).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let total = paginator.num_items().await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let items: Vec<_> = messages
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "message_id": m.message_id,
                "from_addr": m.from_addr,
                "from_name": m.from_name,
                "subject": m.subject,
                "received_at": m.received_at,
                "has_attachments": m.has_attachments,
                "size_bytes": m.size_bytes,
            })
        })
        .collect();

    Ok(Json(ApiResponse::with_meta(
        serde_json::json!({ "items": items }),
        serde_json::json!({ "page": page, "per_page": per_page, "total": total }),
    )))
}

async fn get_message(
    State(state): State<AppState>,
    Path((_id, msg_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let msg_uuid = uuid::Uuid::parse_str(&msg_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let message = crate::models::email_message::Entity::find_by_id(msg_uuid)
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "Message not found".to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": message.id,
        "message_id": message.message_id,
        "in_reply_to": message.in_reply_to,
        "from_name": message.from_name,
        "from_addr": message.from_addr,
        "subject": message.subject,
        "body_text": message.body_text,
        "body_html": message.body_html,
        "received_at": message.received_at,
        "has_attachments": message.has_attachments,
        "is_deleted": message.is_deleted,
    }))))
}

async fn delete_message(
    Extension(claims): Extension<crate::api::middleware::auth::Claims>,
    State(state): State<AppState>,
    Path((_list_id, msg_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

    let msg_uuid = uuid::Uuid::parse_str(&msg_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let user_uuid = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let message = crate::models::email_message::Entity::find_by_id(msg_uuid)
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "Message not found or already deleted".to_string(),
            details: None,
            request_id: None,
        })?;

    let mut active: crate::models::email_message::ActiveModel = message.into();
    active.is_deleted = Set(true);
    active.deleted_at = Set(Some(Utc::now().into()));
    active.deleted_by = Set(Some(user_uuid));
    active.deleted_reason = Set(Some("Administrative deletion".to_string()));
    active.update(&state.db).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Message soft-deleted successfully",
        "deleted_at": Utc::now().to_rfc3339(),
    }))))
}

async fn download_raw_message(
    State(state): State<AppState>,
    Path((_id, msg_id)): Path<(String, String)>,
) -> ApiResult<String> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let msg_uuid = uuid::Uuid::parse_str(&msg_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let message = crate::models::email_message::Entity::find_by_id(msg_uuid)
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "Message not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let raw = message.raw_content.unwrap_or_default();
    Ok(Json(ApiResponse::new(raw)))
}

async fn list_attachments(
    State(state): State<AppState>,
    Path((_id, msg_id)): Path<(String, String)>,
) -> ApiResult<Vec<serde_json::Value>> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let msg_uuid = uuid::Uuid::parse_str(&msg_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let items = crate::models::attachment::Entity::find()
        .filter(crate::models::attachment::Column::MessageId.eq(msg_uuid))
        .all(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let result: Vec<_> = items
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "filename": a.filename,
                "content_type": a.content_type,
                "size_bytes": a.size_bytes,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}

async fn download_attachment(
    State(state): State<AppState>,
    Path((_id, _msg_id, att_id)): Path<(String, String, String)>,
) -> ApiResult<Vec<u8>> {
    use sea_orm::EntityTrait;
    let att_uuid = uuid::Uuid::parse_str(&att_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let att = crate::models::attachment::Entity::find_by_id(att_uuid)
        .one(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?
        .ok_or(ApiError {
            code: "NOT_FOUND".to_string(),
            message: "Attachment not found".to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(
        att.storage_path.unwrap_or_default().into_bytes(),
    )))
}

async fn list_threads(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};

    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let page: u64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let per_page: u64 = params
        .get("per_page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
        .min(100);

    let threads = crate::models::email_message::Entity::find()
        .filter(crate::models::email_message::Column::ListId.eq(list_uuid))
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .filter(crate::models::email_message::Column::ThreadId.is_not_null())
        .order_by_desc(crate::models::email_message::Column::ReceivedAt)
        .paginate(&state.db, per_page);

    let messages: Vec<crate::models::email_message::Model> =
        threads.fetch_page(page - 1).await.map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let items: Vec<_> = messages
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "thread_id": m.thread_id,
                "latest_subject": m.subject,
                "latest_from": m.from_addr,
                "latest_received_at": m.received_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::with_meta(
        serde_json::json!({ "items": items }),
        serde_json::json!({ "page": page, "per_page": per_page }),
    )))
}

async fn get_thread(
    State(state): State<AppState>,
    Path((_id, thread_id)): Path<(String, String)>,
) -> ApiResult<Vec<serde_json::Value>> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    let thread_uuid = uuid::Uuid::parse_str(&thread_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let messages = crate::models::email_message::Entity::find()
        .filter(crate::models::email_message::Column::ThreadId.eq(Some(thread_uuid)))
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .order_by_asc(crate::models::email_message::Column::ReceivedAt)
        .all(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let items: Vec<_> = messages
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "message_id": m.message_id,
                "from_addr": m.from_addr,
                "from_name": m.from_name,
                "subject": m.subject,
                "body_text": m.body_text,
                "received_at": m.received_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(items)))
}

async fn search_archive(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<serde_json::Value> {
    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let keyword = params.get("q").cloned().unwrap_or_default();
    let from = params.get("from").map(|s| s.as_str());

    let service = crate::services::archive_service::ArchiveService::new(state.db.clone());
    let results = service
        .search(&id, &keyword, from)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let items: Vec<_> = results
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "message_id": m.message_id,
                "from_addr": m.from_addr,
                "subject": m.subject,
                "received_at": m.received_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(serde_json::json!({
        "list_id": list_uuid,
        "keyword": keyword,
        "items": items,
    }))))
}

async fn list_moderation_queue(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let page: u64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let per_page: u64 = params
        .get("per_page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
        .min(100);

    let paginator = crate::models::moderation_queue::Entity::find()
        .filter(crate::models::moderation_queue::Column::ListId.eq(list_uuid))
        .order_by_desc(crate::models::moderation_queue::Column::CreatedAt)
        .paginate(&state.db, per_page);

    let items = paginator.fetch_page(page - 1).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let result: Vec<_> = items
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "from_addr": m.from_addr,
                "subject": m.subject,
                "reason": m.reason,
                "status": m.status,
                "ai_risk_score": m.ai_risk_score,
                "created_at": m.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}

async fn list_policies(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _params: Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let items = crate::models::sender_policy::Entity::find()
        .filter(crate::models::sender_policy::Column::ListId.eq(Some(list_uuid)))
        .all(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    let result: Vec<_> = items
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "email_pattern": p.email_pattern,
                "policy_type": p.policy_type,
                "scope": p.scope,
                "note": p.note,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}

async fn add_policy(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::Set;
    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let email_pattern = req
        .get("email_pattern")
        .and_then(|v| v.as_str())
        .ok_or(ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "email_pattern is required".to_string(),
            details: None,
            request_id: None,
        })?;

    let policy_type = req
        .get("policy_type")
        .and_then(|v| v.as_str())
        .unwrap_or("blacklist");
    let scope = req.get("scope").and_then(|v| v.as_str()).unwrap_or("post");

    let policy = crate::models::sender_policy::ActiveModel {
        id: Set(crate::utils::crypto::generate_uuid()),
        list_id: Set(Some(list_uuid)),
        email_pattern: Set(email_pattern.to_string()),
        policy_type: Set(policy_type.to_string()),
        scope: Set(scope.to_string()),
        note: Set(req
            .get("note")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())),
        created_by: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    };

    let model = policy.insert(&state.db).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": model.id,
        "email_pattern": model.email_pattern,
    }))))
}

async fn delete_policy(
    State(state): State<AppState>,
    Path((_id, policy_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::EntityTrait;
    let pol_uuid = uuid::Uuid::parse_str(&policy_id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    crate::models::sender_policy::Entity::delete_by_id(pol_uuid)
        .exec(&state.db)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: e.to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Policy deleted"
    }))))
}
