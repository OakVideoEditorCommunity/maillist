use crate::models::AppState;
use crate::services::list_service::ListService;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Extension, Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::collections::HashMap;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_lists).post(create_list))
        .route("/{id}", get(get_list).put(update_list).delete(delete_list))
        .route("/{id}/archive", post(clear_archive))
        .route("/{id}/stats", get(get_list_stats))
        .route("/{id}/settings", get(get_list_settings).put(update_list_settings))
        .route("/{id}/subscribe", post(subscribe))
        .route("/{id}/unsubscribe", post(unsubscribe))
        .route("/{id}/subscribers", get(list_subscribers).post(add_subscriber))
        .route("/{id}/subscribers/confirm", post(confirm_subscription))
        .route("/{id}/subscribers/{sub_id}", get(get_subscriber).put(update_subscriber).delete(remove_subscriber))
        .route("/{id}/subscribers/import", post(import_subscribers))
        .route("/{id}/subscribers/export", get(export_subscribers))
        .route("/{id}/subscribers/bulk-update", post(bulk_update_subscribers))
        .route("/{id}/messages", get(list_messages))
        .route("/{id}/messages/{msg_id}", get(get_message).delete(delete_message))
        .route("/{id}/messages/{msg_id}/raw", get(download_raw_message))
        .route("/{id}/messages/{msg_id}/attachments", get(list_attachments))
        .route("/{id}/messages/{msg_id}/attachments/{att_id}", get(download_attachment))
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
    let list = service
        .update(&id, req)
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
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn get_list_stats(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
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

async fn subscribe(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn unsubscribe(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn confirm_subscription(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn list_subscribers(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn add_subscriber(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn get_subscriber(
    State(_state): State<AppState>,
    Path((_id, _sub_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn update_subscriber(
    State(_state): State<AppState>,
    Path((_id, _sub_id)): Path<(String, String)>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn remove_subscriber(
    State(_state): State<AppState>,
    Path((_id, _sub_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn import_subscribers(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn export_subscribers(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<String> {
    todo!()
}

async fn bulk_update_subscribers(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn list_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<serde_json::Value> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, PaginatorTrait};

    let list_uuid = uuid::Uuid::parse_str(&id).map_err(|e| ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let page: u64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let per_page: u64 = params.get("per_page").and_then(|v| v.parse().ok()).unwrap_or(20).min(100);

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

    let items: Vec<_> = messages.into_iter().map(|m| {
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
    }).collect();

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
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    use chrono::Utc;

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
    State(_state): State<AppState>,
    Path((_id, _msg_id)): Path<(String, String)>,
) -> ApiResult<String> {
    todo!()
}

async fn list_attachments(
    State(_state): State<AppState>,
    Path((_id, _msg_id)): Path<(String, String)>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn download_attachment(
    State(_state): State<AppState>,
    Path((_id, _msg_id, _att_id)): Path<(String, String, String)>,
) -> ApiResult<Vec<u8>> {
    todo!()
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
    let per_page: u64 = params.get("per_page").and_then(|v| v.parse().ok()).unwrap_or(20).min(100);

    let threads = crate::models::email_message::Entity::find()
        .filter(crate::models::email_message::Column::ListId.eq(list_uuid))
        .filter(crate::models::email_message::Column::IsDeleted.eq(false))
        .filter(crate::models::email_message::Column::ThreadId.is_not_null())
        .order_by_desc(crate::models::email_message::Column::ReceivedAt)
        .paginate(&state.db, per_page);

    let messages: Vec<crate::models::email_message::Model> = threads.fetch_page(page - 1).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let items: Vec<_> = messages.into_iter().map(|m| {
        serde_json::json!({
            "thread_id": m.thread_id,
            "latest_subject": m.subject,
            "latest_from": m.from_addr,
            "latest_received_at": m.received_at,
        })
    }).collect();

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

    let items: Vec<_> = messages.into_iter().map(|m| {
        serde_json::json!({
            "id": m.id,
            "message_id": m.message_id,
            "from_addr": m.from_addr,
            "from_name": m.from_name,
            "subject": m.subject,
            "body_text": m.body_text,
            "received_at": m.received_at,
        })
    }).collect();

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
    let results = service.search(&id, &keyword, from).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let items: Vec<_> = results.into_iter().map(|m| {
        serde_json::json!({
            "id": m.id,
            "message_id": m.message_id,
            "from_addr": m.from_addr,
            "subject": m.subject,
            "received_at": m.received_at,
        })
    }).collect();

    Ok(Json(ApiResponse::new(serde_json::json!({
        "list_id": list_uuid,
        "keyword": keyword,
        "items": items,
    }))))
}

async fn list_moderation_queue(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn list_policies(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn add_policy(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn delete_policy(
    State(_state): State<AppState>,
    Path((_id, _policy_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    todo!()
}
