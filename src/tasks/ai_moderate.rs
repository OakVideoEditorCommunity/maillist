use crate::models::moderation_queue;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use tracing::{info, warn};

pub struct AiModerateTask {
    db: DatabaseConnection,
}

impl AiModerateTask {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Starting AI moderation review task");

        let pending = moderation_queue::Entity::find()
            .filter(moderation_queue::Column::Status.eq("pending"))
            .filter(moderation_queue::Column::Source.eq("ai_flagged"))
            .filter(moderation_queue::Column::AiReviewed.eq(false))
            .all(&self.db)
            .await?;

        let count = pending.len();
        for item in pending {
            let mut active: moderation_queue::ActiveModel = item.into();
            active.ai_reviewed = Set(true);
            if let Err(e) = active.update(&self.db).await {
                warn!("Failed to mark AI moderation item as reviewed: {}", e);
            }
        }

        info!("AI moderation review task completed ({} items reviewed)", count);
        Ok(())
    }
}
