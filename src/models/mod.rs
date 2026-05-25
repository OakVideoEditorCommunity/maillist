use sea_orm::DatabaseConnection;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
}

pub mod prelude;

pub mod domain;
pub mod mailing_list;
pub mod user;
pub mod subscriber;
pub mod email_message;
pub mod moderation_queue;
pub mod list_membership;
pub mod totp_credential;
pub mod passkey_credential;
pub mod auth_session;
pub mod refresh_token;
pub mod sender_policy;
pub mod attachment;
pub mod email_template;
pub mod bounce_log;
