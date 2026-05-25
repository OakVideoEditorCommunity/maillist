use crate::models::{domain, mailing_list};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

pub struct ListService {
    db: DatabaseConnection,
}

impl ListService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        domain_id: &str,
        name: &str,
        email_local_part: &str,
        display_name: Option<&str>,
        description: Option<&str>,
    ) -> anyhow::Result<mailing_list::Model> {
        let domain_uuid = uuid::Uuid::parse_str(domain_id)?;

        let list = mailing_list::ActiveModel {
            id: Set(crate::utils::crypto::generate_uuid()),
            domain_id: Set(domain_uuid),
            name: Set(name.to_string()),
            display_name: Set(display_name.map(|s| s.to_string())),
            email_local_part: Set(email_local_part.to_string()),
            description: Set(description.map(|s| s.to_string())),
            visibility: Set("public".to_string()),
            subscription_policy: Set("confirm".to_string()),
            post_policy: Set("subscriber_only".to_string()),
            reply_to: Set("list".to_string()),
            archive_enabled: Set(true),
            archive_visibility: Set("public".to_string()),
            max_message_size_kb: Set(1024),
            digest_enabled: Set(false),
            header_template: Set(None),
            footer_template: Set(None),
            ai_moderation_enabled: Set(true),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let result = list.insert(&self.db).await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<mailing_list::Model>> {
        let uuid = uuid::Uuid::parse_str(id)?;
        let list = mailing_list::Entity::find_by_id(uuid).one(&self.db).await?;
        Ok(list)
    }

    pub async fn list_public(
        &self,
        page: u64,
        per_page: u64,
    ) -> anyhow::Result<(Vec<mailing_list::Model>, u64)> {
        let paginator = mailing_list::Entity::find()
            .filter(mailing_list::Column::Visibility.eq("public"))
            .filter(mailing_list::Column::IsActive.eq(true))
            .order_by_desc(mailing_list::Column::CreatedAt)
            .paginate(&self.db, per_page);

        let items = paginator.fetch_page(page - 1).await?;
        let total = paginator.num_items().await?;

        Ok((items, total))
    }

    pub async fn update(
        &self,
        id: &str,
        updates: serde_json::Value,
    ) -> anyhow::Result<mailing_list::Model> {
        let uuid = uuid::Uuid::parse_str(id)?;
        let list = mailing_list::Entity::find_by_id(uuid)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("List not found"))?;

        let mut active: mailing_list::ActiveModel = list.into();

        if let Some(v) = updates.get("display_name").and_then(|v| v.as_str()) {
            active.display_name = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("description").and_then(|v| v.as_str()) {
            active.description = Set(Some(v.to_string()));
        }
        if let Some(v) = updates.get("visibility").and_then(|v| v.as_str()) {
            active.visibility = Set(v.to_string());
        }
        if let Some(v) = updates.get("subscription_policy").and_then(|v| v.as_str()) {
            active.subscription_policy = Set(v.to_string());
        }
        if let Some(v) = updates.get("post_policy").and_then(|v| v.as_str()) {
            active.post_policy = Set(v.to_string());
        }
        if let Some(v) = updates.get("reply_to").and_then(|v| v.as_str()) {
            active.reply_to = Set(v.to_string());
        }
        if let Some(v) = updates.get("archive_enabled").and_then(|v| v.as_bool()) {
            active.archive_enabled = Set(v);
        }
        if let Some(v) = updates.get("archive_visibility").and_then(|v| v.as_str()) {
            active.archive_visibility = Set(v.to_string());
        }
        if let Some(v) = updates.get("max_message_size_kb").and_then(|v| v.as_i64()) {
            active.max_message_size_kb = Set(v as i32);
        }
        if let Some(v) = updates.get("digest_enabled").and_then(|v| v.as_bool()) {
            active.digest_enabled = Set(v);
        }
        if let Some(v) = updates.get("ai_moderation_enabled").and_then(|v| v.as_bool()) {
            active.ai_moderation_enabled = Set(v);
        }

        active.updated_at = Set(Utc::now().into());
        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(id)?;
        let list = mailing_list::Entity::find_by_id(uuid)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("List not found"))?;

        let mut active: mailing_list::ActiveModel = list.into();
        active.is_active = Set(false);
        active.updated_at = Set(Utc::now().into());
        active.update(&self.db).await?;

        Ok(())
    }
}
