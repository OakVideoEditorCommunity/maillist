use crate::config::AppConfig;
use sea_orm::DatabaseConnection;

pub struct MfaService {
    db: DatabaseConnection,
    config: AppConfig,
}

impl MfaService {
    pub fn new(db: DatabaseConnection, config: AppConfig) -> Self {
        Self { db, config }
    }

    pub async fn setup_totp(&self, _user_id: &str) -> anyhow::Result<(String, String)> {
        todo!()
    }

    pub async fn verify_totp(&self, _user_id: &str, _code: &str) -> anyhow::Result<bool> {
        todo!()
    }

    pub async fn register_passkey(&self, _user_id: &str) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn verify_passkey_login(&self, _credential_id: &[u8]) -> anyhow::Result<Option<crate::models::user::Model>> {
        todo!()
    }
}
