use crate::models::AppState;
use crate::services::{
    dkim_service::DkimService,
    dns_check_service::{self, DnsCheckService},
    domain_service::DomainService,
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
        .route("/{id}/dmarc-report", get(get_dmarc_report))
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

    let uses_relay = domain.smtp_host.as_ref().is_some_and(|h| !h.is_empty());

    let mut resp = serde_json::json!({
        "id": domain.id,
        "name": domain.name,
        "smtp_host": domain.smtp_host,
        "smtp_port": domain.smtp_port,
        "smtp_username": domain.smtp_username,
        "dmarc_record": domain.dmarc_record,
        "dmarc_verified": domain.dmarc_verified,
        "created_at": domain.created_at,
    });

    if uses_relay {
        resp["spf_managed_by_relay"] = serde_json::Value::Bool(true);
        resp["dkim_managed_by_relay"] = serde_json::Value::Bool(true);
    } else {
        resp["dkim_selector"] = domain.dkim_selector.into();
        resp["dkim_public_key"] = domain.dkim_public_key.into();
        resp["spf_record"] = domain.spf_record.into();
        resp["spf_verified"] = serde_json::Value::Bool(domain.spf_verified);
        resp["dkim_verified"] = serde_json::Value::Bool(domain.dkim_verified);
        resp["dkim_enabled"] = serde_json::Value::Bool(domain.dkim_enabled);
    }

    Ok(Json(ApiResponse::new(resp)))
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

async fn get_dmarc_report(
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

    let dmarc_result = DnsCheckService::verify_dmarc(&domain.name)
        .await
        .map_err(|e| ApiError {
            code: "INTERNAL_ERROR".to_string(),
            message: format!("DMARC lookup failed: {}", e),
            details: None,
            request_id: None,
        })?;

    let report = if let Some(parsed) = &dmarc_result.parsed {
        serde_json::json!({
            "domain": domain.name,
            "dmarc_found": dmarc_result.found,
            "dmarc_valid": dmarc_result.valid,
            "raw_record": dmarc_result.record,
            "policy": parsed.policy,
            "subdomain_policy": parsed.subdomain_policy,
            "percentage": parsed.percentage,
            "aggregate_report_uris": parsed.report_uris_aggregate,
            "forensic_report_uris": parsed.report_uris_forensic,
            "dkim_alignment": parsed.dkim_alignment,
            "spf_alignment": parsed.spf_alignment,
            "failure_options": parsed.failure_options,
            "report_interval_seconds": parsed.report_interval,
            "report_format": parsed.report_format,
            "recommendations": generate_dmarc_recommendations(parsed),
        })
    } else {
        serde_json::json!({
            "domain": domain.name,
            "dmarc_found": dmarc_result.found,
            "dmarc_valid": dmarc_result.valid,
            "raw_record": dmarc_result.record,
            "message": dmarc_result.message,
        })
    };

    Ok(Json(ApiResponse::new(report)))
}

fn generate_dmarc_recommendations(parsed: &dns_check_service::DmarcParsedRecord) -> Vec<String> {
    let mut recs = Vec::new();

    match parsed.policy.as_str() {
        "none" => {
            recs.push("Current policy is 'none' — emails failing DMARC will not be rejected. Consider moving to 'quarantine' after monitoring.".to_string());
        }
        "quarantine" => {
            recs.push("Policy is 'quarantine' — failing emails may be marked as spam. Monitor aggregate reports before moving to 'reject'.".to_string());
        }
        "reject" => {
            recs.push("Policy is 'reject' — failing emails will be rejected. Ensure SPF and DKIM are properly configured.".to_string());
        }
        _ => {}
    }

    if parsed.percentage.is_some_and(|p| p < 100) {
        recs.push(format!(
            "Only {}% of emails are subject to DMARC policy. Consider increasing to 100%% after validation.",
            parsed.percentage.unwrap()
        ));
    }

    if parsed.report_uris_aggregate.is_empty() {
        recs.push("No aggregate report URI (rua) configured. Add an address to receive DMARC aggregate reports.".to_string());
    }

    if parsed.dkim_alignment.as_deref() != Some("s") {
        recs.push("DKIM alignment is not strict (adkim=s). Consider strict alignment for stronger security.".to_string());
    }

    if parsed.spf_alignment.as_deref() != Some("s") {
        recs.push("SPF alignment is not strict (aspf=s). Consider strict alignment for stronger security.".to_string());
    }

    recs
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

    let uses_relay = domain.smtp_host.as_ref().is_some_and(|h| !h.is_empty());
    if uses_relay {
        return Err(ApiError {
            code: "CONFIG_ERROR".to_string(),
            message: "DKIM is managed by the SMTP relay provider when using relay mode".to_string(),
            details: None,
            request_id: None,
        });
    }

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

    let uses_relay = domain.smtp_host.as_ref().is_some_and(|h| !h.is_empty());

    let dmarc = domain
        .dmarc_record
        .clone()
        .unwrap_or_else(|| DkimService::build_dmarc_record(&domain.name));

    let mut resp = serde_json::json!({
        "dmarc": dmarc,
    });

    if uses_relay {
        resp["spf_managed_by_relay"] = serde_json::Value::Bool(true);
        resp["dkim_managed_by_relay"] = serde_json::Value::Bool(true);
    } else {
        let server_ip = state.config.server.host.clone();
        let spf = domain
            .spf_record
            .clone()
            .unwrap_or_else(|| DkimService::build_spf_record(&server_ip));
        resp["spf"] = serde_json::Value::String(spf);

        let dkim = domain.dkim_selector.as_ref().and_then(|sel| {
            domain.dkim_public_key.as_ref().map(|pk| {
                serde_json::json!({
                    "selector": sel,
                    "dns_record": DkimService::build_dns_record(pk),
                })
            })
        });
        resp["dkim"] = dkim.into();
    }

    Ok(Json(ApiResponse::new(resp)))
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

    let uses_relay = domain.smtp_host.as_ref().is_some_and(|h| !h.is_empty());

    let mut result = DnsCheckService::verify_all(
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

    if uses_relay {
        result.spf = dns_check_service::SpfCheckResult {
            found: true,
            valid: true,
            record: None,
            message: "SPF managed by SMTP relay provider".to_string(),
        };
        result.dkim = dns_check_service::DkimCheckResult {
            found: true,
            valid: true,
            record: None,
            message: "DKIM managed by SMTP relay provider".to_string(),
            parsed: None,
        };
    }

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

    let mut resp = serde_json::json!({
        "dmarc": result.dmarc,
    });

    if uses_relay {
        resp["spf_managed_by_relay"] = serde_json::Value::Bool(true);
        resp["dkim_managed_by_relay"] = serde_json::Value::Bool(true);
    } else {
        resp["spf"] = serde_json::to_value(&result.spf).unwrap_or_default();
        resp["dkim"] = serde_json::to_value(&result.dkim).unwrap_or_default();
    }

    Ok(Json(ApiResponse::new(resp)))
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
