use crate::config::AppConfig;
use crate::models::{refresh_token, user};
use crate::utils::crypto::{generate_random_token, generate_uuid, hash_password, verify_password};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

pub struct AuthService {
    db: DatabaseConnection,
    config: AppConfig,
}

impl AuthService {
    pub fn new(db: DatabaseConnection, config: AppConfig) -> Self {
        Self { db, config }
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        name: Option<&str>,
    ) -> anyhow::Result<user::Model> {
        let password_hash = hash_password(password)?;

        let new_user = user::ActiveModel {
            id: Set(generate_uuid()),
            email: Set(email.to_lowercase()),
            password_hash: Set(Some(password_hash)),
            name: Set(name.map(|s| s.to_string())),
            avatar_url: Set(None),
            timezone: Set("Asia/Shanghai".to_string()),
            language: Set("zh-CN".to_string()),
            is_site_admin: Set(false),
            is_active: Set(true),
            mfa_enabled: Set(false),
            last_login_at: Set(None),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let user = new_user.insert(&self.db).await?;
        Ok(user)
    }

    pub async fn login(&self, email: &str, password: &str) -> anyhow::Result<user::Model> {
        let user = user::Entity::find()
            .filter(user::Column::Email.eq(email.to_lowercase()))
            .filter(user::Column::IsActive.eq(true))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid email or password"))?;

        let hash = user
            .password_hash
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Invalid email or password"))?;

        if !verify_password(password, hash)? {
            return Err(anyhow::anyhow!("Invalid email or password"));
        }

        let mut active: user::ActiveModel = user.clone().into();
        active.last_login_at = Set(Some(Utc::now().into()));
        active.update(&self.db).await?;

        Ok(user)
    }

    pub fn generate_access_token(
        &self,
        user_id: &str,
        email: &str,
        role: &str,
    ) -> anyhow::Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.security.jwt_expiration_seconds);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.security.jwt_secret.as_bytes()),
        )?;

        Ok(token)
    }

    pub fn verify_access_token(&self, token: &str) -> anyhow::Result<TokenClaims> {
        let validation = Validation::default();
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.config.security.jwt_secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    pub async fn create_refresh_token(
        &self,
        user_id: &str,
        ip: Option<&str>,
    ) -> anyhow::Result<String> {
        let token = generate_random_token(64);
        let token_hash = sha256::digest(&token);

        let expires_at =
            Utc::now() + Duration::days(self.config.security.refresh_token_expiration_days);

        let refresh = refresh_token::ActiveModel {
            id: Set(generate_uuid()),
            user_id: Set(uuid::Uuid::parse_str(user_id)?),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at.into()),
            revoked_at: Set(None),
            ip_address: Set(ip.map(|s| s.to_string())),
            created_at: Set(Utc::now().into()),
        };

        refresh.insert(&self.db).await?;
        Ok(token)
    }

    pub async fn verify_refresh_token(&self, token: &str) -> anyhow::Result<user::Model> {
        let token_hash = sha256::digest(token);

        let rt = refresh_token::Entity::find()
            .filter(refresh_token::Column::TokenHash.eq(&token_hash))
            .filter(refresh_token::Column::RevokedAt.is_null())
            .filter(refresh_token::Column::ExpiresAt.gt(Utc::now()))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired refresh token"))?;

        let user = user::Entity::find_by_id(rt.user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        if !user.is_active {
            return Err(anyhow::anyhow!("User is inactive"));
        }

        Ok(user)
    }

    pub async fn revoke_refresh_token(&self, token: &str) -> anyhow::Result<()> {
        let token_hash = sha256::digest(token);

        let rt = refresh_token::Entity::find()
            .filter(refresh_token::Column::TokenHash.eq(&token_hash))
            .one(&self.db)
            .await?;

        if let Some(model) = rt {
            let mut active: refresh_token::ActiveModel = model.into();
            active.revoked_at = Set(Some(Utc::now().into()));
            active.update(&self.db).await?;
        }

        Ok(())
    }

    pub async fn revoke_all_user_tokens(&self, user_id: &str) -> anyhow::Result<()> {
        let user_uuid = uuid::Uuid::parse_str(user_id)?;

        let tokens = refresh_token::Entity::find()
            .filter(refresh_token::Column::UserId.eq(user_uuid))
            .filter(refresh_token::Column::RevokedAt.is_null())
            .all(&self.db)
            .await?;

        for token in tokens {
            let mut active: refresh_token::ActiveModel = token.into();
            active.revoked_at = Set(Some(Utc::now().into()));
            active.update(&self.db).await?;
        }

        Ok(())
    }
}

mod sha256 {
    use ring::digest::{Context, SHA256};

    pub fn digest(input: &str) -> String {
        let mut context = Context::new(&SHA256);
        context.update(input.as_bytes());
        let digest = context.finish();
        digest
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}
