use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sender_policy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub list_id: Option<Uuid>,
    pub email_pattern: String,
    pub policy_type: String,
    pub scope: String,
    pub note: Option<String>,
    pub created_by: Option<Uuid>,
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
}

impl ActiveModelBehavior for ActiveModel {}
