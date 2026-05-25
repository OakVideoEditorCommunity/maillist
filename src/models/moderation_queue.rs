use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "moderation_queue")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub list_id: Uuid,
    pub message_id: Option<Uuid>,
    pub from_addr: String,
    pub subject: Option<String>,
    pub reason: String,
    pub status: String,
    pub source: String,
    pub ai_risk_score: Option<i32>,
    pub ai_labels: Option<Json>,
    pub ai_raw_response: Option<String>,
    pub ai_reviewed: bool,
    pub moderated_by: Option<Uuid>,
    pub moderated_at: Option<DateTimeWithTimeZone>,
    pub moderation_note: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mailing_list::Entity",
        from = "Column::ListId",
        to = "super::mailing_list::Column::Id"
    )]
    MailingList,
    #[sea_orm(
        belongs_to = "super::email_message::Entity",
        from = "Column::MessageId",
        to = "super::email_message::Column::Id"
    )]
    EmailMessage,
}

impl Related<super::mailing_list::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MailingList.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
