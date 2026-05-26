use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SenderPolicy::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SenderPolicy::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SenderPolicy::ListId).uuid())
                    .col(
                        ColumnDef::new(SenderPolicy::EmailPattern)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SenderPolicy::PolicyType)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SenderPolicy::Scope)
                            .string_len(20)
                            .not_null()
                            .default("post"),
                    )
                    .col(ColumnDef::new(SenderPolicy::Note).string_len(255))
                    .col(ColumnDef::new(SenderPolicy::CreatedBy).uuid())
                    .col(
                        ColumnDef::new(SenderPolicy::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sender_policy_list")
                            .from(SenderPolicy::Table, SenderPolicy::ListId)
                            .to(MailingList::Table, MailingList::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SenderPolicy::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SenderPolicy {
    Table,
    Id,
    ListId,
    EmailPattern,
    PolicyType,
    Scope,
    Note,
    CreatedBy,
    CreatedAt,
}

#[derive(DeriveIden)]
enum MailingList {
    Table,
    Id,
}
