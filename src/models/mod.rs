use crate::config::AppConfig;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{Notify, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub webauthn: Option<Arc<webauthn_rs::Webauthn>>,
    pub passkey_challenges: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    pub shutdown: Arc<Notify>,
    pub should_reload: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        config: AppConfig,
        shutdown: Arc<Notify>,
        should_reload: Arc<AtomicBool>,
    ) -> Self {
        let webauthn = Self::build_webauthn(&config);
        Self {
            db,
            config,
            webauthn,
            passkey_challenges: Arc::new(RwLock::new(HashMap::new())),
            shutdown,
            should_reload,
        }
    }

    fn build_webauthn(config: &AppConfig) -> Option<Arc<webauthn_rs::Webauthn>> {
        let rp_origin = webauthn_rs::prelude::Url::parse(&config.server.base_url).ok()?;
        let rp_id = config
            .security
            .webauthn_rp_id
            .as_deref()
            .or(rp_origin.domain())
            .unwrap_or("localhost");
        let builder = webauthn_rs::prelude::WebauthnBuilder::new(rp_id, &rp_origin).ok()?;
        Some(Arc::new(builder.build().ok()?))
    }
}

pub mod prelude;

pub mod attachment;
pub mod auth_session;
pub mod bounce_log;
pub mod domain;
pub mod email_message;
pub mod email_template;
pub mod list_membership;
pub mod mailing_list;
pub mod moderation_queue;
pub mod passkey_credential;
pub mod refresh_token;
pub mod sender_policy;
pub mod subscriber;
pub mod totp_credential;
pub mod user;
