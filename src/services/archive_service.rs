use crate::models::email_message;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

pub struct ArchiveService {
    db: DatabaseConnection,
}

impl ArchiveService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn build_threads(&self, list_id: &str) -> anyhow::Result<()> {
        let list_uuid = uuid::Uuid::parse_str(list_id)?;

        let messages = email_message::Entity::find()
            .filter(email_message::Column::ListId.eq(list_uuid))
            .filter(email_message::Column::IsDeleted.eq(false))
            .filter(email_message::Column::ThreadId.is_null())
            .all(&self.db)
            .await?;

        for msg in messages {
            let thread_id = if let Some(ref in_reply_to) = msg.in_reply_to {
                self.find_parent_thread(list_uuid, in_reply_to).await?
            } else if let Some(ref references) = msg.references {
                let first_ref = references.split_whitespace().next();
                if let Some(ref_id) = first_ref {
                    self.find_parent_thread(list_uuid, ref_id).await?
                } else {
                    None
                }
            } else {
                None
            };

            let thread_id = thread_id.unwrap_or(msg.id);

            if msg.thread_id != Some(thread_id) {
                let mut active: email_message::ActiveModel = msg.into();
                active.thread_id = Set(Some(thread_id));
                active.update(&self.db).await?;
            }
        }

        Ok(())
    }

    async fn find_parent_thread(
        &self,
        list_id: uuid::Uuid,
        message_id: &str,
    ) -> anyhow::Result<Option<uuid::Uuid>> {
        let parent = email_message::Entity::find()
            .filter(email_message::Column::ListId.eq(list_id))
            .filter(email_message::Column::MessageId.eq(message_id))
            .filter(email_message::Column::IsDeleted.eq(false))
            .one(&self.db)
            .await?;

        Ok(parent.and_then(|p| p.thread_id.or(Some(p.id))))
    }

    pub async fn search(
        &self,
        list_id: &str,
        keyword: &str,
        from: Option<&str>,
    ) -> anyhow::Result<Vec<email_message::Model>> {
        let list_uuid = uuid::Uuid::parse_str(list_id)?;

        let mut query = email_message::Entity::find()
            .filter(email_message::Column::ListId.eq(list_uuid))
            .filter(email_message::Column::IsDeleted.eq(false));

        let pattern = format!("%{}%", keyword);
        query = query.filter(
            email_message::Column::Subject
                .like(&pattern)
                .or(email_message::Column::BodyText.like(&pattern))
                .or(email_message::Column::FromAddr.like(&pattern)),
        );

        if let Some(from_addr) = from {
            query = query.filter(email_message::Column::FromAddr.eq(from_addr));
        }

        let results = query
            .order_by_desc(email_message::Column::ReceivedAt)
            .limit(100)
            .all(&self.db)
            .await?;

        Ok(results)
    }

    pub async fn get_thread_messages(
        &self,
        thread_id: &str,
    ) -> anyhow::Result<Vec<email_message::Model>> {
        let thread_uuid = uuid::Uuid::parse_str(thread_id)?;

        let messages = email_message::Entity::find()
            .filter(email_message::Column::ThreadId.eq(Some(thread_uuid)))
            .filter(email_message::Column::IsDeleted.eq(false))
            .order_by_asc(email_message::Column::ReceivedAt)
            .all(&self.db)
            .await?;

        Ok(messages)
    }
}
