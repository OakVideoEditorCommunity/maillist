use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "domain")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i32>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub dkim_selector: Option<String>,
    pub dkim_private_key: Option<String>,
    pub dkim_public_key: Option<String>,
    pub spf_record: Option<String>,
    pub dmarc_record: Option<String>,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    pub dkim_enabled: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
