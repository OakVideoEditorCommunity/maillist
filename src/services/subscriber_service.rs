use crate::models::{mailing_list, subscriber};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use tracing::info;

pub struct SubscriberService {
    db: DatabaseConnection,
}

impl SubscriberService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn subscribe(
        &self,
        list_id: &str,
        email: &str,
        name: Option<&str>,
        base_url: &str,
    ) -> anyhow::Result<subscriber::Model> {
        let list_uuid = uuid::Uuid::parse_str(list_id)?;
        let list = mailing_list::Entity::find_by_id(list_uuid)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("List not found"))?;

        let existing = subscriber::Entity::find()
            .filter(subscriber::Column::ListId.eq(list_uuid))
            .filter(subscriber::Column::Email.eq(email))
            .one(&self.db)
            .await?;

        if let Some(existing) = existing {
            if existing.status == "active" {
                anyhow::bail!("Already subscribed");
            }
            return Ok(existing);
        }

        let token = crate::utils::crypto::generate_random_token(32);
        let sub = subscriber::ActiveModel {
            id: Set(crate::utils::crypto::generate_uuid()),
            list_id: Set(list_uuid),
            email: Set(email.to_string()),
            name: Set(name.map(|s| s.to_string())),
            status: Set("pending".to_string()),
            digest_mode: Set("individual".to_string()),
            subscribe_ip: Set(None),
            subscribe_source: Set(Some("web".to_string())),
            bounce_count: Set(0),
            last_bounce_at: Set(None),
            token: Set(token.clone()),
            confirmed_at: Set(None),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let model = sub.insert(&self.db).await?;
        info!("Subscriber {} created for list {} with token {}", email, list_id, token);

        Ok(model)
    }

    pub async fn confirm(&self, list_id: &str, token: &str) -> anyhow::Result<subscriber::Model> {
        let list_uuid = uuid::Uuid::parse_str(list_id)?;
        let sub = subscriber::Entity::find()
            .filter(subscriber::Column::ListId.eq(list_uuid))
            .filter(subscriber::Column::Token.eq(token))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid confirmation token"))?;

        if sub.status == "active" {
            return Ok(sub);
        }

        let mut active: subscriber::ActiveModel = sub.into();
        active.status = Set("active".to_string());
        active.confirmed_at = Set(Some(Utc::now().into()));
        active.updated_at = Set(Utc::now().into());
        let model = active.update(&self.db).await?;
        info!("Subscriber {} confirmed", model.email);
        Ok(model)
    }

    pub async fn unsubscribe(&self, list_id: &str, token: &str) -> anyhow::Result<()> {
        let list_uuid = uuid::Uuid::parse_str(list_id)?;
        let sub = subscriber::Entity::find()
            .filter(subscriber::Column::ListId.eq(list_uuid))
            .filter(subscriber::Column::Token.eq(token))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Subscriber not found"))?;

        let mut active: subscriber::ActiveModel = sub.into();
        active.status = Set("unsubscribed".to_string());
        active.updated_at = Set(Utc::now().into());
        active.update(&self.db).await?;
        info!("Subscriber unsubscribed");
        Ok(())
    }

    pub async fn list_by_list(
        &self,
        list_id: &str,
        page: u64,
        per_page: u64,
    ) -> anyhow::Result<(Vec<subscriber::Model>, u64)> {
        use sea_orm::PaginatorTrait;
        let list_uuid = uuid::Uuid::parse_str(list_id)?;
        let paginator = subscriber::Entity::find()
            .filter(subscriber::Column::ListId.eq(list_uuid))
            .paginate(&self.db, per_page);

        let items = paginator.fetch_page(page - 1).await?;
        let total = paginator.num_items().await?;
        Ok((items, total))
    }

    pub async fn update_digest_mode(
        &self,
        subscriber_id: &str,
        mode: &str,
    ) -> anyhow::Result<subscriber::Model> {
        let id = uuid::Uuid::parse_str(subscriber_id)?;
        let sub = subscriber::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Subscriber not found"))?;

        let mut active: subscriber::ActiveModel = sub.into();
        active.digest_mode = Set(mode.to_string());
        active.updated_at = Set(Utc::now().into());
        let model = active.update(&self.db).await?;
        Ok(model)
    }
}
