use crate::config::AppConfig;
use crate::models::passkey_credential;
use crate::models::user;
use crate::utils::crypto::{generate_random_token, generate_uuid};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use webauthn_rs::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    pub user_id: Uuid,
    pub state: PasskeyRegistration,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    pub user_id: Option<Uuid>,
    pub state: PasskeyAuthentication,
}

pub struct PasskeyService {
    db: DatabaseConnection,
    webauthn: Arc<Webauthn>,
    challenges: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl PasskeyService {
    pub fn new(
        db: DatabaseConnection,
        config: &AppConfig,
        challenges: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    ) -> anyhow::Result<Self> {
        let rp_origin = Url::parse(&config.server.base_url)?;
        let rp_id = config
            .security
            .webauthn_rp_id
            .as_deref()
            .or(rp_origin.domain())
            .unwrap_or("localhost");
        let builder = WebauthnBuilder::new(rp_id, &rp_origin)?;
        let webauthn = Arc::new(builder.build()?);
        Ok(Self {
            db,
            webauthn,
            challenges,
        })
    }

    pub fn from_state(
        db: DatabaseConnection,
        webauthn: Arc<Webauthn>,
        challenges: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    ) -> Self {
        Self {
            db,
            webauthn,
            challenges,
        }
    }

    pub async fn start_registration(
        &self,
        user_id: Uuid,
        email: &str,
        name: Option<&str>,
    ) -> anyhow::Result<(CreationChallengeResponse, String)> {
        let existing = passkey_credential::Entity::find()
            .filter(passkey_credential::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?;

        let exclude_creds: Option<Vec<CredentialID>> = if existing.is_empty() {
            None
        } else {
            Some(
                existing
                    .iter()
                    .map(|c| CredentialID::from(c.credential_id.clone()))
                    .collect(),
            )
        };

        let (ccr, state) = self.webauthn.start_passkey_registration(
            user_id,
            email,
            name.unwrap_or(email),
            exclude_creds,
        )?;

        let challenge_id = generate_random_token(32);
        let reg_state = RegistrationChallenge { user_id, state };
        let serialized = serde_json::to_vec(&reg_state)?;

        self.challenges
            .write()
            .await
            .insert(challenge_id.clone(), serialized);

        Ok((ccr, challenge_id))
    }

    pub async fn finish_registration(
        &self,
        challenge_id: &str,
        credential: &RegisterPublicKeyCredential,
    ) -> anyhow::Result<passkey_credential::Model> {
        let serialized = self
            .challenges
            .write()
            .await
            .remove(challenge_id)
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired challenge"))?;
        let reg_state: RegistrationChallenge = serde_json::from_slice(&serialized)?;

        let passkey = self
            .webauthn
            .finish_passkey_registration(credential, &reg_state.state)?;

        let cred: webauthn_rs::prelude::Credential = passkey.into();
        let cred_id = cred.cred_id.clone();
        let pk_json = serde_json::to_vec(&Passkey::from(cred.clone()))?;

        let model = passkey_credential::ActiveModel {
            id: Set(generate_uuid()),
            user_id: Set(reg_state.user_id),
            credential_id: Set(cred_id.to_vec()),
            public_key: Set(pk_json),
            sign_count: Set(0),
            aaguid: Set(None),
            device_name: Set(None),
            transports: Set(None),
            is_backup_eligible: Set(cred.backup_eligible),
            is_backup: Set(cred.backup_state),
            last_used_at: Set(None),
            created_at: Set(Utc::now().into()),
        };

        model.insert(&self.db).await.map_err(Into::into)
    }

    pub async fn start_authentication(
        &self,
        email: Option<&str>,
    ) -> anyhow::Result<(RequestChallengeResponse, String)> {
        let mut target_user_id: Option<Uuid> = None;
        let stored = if let Some(email) = email {
            let user = user::Entity::find()
                .filter(user::Column::Email.eq(email.to_lowercase()))
                .one(&self.db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("User not found"))?;
            target_user_id = Some(user.id);

            let creds = passkey_credential::Entity::find()
                .filter(passkey_credential::Column::UserId.eq(user.id))
                .all(&self.db)
                .await?;

            if creds.is_empty() {
                return Err(anyhow::anyhow!("No passkey registered for this user"));
            }
            creds
        } else {
            return Err(anyhow::anyhow!("Email is required"));
        };

        let passkeys: Vec<Passkey> = stored
            .iter()
            .map(|c| serde_json::from_slice(&c.public_key))
            .collect::<Result<Vec<_>, _>>()?;

        let (rcr, state) = self.webauthn.start_passkey_authentication(&passkeys)?;

        let challenge_id = generate_random_token(32);
        let auth_state = AuthenticationChallenge {
            user_id: target_user_id,
            state,
        };
        let serialized = serde_json::to_vec(&auth_state)?;

        self.challenges
            .write()
            .await
            .insert(challenge_id.clone(), serialized);

        Ok((rcr, challenge_id))
    }

    pub async fn finish_authentication(
        &self,
        challenge_id: &str,
        credential: &PublicKeyCredential,
    ) -> anyhow::Result<(Uuid, String)> {
        let serialized = self
            .challenges
            .write()
            .await
            .remove(challenge_id)
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired challenge"))?;
        let auth_state: AuthenticationChallenge = serde_json::from_slice(&serialized)?;

        let _result = self
            .webauthn
            .finish_passkey_authentication(credential, &auth_state.state)?;

        let user_id = auth_state
            .user_id
            .ok_or_else(|| anyhow::anyhow!("User not found in auth state"))?;

        let user = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        Ok((user.id, user.email))
    }
}
