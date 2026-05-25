use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Attachment::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Attachment::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Attachment::MessageId).uuid().not_null())
                    .col(ColumnDef::new(Attachment::Filename).string_len(255))
                    .col(ColumnDef::new(Attachment::ContentType).string_len(100))
                    .col(ColumnDef::new(Attachment::SizeBytes).big_integer())
                    .col(ColumnDef::new(Attachment::StoragePath).string_len(512))
                    .col(ColumnDef::new(Attachment::ContentId).string_len(255))
                    .col(ColumnDef::new(Attachment::ChecksumSha256).string_len(64))
                    .col(ColumnDef::new(Attachment::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_attachment_message")
                            .from(Attachment::Table, Attachment::MessageId)
                            .to(EmailMessage::Table, EmailMessage::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Attachment::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Attachment {
    Table,
    Id,
    MessageId,
    Filename,
    ContentType,
    SizeBytes,
    StoragePath,
    ContentId,
    ChecksumSha256,
    CreatedAt,
}

#[derive(DeriveIden)]
enum EmailMessage {
    Table,
    Id,
}
