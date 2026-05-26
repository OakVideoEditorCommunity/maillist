use crate::models::{bounce_log, email_message, subscriber};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{error, info, warn};

pub struct BounceProcessor {
    db: DatabaseConnection,
}

impl BounceProcessor {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn process_bounce(
        &self,
        verp_token: &str,
        bounce_type: &str,
        reason: &str,
    ) -> anyhow::Result<()> {
        info!(
            "Processing bounce for token: {}, type: {}",
            verp_token, bounce_type
        );

        let sub = subscriber::Entity::find()
            .filter(subscriber::Column::Token.eq(verp_token))
            .one(&self.db)
            .await?;

        let Some(sub) = sub else {
            warn!("Bounce for unknown subscriber token: {}", verp_token);
            return Ok(());
        };

        let bounce = bounce_log::ActiveModel {
            id: Set(crate::utils::crypto::generate_uuid()),
            subscriber_id: Set(Some(sub.id)),
            message_id: Set(None),
            bounce_type: Set(bounce_type.to_string()),
            bounce_reason: Set(Some(reason.to_string())),
            diagnostic_code: Set(None),
            remote_mta: Set(None),
            created_at: Set(Utc::now().into()),
        };
        bounce.insert(&self.db).await?;

        match bounce_type {
            "hard" => {
                let new_count = sub.bounce_count + 1;
                if new_count >= 3 {
                    info!(
                        "Subscriber {} reached bounce threshold ({}), unsubscribing",
                        sub.email, new_count
                    );
                    let mut active: subscriber::ActiveModel = sub.into();
                    active.status = Set("unsubscribed".to_string());
                    active.bounce_count = Set(new_count);
                    active.last_bounce_at = Set(Some(Utc::now().into()));
                    active.update(&self.db).await?;
                } else {
                    let mut active: subscriber::ActiveModel = sub.into();
                    active.bounce_count = Set(new_count);
                    active.last_bounce_at = Set(Some(Utc::now().into()));
                    active.update(&self.db).await?;
                }
            }
            "soft" => {
                let new_count = sub.bounce_count + 1;
                let mut active: subscriber::ActiveModel = sub.into();
                active.bounce_count = Set(new_count);
                active.last_bounce_at = Set(Some(Utc::now().into()));
                active.update(&self.db).await?;
            }
            _ => {
                warn!("Unknown bounce type: {}", bounce_type);
            }
        }

        Ok(())
    }
}
