use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use crate::models::AppState;
use crate::services::auth_service::AuthService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let service = AuthService::new(state.db.clone(), state.config.clone());
    let claims = match service.verify_access_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

pub async fn require_admin(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request.extensions().get::<Claims>().cloned();

    match claims {
        Some(claims) if claims.role == "site_admin" => Ok(next.run(request).await),
        _ => Err(StatusCode::FORBIDDEN),
    }
}

pub async fn optional_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        let service = AuthService::new(state.db.clone(), state.config.clone());
        if let Ok(claims) = service.verify_access_token(token) {
            request.extensions_mut().insert(claims);
        }
    }

    next.run(request).await
}


