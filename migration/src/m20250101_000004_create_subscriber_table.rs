use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscriber::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Subscriber::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Subscriber::ListId).uuid().not_null())
                    .col(ColumnDef::new(Subscriber::Email).string_len(255).not_null())
                    .col(ColumnDef::new(Subscriber::Name).string_len(255))
                    .col(
                        ColumnDef::new(Subscriber::Status)
                            .string_len(20)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Subscriber::DigestMode)
                            .string_len(20)
                            .not_null()
                            .default("none"),
                    )
                    .col(ColumnDef::new(Subscriber::SubscribeIp).string_len(45))
                    .col(ColumnDef::new(Subscriber::SubscribeSource).string_len(50))
                    .col(
                        ColumnDef::new(Subscriber::BounceCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Subscriber::LastBounceAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Subscriber::Token)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Subscriber::ConfirmedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Subscriber::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscriber::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscriber_list")
                            .from(Subscriber::Table, Subscriber::ListId)
                            .to(MailingList::Table, MailingList::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriber::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Subscriber {
    Table,
    Id,
    ListId,
    Email,
    Name,
    Status,
    DigestMode,
    SubscribeIp,
    SubscribeSource,
    BounceCount,
    LastBounceAt,
    Token,
    ConfirmedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum MailingList {
    Table,
    Id,
}
