use axum::{extract::Request, middleware::Next, response::Response};
use tracing::error;

pub async fn error_handler(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    if response.status().is_server_error() {
        error!("Server error occurred: {}", response.status());
    }

    response
}
