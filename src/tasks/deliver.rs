use crate::models::{email_message, moderation_queue, subscriber};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{error, info};

pub struct DeliverTask {
    db: DatabaseConnection,
}

impl DeliverTask {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Starting delivery task");

        let approved = moderation_queue::Entity::find()
            .filter(moderation_queue::Column::Status.eq("approved"))
            .filter(moderation_queue::Column::MessageId.is_null())
            .all(&self.db)
            .await?;

        for item in approved {
            let msg = email_message::ActiveModel {
                id: Set(crate::utils::crypto::generate_uuid()),
                list_id: Set(item.list_id),
                message_id: Set(format!("<moderated-{}@oak-maillist>", item.id)),
                in_reply_to: Set(None),
                references: Set(None),
                from_name: Set(None),
                from_addr: Set(item.from_addr.clone()),
                to_addr: Set(None),
                subject: Set(item.subject.clone()),
                subject_normalized: Set(None),
                body_text: Set(None),
                body_html: Set(None),
                raw_content: Set(None),
                size_bytes: Set(None),
                has_attachments: Set(false),
                received_at: Set(Utc::now().into()),
                thread_id: Set(None),
                is_deleted: Set(false),
                deleted_at: Set(None),
                deleted_by: Set(None),
                deleted_reason: Set(None),
            };

            match msg.insert(&self.db).await {
                Ok(created) => {
                    let mut active: moderation_queue::ActiveModel = item.into();
                    active.message_id = Set(Some(created.id));
                    if let Err(e) = active.update(&self.db).await {
                        error!("Failed to update moderation_queue message_id: {}", e);
                    } else {
                        info!("Delivered moderated message {}", created.id);
                    }
                }
                Err(e) => {
                    error!("Failed to insert email_message for moderation item: {}", e);
                }
            }
        }

        info!("Delivery task completed");
        Ok(())
    }
}
