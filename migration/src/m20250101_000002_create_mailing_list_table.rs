use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MailingList::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MailingList::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(MailingList::DomainId).uuid().not_null())
                    .col(ColumnDef::new(MailingList::Name).string_len(100).not_null())
                    .col(ColumnDef::new(MailingList::DisplayName).string_len(255))
                    .col(ColumnDef::new(MailingList::EmailLocalPart).string_len(100).not_null())
                    .col(ColumnDef::new(MailingList::Description).text())
                    .col(ColumnDef::new(MailingList::Visibility).string_len(20).not_null().default("public"))
                    .col(ColumnDef::new(MailingList::SubscriptionPolicy).string_len(20).not_null().default("confirm"))
                    .col(ColumnDef::new(MailingList::PostPolicy).string_len(20).not_null().default("subscriber_only"))
                    .col(ColumnDef::new(MailingList::ReplyTo).string_len(20).not_null().default("list"))
                    .col(ColumnDef::new(MailingList::ArchiveEnabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(MailingList::ArchiveVisibility).string_len(20).not_null().default("public"))
                    .col(ColumnDef::new(MailingList::MaxMessageSizeKb).integer().not_null().default(1024))
                    .col(ColumnDef::new(MailingList::DigestEnabled).boolean().not_null().default(false))
                    .col(ColumnDef::new(MailingList::HeaderTemplate).text())
                    .col(ColumnDef::new(MailingList::FooterTemplate).text())
                    .col(ColumnDef::new(MailingList::AiModerationEnabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(MailingList::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(MailingList::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(MailingList::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mailing_list_domain")
                            .from(MailingList::Table, MailingList::DomainId)
                            .to(Domain::Table, Domain::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MailingList::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MailingList {
    Table,
    Id,
    DomainId,
    Name,
    DisplayName,
    EmailLocalPart,
    Description,
    Visibility,
    SubscriptionPolicy,
    PostPolicy,
    ReplyTo,
    ArchiveEnabled,
    ArchiveVisibility,
    MaxMessageSizeKb,
    DigestEnabled,
    HeaderTemplate,
    FooterTemplate,
    AiModerationEnabled,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Domain {
    Table,
    Id,
}
