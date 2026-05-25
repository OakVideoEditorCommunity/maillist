use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EmailMessage::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(EmailMessage::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(EmailMessage::ListId).uuid().not_null())
                    .col(ColumnDef::new(EmailMessage::MessageId).string_len(255).not_null())
                    .col(ColumnDef::new(EmailMessage::InReplyTo).string_len(255))
                    .col(ColumnDef::new(EmailMessage::References).text())
                    .col(ColumnDef::new(EmailMessage::FromName).string_len(255))
                    .col(ColumnDef::new(EmailMessage::FromAddr).string_len(255).not_null())
                    .col(ColumnDef::new(EmailMessage::ToAddr).string_len(255))
                    .col(ColumnDef::new(EmailMessage::Subject).string_len(512))
                    .col(ColumnDef::new(EmailMessage::SubjectNormalized).string_len(512))
                    .col(ColumnDef::new(EmailMessage::BodyText).text())
                    .col(ColumnDef::new(EmailMessage::BodyHtml).text())
                    .col(ColumnDef::new(EmailMessage::RawContent).text())
                    .col(ColumnDef::new(EmailMessage::SizeBytes).integer())
                    .col(ColumnDef::new(EmailMessage::HasAttachments).boolean().not_null().default(false))
                    .col(ColumnDef::new(EmailMessage::ReceivedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(EmailMessage::ThreadId).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_message_list")
                            .from(EmailMessage::Table, EmailMessage::ListId)
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
            .drop_table(Table::drop().table(EmailMessage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum EmailMessage {
    Table,
    Id,
    ListId,
    MessageId,
    InReplyTo,
    References,
    FromName,
    FromAddr,
    ToAddr,
    Subject,
    SubjectNormalized,
    BodyText,
    BodyHtml,
    RawContent,
    SizeBytes,
    HasAttachments,
    ReceivedAt,
    ThreadId,
}

#[derive(DeriveIden)]
enum MailingList {
    Table,
    Id,
}
