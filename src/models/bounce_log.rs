use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bounce_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub subscriber_id: Option<Uuid>,
    pub message_id: Option<Uuid>,
    pub bounce_type: String,
    pub bounce_reason: Option<String>,
    pub diagnostic_code: Option<String>,
    pub remote_mta: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscriber::Entity",
        from = "Column::SubscriberId",
        to = "super::subscriber::Column::Id"
    )]
    Subscriber,
    #[sea_orm(
        belongs_to = "super::email_message::Entity",
        from = "Column::MessageId",
        to = "super::email_message::Column::Id"
    )]
    EmailMessage,
}

impl ActiveModelBehavior for ActiveModel {}
