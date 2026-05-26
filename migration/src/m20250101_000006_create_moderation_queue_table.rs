use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ModerationQueue::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ModerationQueue::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ModerationQueue::ListId).uuid().not_null())
                    .col(ColumnDef::new(ModerationQueue::MessageId).uuid())
                    .col(
                        ColumnDef::new(ModerationQueue::FromAddr)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ModerationQueue::Subject).string_len(512))
                    .col(
                        ColumnDef::new(ModerationQueue::Reason)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModerationQueue::Status)
                            .string_len(20)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(ModerationQueue::Source)
                            .string_len(20)
                            .not_null()
                            .default("manual"),
                    )
                    .col(ColumnDef::new(ModerationQueue::AiRiskScore).integer())
                    .col(ColumnDef::new(ModerationQueue::AiLabels).json())
                    .col(ColumnDef::new(ModerationQueue::AiRawResponse).text())
                    .col(
                        ColumnDef::new(ModerationQueue::AiReviewed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(ModerationQueue::ModeratedBy).uuid())
                    .col(ColumnDef::new(ModerationQueue::ModeratedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ModerationQueue::ModerationNote).text())
                    .col(
                        ColumnDef::new(ModerationQueue::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mod_queue_list")
                            .from(ModerationQueue::Table, ModerationQueue::ListId)
                            .to(MailingList::Table, MailingList::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ModerationQueue::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ModerationQueue {
    Table,
    Id,
    ListId,
    MessageId,
    FromAddr,
    Subject,
    Reason,
    Status,
    Source,
    AiRiskScore,
    AiLabels,
    AiRawResponse,
    AiReviewed,
    ModeratedBy,
    ModeratedAt,
    ModerationNote,
    CreatedAt,
}

#[derive(DeriveIden)]
enum MailingList {
    Table,
    Id,
}
