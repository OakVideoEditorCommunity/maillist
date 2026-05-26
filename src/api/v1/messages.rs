use crate::models::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new()
}
