use crate::models::{AppState, email_message, subscriber};
use chrono::{Duration, Utc};
use lettre::Transport;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use tracing::{error, info};

pub struct DigestTask {
    db: DatabaseConnection,
    state: AppState,
}

impl DigestTask {
    pub fn new(state: AppState) -> Self {
        Self {
            db: state.db.clone(),
            state,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Starting digest generation task");

        let daily_subscribers = self.get_digest_subscribers("daily").await?;
        let weekly_subscribers = self.get_digest_subscribers("weekly").await?;

        for sub in daily_subscribers {
            if let Err(e) = self.generate_digest(&sub, Duration::hours(24)).await {
                error!("Failed to generate daily digest for {}: {}", sub.email, e);
            }
        }

        for sub in weekly_subscribers {
            if let Err(e) = self.generate_digest(&sub, Duration::hours(24 * 7)).await {
                error!("Failed to generate weekly digest for {}: {}", sub.email, e);
            }
        }

        info!("Digest generation task completed");
        Ok(())
    }

    async fn get_digest_subscribers(&self, mode: &str) -> anyhow::Result<Vec<subscriber::Model>> {
        let subs = subscriber::Entity::find()
            .filter(subscriber::Column::Status.eq("active"))
            .filter(subscriber::Column::DigestMode.eq(mode))
            .all(&self.db)
            .await?;
        Ok(subs)
    }

    async fn generate_digest(
        &self,
        sub: &subscriber::Model,
        since: Duration,
    ) -> anyhow::Result<()> {
        let cutoff = Utc::now() - since;

        let messages = email_message::Entity::find()
            .filter(email_message::Column::ListId.eq(sub.list_id))
            .filter(email_message::Column::IsDeleted.eq(false))
            .filter(email_message::Column::ReceivedAt.gte(cutoff))
            .order_by_asc(email_message::Column::ReceivedAt)
            .all(&self.db)
            .await?;

        if messages.is_empty() {
            return Ok(());
        }

        let mut digest_body = format!(
            "邮件列表摘要 - {}\n==================\n\n",
            Utc::now().format("%Y-%m-%d")
        );

        for msg in &messages {
            digest_body.push_str(&format!(
                "[{}] {}\n来自: {}\n\n{:.200}\n\n---\n\n",
                msg.received_at.format("%H:%M"),
                msg.subject.as_deref().unwrap_or("(无主题)"),
                msg.from_addr,
                msg.body_text.as_deref().unwrap_or(""),
            ));
        }

        info!(
            "Generated {} digest for {} with {} messages",
            sub.digest_mode,
            sub.email,
            messages.len()
        );

        let smtp_config = &self.state.config.smtp.outgoing;
        if !smtp_config.host.is_empty() {
            let email = lettre::Message::builder()
                .from(smtp_config.from_address.parse()?)
                .to(sub.email.parse()?)
                .subject(format!("[摘要] {} 封新邮件", messages.len()))
                .body(digest_body)?;

            let creds = lettre::transport::smtp::authentication::Credentials::new(
                smtp_config.username.clone(),
                smtp_config.password.clone(),
            );
            let mailer = lettre::SmtpTransport::relay(&smtp_config.host)?
                .port(smtp_config.port)
                .credentials(creds)
                .build();

            mailer.send(&email)?;
            info!("Digest sent to {}", sub.email);
        }

        Ok(())
    }
}
