use crate::models::{email_message, moderation_queue, sender_policy};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};

pub struct ModerationService {
    db: DatabaseConnection,
}

impl ModerationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn approve(
        &self,
        moderation_id: &str,
        moderated_by: Option<&str>,
    ) -> anyhow::Result<()> {
        let id = uuid::Uuid::parse_str(moderation_id)?;
        let item = moderation_queue::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Moderation item not found"))?;

        let mut active: moderation_queue::ActiveModel = item.into();
        active.status = Set("approved".to_string());
        if let Some(user_id) = moderated_by {
            active.moderated_by = Set(Some(uuid::Uuid::parse_str(user_id)?));
        }
        active.moderated_at = Set(Some(Utc::now().into()));
        active.update(&self.db).await?;

        Ok(())
    }

    pub async fn reject(
        &self,
        moderation_id: &str,
        moderated_by: Option<&str>,
        note: Option<&str>,
    ) -> anyhow::Result<()> {
        let id = uuid::Uuid::parse_str(moderation_id)?;
        let item = moderation_queue::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Moderation item not found"))?;

        let mut active: moderation_queue::ActiveModel = item.into();
        active.status = Set("rejected".to_string());
        if let Some(user_id) = moderated_by {
            active.moderated_by = Set(Some(uuid::Uuid::parse_str(user_id)?));
        }
        active.moderated_at = Set(Some(Utc::now().into()));
        if let Some(n) = note {
            active.moderation_note = Set(Some(n.to_string()));
        }
        active.update(&self.db).await?;

        Ok(())
    }

    pub async fn discard(
        &self,
        moderation_id: &str,
        moderated_by: Option<&str>,
    ) -> anyhow::Result<()> {
        let id = uuid::Uuid::parse_str(moderation_id)?;
        let item = moderation_queue::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Moderation item not found"))?;

        let mut active: moderation_queue::ActiveModel = item.into();
        active.status = Set("discarded".to_string());
        if let Some(user_id) = moderated_by {
            active.moderated_by = Set(Some(uuid::Uuid::parse_str(user_id)?));
        }
        active.moderated_at = Set(Some(Utc::now().into()));
        active.update(&self.db).await?;

        Ok(())
    }

    pub async fn whitelist_sender(
        &self,
        moderation_id: &str,
        moderated_by: Option<&str>,
    ) -> anyhow::Result<()> {
        let id = uuid::Uuid::parse_str(moderation_id)?;
        let item = moderation_queue::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Moderation item not found"))?;

        self.approve(moderation_id, moderated_by).await?;

        let policy = sender_policy::ActiveModel {
            id: Set(crate::utils::crypto::generate_uuid()),
            list_id: Set(Some(item.list_id)),
            email_pattern: Set(item.from_addr),
            policy_type: Set("whitelist".to_string()),
            scope: Set("post".to_string()),
            note: Set(Some("Auto-whitelisted by moderator".to_string())),
            created_by: Set(moderated_by.map(|id| uuid::Uuid::parse_str(id).unwrap())),
            created_at: Set(Utc::now().into()),
        };
        policy.insert(&self.db).await?;

        Ok(())
    }

    pub async fn blacklist_sender(
        &self,
        moderation_id: &str,
        moderated_by: Option<&str>,
    ) -> anyhow::Result<()> {
        let id = uuid::Uuid::parse_str(moderation_id)?;
        let item = moderation_queue::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Moderation item not found"))?;

        self.reject(moderation_id, moderated_by, Some("Sender blacklisted")).await?;

        let policy = sender_policy::ActiveModel {
            id: Set(crate::utils::crypto::generate_uuid()),
            list_id: Set(Some(item.list_id)),
            email_pattern: Set(item.from_addr),
            policy_type: Set("blacklist".to_string()),
            scope: Set("post".to_string()),
            note: Set(Some("Auto-blacklisted by moderator".to_string())),
            created_by: Set(moderated_by.map(|id| uuid::Uuid::parse_str(id).unwrap())),
            created_at: Set(Utc::now().into()),
        };
        policy.insert(&self.db).await?;

        Ok(())
    }
}
