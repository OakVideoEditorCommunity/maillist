pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_domain_table;
mod m20250101_000002_create_mailing_list_table;
mod m20250101_000003_create_user_table;
mod m20250101_000004_create_subscriber_table;
mod m20250101_000005_create_email_message_table;
mod m20250101_000006_create_moderation_queue_table;
mod m20250101_000007_create_list_membership_table;
mod m20250101_000008_create_totp_credential_table;
mod m20250101_000009_create_passkey_credential_table;
mod m20250101_000010_create_auth_session_table;
mod m20250101_000011_create_refresh_token_table;
mod m20250101_000012_create_sender_policy_table;
mod m20250101_000013_create_attachment_table;
mod m20250101_000014_create_email_template_table;
mod m20250101_000015_create_bounce_log_table;
mod m20250101_000016_seed_email_templates;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_domain_table::Migration),
            Box::new(m20250101_000002_create_mailing_list_table::Migration),
            Box::new(m20250101_000003_create_user_table::Migration),
            Box::new(m20250101_000004_create_subscriber_table::Migration),
            Box::new(m20250101_000005_create_email_message_table::Migration),
            Box::new(m20250101_000006_create_moderation_queue_table::Migration),
            Box::new(m20250101_000007_create_list_membership_table::Migration),
            Box::new(m20250101_000008_create_totp_credential_table::Migration),
            Box::new(m20250101_000009_create_passkey_credential_table::Migration),
            Box::new(m20250101_000010_create_auth_session_table::Migration),
            Box::new(m20250101_000011_create_refresh_token_table::Migration),
            Box::new(m20250101_000012_create_sender_policy_table::Migration),
            Box::new(m20250101_000013_create_attachment_table::Migration),
            Box::new(m20250101_000014_create_email_template_table::Migration),
            Box::new(m20250101_000015_create_bounce_log_table::Migration),
            Box::new(m20250101_000016_seed_email_templates::Migration),
        ]
    }
}
