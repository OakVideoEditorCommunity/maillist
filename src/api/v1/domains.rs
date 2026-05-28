use crate::models::AppState;
use crate::services::{
    dkim_service::DkimService, dns_check_service::DnsCheckService, domain_service::DomainService,
};
use crate::utils::response::{ApiError, ApiResponse, ApiResult};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_domains).post(create_domain))
        .route(
            "/{id}",
            get(get_domain).put(update_domain).delete(delete_domain),
        )
        .route("/{id}/generate-dkim", post(generate_dkim))
        .route("/{id}/dns-records", get(get_dns_records))
        .route("/{id}/verify-dns", post(verify_dns))
        .route("/{id}/test-smtp", post(test_smtp))
        .route("/{id}/verify-dkim", post(verify_dkim_stub))
}

async fn list_domains(State(state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    let svc = DomainService::new(state.db.clone());
    let items = svc.list().await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    let result: Vec<_> = items
        .into_iter()
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "name": d.name,
                "spf_verified": d.spf_verified,
                "dkim_verified": d.dkim_verified,
                "dmarc_verified": d.dmarc_verified,
                "dkim_enabled": d.dkim_enabled,
                "created_at": d.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::new(result)))
}

async fn create_domain(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let name = req.get("name").and_then(|v| v.as_str()).ok_or(ApiError {
        code: "VALIDATION_ERROR".to_string(),
        message: "name is required".to_string(),
        details: None,
        request_id: None,
    })?;

    let svc = DomainService::new(state.db.clone());
    let domain = svc.create(name).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": domain.id,
        "name": domain.name,
    }))))
}

async fn get_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    let domain = svc
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
            message: "Domain not found".to_string(),
            details: None,
            request_id: None,
        })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": domain.id,
        "name": domain.name,
        "smtp_host": domain.smtp_host,
        "smtp_port": domain.smtp_port,
        "smtp_username": domain.smtp_username,
        "dkim_selector": domain.dkim_selector,
        "dkim_public_key": domain.dkim_public_key,
        "spf_record": domain.spf_record,
        "dmarc_record": domain.dmarc_record,
        "spf_verified": domain.spf_verified,
        "dkim_verified": domain.dkim_verified,
        "dmarc_verified": domain.dmarc_verified,
        "dkim_enabled": domain.dkim_enabled,
        "created_at": domain.created_at,
    }))))
}

async fn update_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    let domain = svc.update(&id, req).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "id": domain.id,
        "name": domain.name,
    }))))
}

async fn delete_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    svc.delete(&id).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "Domain deleted"
    }))))
}

async fn verify_dkim_stub(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    Ok(Json(ApiResponse::new(serde_json::json!({
        "message": "DKIM verification not yet implemented"
    }))))
}

async fn generate_dkim(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    let domain = svc
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
            message: "Domain not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let keypair = DkimService::generate_keypair(&domain.name).map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: format!("Failed to generate DKIM keypair: {}", e),
        details: None,
        request_id: None,
    })?;

    let dns_record = DkimService::build_dns_record(&keypair.public_key_base64);

    let updates = serde_json::json!({
        "dkim_selector": keypair.selector,
        "dkim_private_key": keypair.private_key_pem,
        "dkim_public_key": keypair.public_key_base64,
        "dkim_enabled": true,
    });
    svc.update(&id, updates).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "selector": keypair.selector,
        "dns_record": dns_record,
        "public_key": keypair.public_key_base64,
    }))))
}

async fn get_dns_records(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    let domain = svc
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
            message: "Domain not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let server_ip = state.config.server.host.clone();
    let spf = domain
        .spf_record
        .clone()
        .unwrap_or_else(|| DkimService::build_spf_record(&server_ip));
    let dmarc = domain
        .dmarc_record
        .clone()
        .unwrap_or_else(|| DkimService::build_dmarc_record(&domain.name));

    let dkim = domain.dkim_selector.as_ref().and_then(|sel| {
        domain.dkim_public_key.as_ref().map(|pk| {
            serde_json::json!({
                "selector": sel,
                "dns_record": DkimService::build_dns_record(pk),
            })
        })
    });

    Ok(Json(ApiResponse::new(serde_json::json!({
        "spf": spf,
        "dmarc": dmarc,
        "dkim": dkim,
    }))))
}

async fn verify_dns(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    let domain = svc
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
            message: "Domain not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let result = DnsCheckService::verify_all(
        &domain.name,
        domain.dkim_selector.as_deref(),
        domain.dkim_public_key.as_deref(),
    )
    .await
    .map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: format!("DNS verification failed: {}", e),
        details: None,
        request_id: None,
    })?;

    let updates = serde_json::json!({
        "spf_verified": result.spf.valid,
        "dkim_verified": result.dkim.valid,
        "dmarc_verified": result.dmarc.valid,
    });
    svc.update(&id, updates).await.map_err(|e| ApiError {
        code: "INTERNAL_ERROR".to_string(),
        message: e.to_string(),
        details: None,
        request_id: None,
    })?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "spf": result.spf,
        "dkim": result.dkim,
        "dmarc": result.dmarc,
    }))))
}

async fn test_smtp(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let svc = DomainService::new(state.db.clone());
    let domain = svc
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
            message: "Domain not found".to_string(),
            details: None,
            request_id: None,
        })?;

    let smtp_host = domain
        .smtp_host
        .as_deref()
        .filter(|h| !h.is_empty())
        .unwrap_or(&state.config.smtp.outgoing.host);
    let smtp_port = domain
        .smtp_port
        .map(|p| p as u16)
        .unwrap_or(state.config.smtp.outgoing.port);
    let smtp_username = domain
        .smtp_username
        .as_deref()
        .filter(|u| !u.is_empty())
        .unwrap_or(&state.config.smtp.outgoing.username);
    let smtp_password = domain
        .smtp_password
        .as_deref()
        .filter(|p| !p.is_empty())
        .unwrap_or(&state.config.smtp.outgoing.password);

    if smtp_host.is_empty() {
        return Err(ApiError {
            code: "CONFIG_ERROR".to_string(),
            message: "SMTP host not configured".to_string(),
            details: None,
            request_id: None,
        });
    }

    use lettre::Transport;
    use lettre::message::{Mailbox, Message};

    let from = state
        .config
        .smtp
        .outgoing
        .from_address
        .parse::<Mailbox>()
        .unwrap_or_else(|_| "noreply@oak-maillist".parse().unwrap());

    let email = Message::builder()
        .from(from.clone())
        .to(from)
        .subject("SMTP Test from Oak MailList")
        .body("This is a test email to verify your SMTP relay configuration.".to_string())
        .map_err(|e| ApiError {
            code: "SMTP_ERROR".to_string(),
            message: format!("Failed to build test email: {}", e),
            details: None,
            request_id: None,
        })?;

    let creds = lettre::transport::smtp::authentication::Credentials::new(
        smtp_username.to_string(),
        smtp_password.to_string(),
    );

    let mailer = lettre::SmtpTransport::relay(smtp_host)
        .map_err(|e| ApiError {
            code: "SMTP_ERROR".to_string(),
            message: format!("Invalid SMTP host: {}", e),
            details: None,
            request_id: None,
        })?
        .port(smtp_port)
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(Json(ApiResponse::new(serde_json::json!({
            "success": true,
            "message": "Test email sent successfully",
        })))),
        Err(e) => Err(ApiError {
            code: "SMTP_ERROR".to_string(),
            message: format!("Failed to send test email: {}", e),
            details: None,
            request_id: None,
        }),
    }
}
