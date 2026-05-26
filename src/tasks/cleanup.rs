use crate::models::{auth_session, bounce_log, email_message, moderation_queue, refresh_token};
use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::info;

pub struct CleanupTask {
    db: DatabaseConnection,
}

impl CleanupTask {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Starting cleanup task");

        let cutoff = Utc::now() - Duration::days(30);

        let deleted = email_message::Entity::delete_many()
            .filter(email_message::Column::IsDeleted.eq(true))
            .filter(email_message::Column::DeletedAt.lt(cutoff))
            .exec(&self.db)
            .await?;
        info!("Hard-deleted {} old email messages", deleted.rows_affected);

        let mq_deleted = moderation_queue::Entity::delete_many()
            .filter(moderation_queue::Column::Status.is_in(["rejected", "discarded"]))
            .filter(moderation_queue::Column::CreatedAt.lt(cutoff))
            .exec(&self.db)
            .await?;
        info!(
            "Deleted {} old moderation queue entries",
            mq_deleted.rows_affected
        );

        let session_deleted = auth_session::Entity::delete_many()
            .filter(auth_session::Column::ExpiresAt.lt(Utc::now()))
            .exec(&self.db)
            .await?;
        info!(
            "Deleted {} expired auth sessions",
            session_deleted.rows_affected
        );

        let token_deleted = refresh_token::Entity::delete_many()
            .filter(refresh_token::Column::ExpiresAt.lt(Utc::now()))
            .exec(&self.db)
            .await?;
        info!(
            "Deleted {} expired refresh tokens",
            token_deleted.rows_affected
        );

        let bounce_deleted = bounce_log::Entity::delete_many()
            .filter(bounce_log::Column::CreatedAt.lt(cutoff))
            .exec(&self.db)
            .await?;
        info!("Deleted {} old bounce logs", bounce_deleted.rows_affected);

        Ok(())
    }
}
