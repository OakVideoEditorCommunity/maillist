use crate::models::domain;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub struct DomainService {
    db: DatabaseConnection,
}

impl DomainService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list(&self) -> anyhow::Result<Vec<domain::Model>> {
        let items = domain::Entity::find().all(&self.db).await?;
        Ok(items)
    }

    pub async fn create(&self, name: &str) -> anyhow::Result<domain::Model> {
        let model = domain::ActiveModel {
            id: Set(crate::utils::crypto::generate_uuid()),
            name: Set(name.to_string()),
            smtp_host: Set(None),
            smtp_port: Set(None),
            smtp_username: Set(None),
            smtp_password: Set(None),
            dkim_selector: Set(None),
            dkim_private_key: Set(None),
            dkim_public_key: Set(None),
            spf_record: Set(None),
            dmarc_record: Set(None),
            spf_verified: Set(false),
            dkim_verified: Set(false),
            dmarc_verified: Set(false),
            dkim_enabled: Set(false),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        Ok(model.insert(&self.db).await?)
    }

    pub async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<domain::Model>> {
        let uuid = uuid::Uuid::parse_str(id)?;
        Ok(domain::Entity::find_by_id(uuid).one(&self.db).await?)
    }

    pub async fn update(
        &self,
        id: &str,
        updates: serde_json::Value,
    ) -> anyhow::Result<domain::Model> {
        let uuid = uuid::Uuid::parse_str(id)?;
        let item = domain::Entity::find_by_id(uuid)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Domain not found"))?;

        let mut active: domain::ActiveModel = item.into();
        if let Some(v) = updates.get("name").and_then(|v| v.as_str()) {
            active.name = Set(v.to_string());
        }
        if let Some(v) = updates.get("smtp_host").and_then(|v| v.as_str()) {
            active.smtp_host = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("smtp_port").and_then(|v| v.as_i64()) {
            active.smtp_port = Set(Some(v as i32));
        }
        if let Some(v) = updates.get("smtp_username").and_then(|v| v.as_str()) {
            active.smtp_username = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("smtp_password").and_then(|v| v.as_str()) {
            active.smtp_password = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("dkim_selector").and_then(|v| v.as_str()) {
            active.dkim_selector = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("dkim_private_key").and_then(|v| v.as_str()) {
            active.dkim_private_key = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("dkim_public_key").and_then(|v| v.as_str()) {
            active.dkim_public_key = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("spf_record").and_then(|v| v.as_str()) {
            active.spf_record = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("dmarc_record").and_then(|v| v.as_str()) {
            active.dmarc_record = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("spf_verified").and_then(|v| v.as_bool()) {
            active.spf_verified = Set(v);
        }
        if let Some(v) = updates.get("dkim_verified").and_then(|v| v.as_bool()) {
            active.dkim_verified = Set(v);
        }
        if let Some(v) = updates.get("dmarc_verified").and_then(|v| v.as_bool()) {
            active.dmarc_verified = Set(v);
        }
        if let Some(v) = updates.get("dkim_enabled").and_then(|v| v.as_bool()) {
            active.dkim_enabled = Set(v);
        }
        active.updated_at = Set(Utc::now().into());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(id)?;
        domain::Entity::delete_by_id(uuid).exec(&self.db).await?;
        Ok(())
    }
}
