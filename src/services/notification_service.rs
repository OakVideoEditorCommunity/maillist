use crate::config::SmtpOutgoingConfig;
use crate::models::{mailing_list, subscriber};
use sea_orm::DatabaseConnection;
use tera::Context;

pub struct NotificationService {
    _db: DatabaseConnection,
    template_svc: super::template_service::TemplateService,
    smtp: SmtpOutgoingConfig,
}

impl NotificationService {
    pub fn new(db: DatabaseConnection, smtp: SmtpOutgoingConfig) -> Self {
        let template_svc = super::template_service::TemplateService::new(db.clone());
        Self {
            _db: db,
            template_svc,
            smtp,
        }
    }

    pub async fn send_subscription_confirm(
        &self,
        sub: &subscriber::Model,
        list: &mailing_list::Model,
        confirm_url: &str,
    ) -> anyhow::Result<()> {
        let mut ctx = Context::new();
        ctx.insert("subscriber_email", &sub.email);
        ctx.insert("subscriber_name", &sub.name);
        ctx.insert("list_name", &list.name);
        ctx.insert("confirm_url", confirm_url);

        self.template_svc
            .send_templated_email(&sub.email, "subscription_confirm", &ctx, &self.smtp)
            .await
    }

    pub async fn send_unsubscribe_confirm(
        &self,
        sub: &subscriber::Model,
        list: &mailing_list::Model,
        confirm_url: &str,
    ) -> anyhow::Result<()> {
        let mut ctx = Context::new();
        ctx.insert("subscriber_email", &sub.email);
        ctx.insert("subscriber_name", &sub.name);
        ctx.insert("list_name", &list.name);
        ctx.insert("confirm_url", confirm_url);

        self.template_svc
            .send_templated_email(&sub.email, "unsubscribe_confirm", &ctx, &self.smtp)
            .await
    }

    pub async fn send_moderation_notice(
        &self,
        moderator_email: &str,
        list: &mailing_list::Model,
        message_subject: &str,
        review_url: &str,
    ) -> anyhow::Result<()> {
        let mut ctx = Context::new();
        ctx.insert("moderator_email", moderator_email);
        ctx.insert("list_name", &list.name);
        ctx.insert("message_subject", message_subject);
        ctx.insert("review_url", review_url);

        self.template_svc
            .send_templated_email(moderator_email, "moderation_notice", &ctx, &self.smtp)
            .await
    }

    pub async fn send_welcome(
        &self,
        sub: &subscriber::Model,
        list: &mailing_list::Model,
    ) -> anyhow::Result<()> {
        let mut ctx = Context::new();
        ctx.insert("subscriber_email", &sub.email);
        ctx.insert("subscriber_name", &sub.name);
        ctx.insert("list_name", &list.name);

        self.template_svc
            .send_templated_email(&sub.email, "welcome", &ctx, &self.smtp)
            .await
    }
}
