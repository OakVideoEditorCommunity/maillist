use crate::models::AppState;
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    extract::{Path, Query, State},
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
    State(_state): State<AppState>,
    Query(_params): Query<ListQuery>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn create_list(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn get_list(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn update_list(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn delete_list(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
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
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn update_list_settings(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    todo!()
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
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn get_message(
    State(_state): State<AppState>,
    Path((_id, _msg_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    todo!()
}

async fn delete_message(
    State(_state): State<AppState>,
    Path((_id, _msg_id)): Path<(String, String)>,
) -> ApiResult<serde_json::Value> {
    todo!()
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
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn get_thread(
    State(_state): State<AppState>,
    Path((_id, _thread_id)): Path<(String, String)>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
}

async fn search_archive(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Vec<serde_json::Value>> {
    todo!()
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
