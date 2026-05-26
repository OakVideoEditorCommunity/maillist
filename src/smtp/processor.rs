use crate::config::AppConfig;
use crate::models::{AppState, email_message, moderation_queue, subscriber};
use crate::services::ai_service::AiService;
use crate::smtp::server::IncomingEmail;
use crate::utils::email::extract_local_part;
use chrono::Utc;
use mailparse::ParsedMail;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{error, info, warn};

pub struct MailPipeline {
    _state: AppState,
    db: DatabaseConnection,
    config: AppConfig,
}

impl MailPipeline {
    pub fn new(state: AppState) -> Self {
        let db = state.db.clone();
        let config = state.config.clone();
        Self {
            _state: state,
            db,
            config,
        }
    }

    pub async fn process(
        &self,
        email: IncomingEmail,
        parsed: ParsedMail<'_>,
    ) -> anyhow::Result<()> {
        let from_addr = get_header_value(&parsed, "From").unwrap_or_else(|| email.from.clone());

        let subject = get_header_value(&parsed, "Subject").unwrap_or_default();

        let to_addr = email.to.first().cloned().unwrap_or_default();

        info!(
            "Mail pipeline: from={}, to={}, subject={}",
            from_addr, to_addr, subject
        );

        let list = self.resolve_list(&to_addr).await?;
        let list = match list {
            Some(l) => l,
            None => {
                warn!("No list found for address: {}", to_addr);
                return Ok(());
            }
        };

        if !list.is_active {
            warn!("List {} is inactive, rejecting email", list.name);
            return Ok(());
        }

        let is_subscriber = self.check_subscriber(&list.id, &from_addr).await?;

        if list.post_policy == "subscriber_only" && !is_subscriber {
            warn!(
                "Rejecting email from non-subscriber {} to list {}",
                from_addr, list.name
            );
            return Ok(());
        }

        let body_text = get_body_text(&parsed);
        let body_html = get_body_html(&parsed);
        let raw_content = String::from_utf8_lossy(&email.raw_data).to_string();
        let size_bytes = email.raw_data.len() as i32;

        let in_reply_to = get_header_value(&parsed, "In-Reply-To");
        let references = get_header_value(&parsed, "References");
        let message_id = get_header_value(&parsed, "Message-Id")
            .unwrap_or_else(|| format!("<{}@oak-maillist>", uuid::Uuid::new_v4()));

        let msg = email_message::ActiveModel {
            id: Set(crate::utils::crypto::generate_uuid()),
            list_id: Set(list.id),
            message_id: Set(message_id.clone()),
            in_reply_to: Set(in_reply_to.clone()),
            references: Set(references.clone()),
            from_name: Set(None),
            from_addr: Set(from_addr.clone()),
            to_addr: Set(Some(to_addr.clone())),
            subject: Set(Some(subject.clone())),
            subject_normalized: Set(Some(normalize_subject(&subject))),
            body_text: Set(Some(body_text.clone())),
            body_html: Set(body_html.clone()),
            raw_content: Set(Some(raw_content)),
            size_bytes: Set(Some(size_bytes)),
            has_attachments: Set(false),
            received_at: Set(Utc::now().into()),
            thread_id: Set(None),
            is_deleted: Set(false),
            deleted_at: Set(None),
            deleted_by: Set(None),
            deleted_reason: Set(None),
        };

        let ai_result = if list.ai_moderation_enabled {
            let ai_service = AiService::new(self.config.ai_moderation.clone());
            match ai_service.moderate_email(&subject, &body_text).await {
                Ok(result) => {
                    info!(
                        "AI moderation result: score={}, verdict={}",
                        result.overall_score, result.verdict
                    );
                    Some(result)
                }
                Err(e) => {
                    error!("AI moderation failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        if let Some(ref ai) = ai_result {
            if ai.verdict == "flagged" {
                warn!("Email flagged by AI, entering moderation queue");

                let mod_queue = moderation_queue::ActiveModel {
                    id: Set(crate::utils::crypto::generate_uuid()),
                    list_id: Set(list.id),
                    message_id: Set(None),
                    from_addr: Set(from_addr),
                    subject: Set(Some(subject)),
                    reason: Set("ai_flagged".to_string()),
                    status: Set("pending".to_string()),
                    source: Set("ai_flagged".to_string()),
                    ai_risk_score: Set(Some(ai.overall_score)),
                    ai_labels: Set(Some(ai.categories.clone())),
                    ai_raw_response: Set(ai.llm_raw_output.clone()),
                    ai_reviewed: Set(false),
                    moderated_by: Set(None),
                    moderated_at: Set(None),
                    moderation_note: Set(None),
                    created_at: Set(Utc::now().into()),
                };
                mod_queue.insert(&self.db).await?;
                return Ok(());
            }
        }

        let saved = msg.insert(&self.db).await?;
        info!("Email saved to archive: {}", saved.id);

        if list.archive_enabled {
            self.deliver_to_subscribers(&list.id, &saved).await?;
        }

        Ok(())
    }

    async fn resolve_list(
        &self,
        to_addr: &str,
    ) -> anyhow::Result<Option<crate::models::mailing_list::Model>> {
        let local_part = extract_local_part(to_addr)
            .ok_or_else(|| anyhow::anyhow!("Invalid email address: {}", to_addr))?;

        use crate::models::mailing_list;
        let list = mailing_list::Entity::find()
            .filter(mailing_list::Column::EmailLocalPart.eq(local_part))
            .filter(mailing_list::Column::IsActive.eq(true))
            .one(&self.db)
            .await?;

        Ok(list)
    }

    async fn check_subscriber(&self, list_id: &uuid::Uuid, email: &str) -> anyhow::Result<bool> {
        use sea_orm::PaginatorTrait;
        let count = subscriber::Entity::find()
            .filter(subscriber::Column::ListId.eq(*list_id))
            .filter(subscriber::Column::Email.eq(email.to_lowercase()))
            .filter(subscriber::Column::Status.eq("active"))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    async fn deliver_to_subscribers(
        &self,
        list_id: &uuid::Uuid,
        message: &email_message::Model,
    ) -> anyhow::Result<()> {
        let subscribers = subscriber::Entity::find()
            .filter(subscriber::Column::ListId.eq(*list_id))
            .filter(subscriber::Column::Status.eq("active"))
            .all(&self.db)
            .await?;

        info!(
            "Delivering message {} to {} subscribers",
            message.id,
            subscribers.len()
        );

        for sub in subscribers {
            if sub.digest_mode != "none" {
                info!(
                    "Subscriber {} is on digest mode, skipping immediate delivery",
                    sub.email
                );
                continue;
            }

            if let Err(e) = self.send_to_subscriber(message, &sub.email).await {
                error!("Failed to deliver to {}: {}", sub.email, e);
            }
        }

        Ok(())
    }

    async fn send_to_subscriber(
        &self,
        message: &email_message::Model,
        to_email: &str,
    ) -> anyhow::Result<()> {
        use lettre::Transport;
        use lettre::message::{Mailbox, Message, header::ContentType};

        let from = message
            .from_addr
            .parse::<Mailbox>()
            .unwrap_or_else(|_| "noreply@oak-maillist".parse().unwrap());

        let to = to_email.parse::<Mailbox>()?;

        let email_builder = Message::builder()
            .from(from)
            .to(to)
            .subject(message.subject.clone().unwrap_or_default());

        let email = if let Some(ref html) = message.body_html {
            email_builder
                .header(ContentType::TEXT_HTML)
                .body(html.clone())?
        } else {
            email_builder
                .header(ContentType::TEXT_PLAIN)
                .body(message.body_text.clone().unwrap_or_default())?
        };

        let smtp_config = &self.config.smtp.outgoing;
        if smtp_config.host.is_empty() {
            info!("SMTP outgoing not configured, would send to: {}", to_email);
            return Ok(());
        }

        let creds = lettre::transport::smtp::authentication::Credentials::new(
            smtp_config.username.clone(),
            smtp_config.password.clone(),
        );

        let mailer = lettre::SmtpTransport::relay(&smtp_config.host)?
            .port(smtp_config.port)
            .credentials(creds)
            .build();

        mailer.send(&email)?;
        info!("Delivered message to {}", to_email);

        Ok(())
    }
}

fn get_body_text(parsed: &ParsedMail) -> String {
    if parsed.subparts.is_empty() {
        return parsed.get_body().unwrap_or_default();
    }

    for part in &parsed.subparts {
        let ctype = &part.ctype;
        if ctype.mimetype == "text/plain" {
            return part.get_body().unwrap_or_default();
        }
    }

    parsed.get_body().unwrap_or_default()
}

fn get_body_html(parsed: &ParsedMail) -> Option<String> {
    if parsed.subparts.is_empty() {
        return if parsed.ctype.mimetype == "text/html" {
            parsed.get_body().ok()
        } else {
            None
        };
    }

    for part in &parsed.subparts {
        let ctype = &part.ctype;
        if ctype.mimetype == "text/html" {
            return part.get_body().ok();
        }
    }

    None
}

fn get_header_value(parsed: &ParsedMail, key: &str) -> Option<String> {
    parsed
        .headers
        .iter()
        .find(|h| h.get_key().eq_ignore_ascii_case(key))
        .map(|h| h.get_value())
}

fn normalize_subject(subject: &str) -> String {
    let mut result = subject.to_string();
    for prefix in &["Re: ", "RE: ", "Fw: ", "FW: ", "Fwd: ", "FWD: "] {
        if result.starts_with(prefix) {
            result = result[prefix.len()..].to_string();
        }
    }
    result.trim().to_string()
}
