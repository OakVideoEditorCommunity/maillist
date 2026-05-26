use crate::models::email_template;
use lettre::Transport;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tera::Tera;
use tracing::info;

pub struct TemplateService {
    db: DatabaseConnection,
    _tera: Tera,
}

impl TemplateService {
    pub fn new(db: DatabaseConnection) -> Self {
        let mut tera = Tera::default();
        tera.autoescape_on(vec!["html"]);
        Self { db, _tera: tera }
    }

    pub async fn render_template(
        &self,
        name: &str,
        vars: &tera::Context,
    ) -> anyhow::Result<(Option<String>, Option<String>)> {
        let tmpl = email_template::Entity::find()
            .filter(email_template::Column::Name.eq(name))
            .one(&self.db)
            .await?;

        let Some(tmpl) = tmpl else {
            anyhow::bail!("Template '{}' not found", name);
        };

        let subject = if let Some(ref s) = tmpl.subject {
            let rendered = Tera::one_off(s, vars, false)
                .map_err(|e| anyhow::anyhow!("Template subject render error: {}", e))?;
            Some(rendered)
        } else {
            None
        };

        let body_text = if let Some(ref b) = tmpl.body_text {
            let rendered = Tera::one_off(b, vars, false)
                .map_err(|e| anyhow::anyhow!("Template body_text render error: {}", e))?;
            Some(rendered)
        } else {
            None
        };

        let body_html = if let Some(ref b) = tmpl.body_html {
            let rendered = Tera::one_off(b, vars, true)
                .map_err(|e| anyhow::anyhow!("Template body_html render error: {}", e))?;
            Some(rendered)
        } else {
            None
        };

        if body_text.is_some() {
            Ok((subject, body_text))
        } else if body_html.is_some() {
            Ok((subject, body_html))
        } else {
            anyhow::bail!("Template '{}' has no body", name);
        }
    }

    pub async fn send_templated_email(
        &self,
        to: &str,
        template_name: &str,
        vars: &tera::Context,
        smtp_config: &crate::config::SmtpOutgoingConfig,
    ) -> anyhow::Result<()> {
        let (subject_opt, body_opt) = self.render_template(template_name, vars).await?;

        if smtp_config.host.is_empty() {
            info!("SMTP outgoing not configured, skipping email to {}", to);
            return Ok(());
        }

        let subject = subject_opt.unwrap_or_else(|| "Notification".to_string());
        let body = body_opt.unwrap_or_default();

        let email = lettre::Message::builder()
            .from(smtp_config.from_address.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body)?;

        let creds = lettre::transport::smtp::authentication::Credentials::new(
            smtp_config.username.clone(),
            smtp_config.password.clone(),
        );
        let mailer = lettre::SmtpTransport::relay(&smtp_config.host)?
            .port(smtp_config.port)
            .credentials(creds)
            .build();

        mailer.send(&email)?;
        info!(
            "Templated email sent to {} using template '{}'",
            to, template_name
        );
        Ok(())
    }
}
