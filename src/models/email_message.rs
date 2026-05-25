use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "email_message")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub list_id: Uuid,
    pub message_id: String,
    pub in_reply_to: Option<String>,
    pub references: Option<String>,
    pub from_name: Option<String>,
    pub from_addr: String,
    pub to_addr: Option<String>,
    pub subject: Option<String>,
    pub subject_normalized: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub raw_content: Option<String>,
    pub size_bytes: Option<i32>,
    pub has_attachments: bool,
    pub received_at: DateTimeWithTimeZone,
    pub thread_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mailing_list::Entity",
        from = "Column::ListId",
        to = "super::mailing_list::Column::Id"
    )]
    MailingList,
}

impl Related<super::mailing_list::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MailingList.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
