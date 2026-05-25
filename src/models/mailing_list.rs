use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mailing_list")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub domain_id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub email_local_part: String,
    pub description: Option<String>,
    pub visibility: String,
    pub subscription_policy: String,
    pub post_policy: String,
    pub reply_to: String,
    pub archive_enabled: bool,
    pub archive_visibility: String,
    pub max_message_size_kb: i32,
    pub digest_enabled: bool,
    pub header_template: Option<String>,
    pub footer_template: Option<String>,
    pub ai_moderation_enabled: bool,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::domain::Entity",
        from = "Column::DomainId",
        to = "super::domain::Column::Id"
    )]
    Domain,
}

impl Related<super::domain::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Domain.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
