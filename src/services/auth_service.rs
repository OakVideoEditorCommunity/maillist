use crate::config::AppConfig;
use crate::models::{user, AppState};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct AuthService {
    db: DatabaseConnection,
    config: AppConfig,
}

impl AuthService {
    pub fn new(db: DatabaseConnection, config: AppConfig) -> Self {
        Self { db, config }
    }

    pub async fn register(&self, _email: &str, _password: &str, _name: Option<&str>) -> anyhow::Result<user::Model> {
        todo!()
    }

    pub async fn login(&self, _email: &str, _password: &str) -> anyhow::Result<user::Model> {
        todo!()
    }

    pub fn generate_access_token(&self, _user_id: &str, _email: &str, _role: &str) -> anyhow::Result<String> {
        todo!()
    }

    pub fn generate_refresh_token(&self) -> String {
        crate::utils::crypto::generate_random_token(64)
    }
}
