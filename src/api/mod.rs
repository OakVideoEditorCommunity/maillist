use crate::models::AppState;
use axum::{
    Router, body::Body, extract::State, http::StatusCode, middleware::from_fn_with_state,
    response::IntoResponse, routing::get,
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

mod middleware;
pub mod v1;

#[derive(rust_embed::RustEmbed)]
#[folder = "frontend/dist/"]
struct Assets;

pub fn create_router(state: AppState) -> Router {
    let auth_routes = v1::auth::routes();

    let protected_routes = Router::new()
        .nest("/users", v1::users::routes())
        .nest("/lists", v1::lists::routes())
        .nest("/subscribers", v1::subscribers::routes())
        .nest("/messages", v1::messages::routes())
        .nest("/moderation", v1::moderation::routes())
        .nest("/domains", v1::domains::routes())
        .nest("/templates", v1::templates::routes())
        .nest("/admin", v1::admin::routes())
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware::auth::require_auth,
        ));

    let public_routes = Router::new()
        .nest("/auth", auth_routes)
        .nest("/setup", v1::setup::routes())
        .route("/health", get(v1::health::health_check))
        .route("/health/ready", get(v1::health::readiness_check))
        .route("/health/live", get(v1::health::liveness_check))
        .route("/metrics", get(v1::health::metrics_handler))
        .route("/config", get(public_config));

    let api_routes = protected_routes.merge(public_routes);

    Router::new()
        .nest("/api/v1", api_routes)
        .fallback(get(static_handler))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::error::error_handler,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn public_config(State(state): axum::extract::State<AppState>) -> impl IntoResponse {
    let branding = &state.config.branding;
    axum::Json(serde_json::json!({
        "site_name": branding.site_name,
        "primary_color": branding.primary_color,
        "logo_url": branding.logo_url,
    }))
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.to_string())],
                Body::from(content.data.into_owned()),
            )
                .into_response()
        }
        None => match Assets::get("index.html") {
            Some(content) => (
                [(axum::http::header::CONTENT_TYPE, "text/html".to_string())],
                Body::from(content.data.into_owned()),
            )
                .into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
    }
}
