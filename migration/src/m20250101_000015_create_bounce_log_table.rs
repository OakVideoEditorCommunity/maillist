use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BounceLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BounceLog::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BounceLog::SubscriberId).uuid())
                    .col(ColumnDef::new(BounceLog::MessageId).uuid())
                    .col(
                        ColumnDef::new(BounceLog::BounceType)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(BounceLog::BounceReason).text())
                    .col(ColumnDef::new(BounceLog::DiagnosticCode).string_len(255))
                    .col(ColumnDef::new(BounceLog::RemoteMta).string_len(255))
                    .col(
                        ColumnDef::new(BounceLog::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bounce_subscriber")
                            .from(BounceLog::Table, BounceLog::SubscriberId)
                            .to(Subscriber::Table, Subscriber::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BounceLog::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BounceLog {
    Table,
    Id,
    SubscriberId,
    MessageId,
    BounceType,
    BounceReason,
    DiagnosticCode,
    RemoteMta,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Subscriber {
    Table,
    Id,
}
