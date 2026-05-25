use crate::models::AppState;
use axum::{
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

mod middleware;
pub mod v1;

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .nest("/auth", v1::auth::routes())
        .nest("/users", v1::users::routes())
        .nest("/lists", v1::lists::routes())
        .nest("/subscribers", v1::subscribers::routes())
        .nest("/messages", v1::messages::routes())
        .nest("/moderation", v1::moderation::routes())
        .nest("/domains", v1::domains::routes())
        .nest("/templates", v1::templates::routes())
        .nest("/admin", v1::admin::routes())
        .route("/health", get(v1::health::health_check))
        .route("/health/ready", get(v1::health::readiness_check))
        .route("/health/live", get(v1::health::liveness_check))
        .route("/metrics", get(v1::health::metrics_handler));

    Router::new()
        .nest("/api/v1", api_routes)
        .layer(from_fn(middleware::error::error_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
