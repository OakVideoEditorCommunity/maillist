use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "subscriber")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub list_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub status: String,
    pub digest_mode: String,
    pub subscribe_ip: Option<String>,
    pub subscribe_source: Option<String>,
    pub bounce_count: i32,
    pub last_bounce_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(unique)]
    pub token: String,
    pub confirmed_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
