use crate::config::AppConfig;
use crate::models::{totp_credential, user};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use totp_rs::{Algorithm, Secret, TOTP};

pub struct MfaService {
    db: DatabaseConnection,
    _config: AppConfig,
}

impl MfaService {
    pub fn new(db: DatabaseConnection, config: AppConfig) -> Self {
        Self {
            db,
            _config: config,
        }
    }

    pub async fn setup_totp(
        &self,
        user_id: &str,
        issuer: &str,
    ) -> anyhow::Result<(String, String)> {
        let secret_bytes: Vec<u8> = (0..20).map(|_| rand::random::<u8>()).collect();
        let secret = base32::encode(&secret_bytes);

        let user_uuid = uuid::Uuid::parse_str(user_id)?;
        let user = user::Entity::find_by_id(user_uuid)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes.clone(),
            Some(issuer.to_string()),
            user.email.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create TOTP: {}", e))?;

        let qr_url = totp.get_url();

        let cred = totp_credential::ActiveModel {
            id: Set(crate::utils::crypto::generate_uuid()),
            user_id: Set(user_uuid),
            secret: Set(secret.clone()),
            issuer: Set(Some(issuer.to_string())),
            account_name: Set(Some(user.email.clone())),
            algorithm: Set("SHA1".to_string()),
            digits: Set(6),
            period: Set(30),
            verified: Set(false),
            backup_codes: Set(None),
            created_at: Set(Utc::now().into()),
            last_used_at: Set(None),
        };

        cred.insert(&self.db).await?;

        Ok((secret, qr_url))
    }

    pub async fn verify_totp_setup(
        &self,
        user_id: &str,
        code: &str,
    ) -> anyhow::Result<Vec<String>> {
        let user_uuid = uuid::Uuid::parse_str(user_id)?;

        let cred = totp_credential::Entity::find()
            .filter(totp_credential::Column::UserId.eq(user_uuid))
            .filter(totp_credential::Column::Verified.eq(false))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No pending TOTP setup found"))?;

        if !self.check_totp_code(&cred.secret, code)? {
            return Err(anyhow::anyhow!("Invalid TOTP code"));
        }

        let backup_codes = self.generate_backup_codes();
        let backup_codes_json = serde_json::to_value(&backup_codes)?;

        let mut active: totp_credential::ActiveModel = cred.into();
        active.verified = Set(true);
        active.backup_codes = Set(Some(backup_codes_json));
        active.update(&self.db).await?;

        let mut user_active: user::ActiveModel = user::Entity::find_by_id(user_uuid)
            .one(&self.db)
            .await?
            .unwrap()
            .into();
        user_active.mfa_enabled = Set(true);
        user_active.update(&self.db).await?;

        Ok(backup_codes)
    }

    pub async fn verify_totp(&self, user_id: &str, code: &str) -> anyhow::Result<bool> {
        let user_uuid = uuid::Uuid::parse_str(user_id)?;

        let cred = totp_credential::Entity::find()
            .filter(totp_credential::Column::UserId.eq(user_uuid))
            .filter(totp_credential::Column::Verified.eq(true))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("TOTP not configured"))?;

        let valid = self.check_totp_code(&cred.secret, code)?;

        if valid {
            let mut active: totp_credential::ActiveModel = cred.into();
            active.last_used_at = Set(Some(Utc::now().into()));
            active.update(&self.db).await?;
        }

        Ok(valid)
    }

    pub async fn disable_totp(&self, user_id: &str, code: &str) -> anyhow::Result<()> {
        let user_uuid = uuid::Uuid::parse_str(user_id)?;

        let cred = totp_credential::Entity::find()
            .filter(totp_credential::Column::UserId.eq(user_uuid))
            .filter(totp_credential::Column::Verified.eq(true))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("TOTP not configured"))?;

        if !self.check_totp_code(&cred.secret, code)? {
            return Err(anyhow::anyhow!("Invalid TOTP code"));
        }

        totp_credential::Entity::delete_by_id(cred.id)
            .exec(&self.db)
            .await?;

        let mut user_active: user::ActiveModel = user::Entity::find_by_id(user_uuid)
            .one(&self.db)
            .await?
            .unwrap()
            .into();
        user_active.mfa_enabled = Set(false);
        user_active.update(&self.db).await?;

        Ok(())
    }

    pub async fn regenerate_backup_codes(&self, user_id: &str) -> anyhow::Result<Vec<String>> {
        let user_uuid = uuid::Uuid::parse_str(user_id)?;

        let cred = totp_credential::Entity::find()
            .filter(totp_credential::Column::UserId.eq(user_uuid))
            .filter(totp_credential::Column::Verified.eq(true))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("TOTP not configured"))?;

        let backup_codes = self.generate_backup_codes();
        let backup_codes_json = serde_json::to_value(&backup_codes)?;

        let mut active: totp_credential::ActiveModel = cred.into();
        active.backup_codes = Set(Some(backup_codes_json));
        active.update(&self.db).await?;

        Ok(backup_codes)
    }

    pub async fn get_backup_codes_count(&self, user_id: &str) -> anyhow::Result<usize> {
        let user_uuid = uuid::Uuid::parse_str(user_id)?;

        let cred = totp_credential::Entity::find()
            .filter(totp_credential::Column::UserId.eq(user_uuid))
            .filter(totp_credential::Column::Verified.eq(true))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("TOTP not configured"))?;

        let count = cred
            .backup_codes
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(count)
    }

    fn check_totp_code(&self, secret: &str, code: &str) -> anyhow::Result<bool> {
        let secret_bytes =
            base32::decode(secret).ok_or_else(|| anyhow::anyhow!("Invalid base32 secret"))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            None,
            "".to_string(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create TOTP: {}", e))?;

        Ok(totp.check(
            code,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        ))
    }

    fn generate_backup_codes(&self) -> Vec<String> {
        (0..10)
            .map(|_| crate::utils::crypto::generate_random_token(8).to_lowercase())
            .collect()
    }
}

mod base32 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    pub fn encode(data: &[u8]) -> String {
        let mut result = String::new();
        let mut bits = 0u32;
        let mut bit_count = 0;

        for &byte in data {
            bits = (bits << 8) | byte as u32;
            bit_count += 8;
            while bit_count >= 5 {
                result.push(ALPHABET[((bits >> (bit_count - 5)) & 0x1F) as usize] as char);
                bit_count -= 5;
            }
        }

        if bit_count > 0 {
            result.push(ALPHABET[((bits << (5 - bit_count)) & 0x1F) as usize] as char);
        }

        result
    }

    pub fn decode(input: &str) -> Option<Vec<u8>> {
        let mut result = Vec::new();
        let mut bits = 0u32;
        let mut bit_count = 0;

        for ch in input.to_uppercase().chars() {
            let val = ALPHABET.iter().position(|&b| b as char == ch)? as u32;
            bits = (bits << 5) | val;
            bit_count += 5;
            if bit_count >= 8 {
                result.push(((bits >> (bit_count - 8)) & 0xFF) as u8);
                bit_count -= 8;
            }
        }

        Some(result)
    }
}
