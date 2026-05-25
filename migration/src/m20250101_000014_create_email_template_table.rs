use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EmailTemplate::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(EmailTemplate::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(EmailTemplate::Name).string_len(100).not_null().unique_key())
                    .col(ColumnDef::new(EmailTemplate::Subject).string_len(512))
                    .col(ColumnDef::new(EmailTemplate::BodyText).text())
                    .col(ColumnDef::new(EmailTemplate::BodyHtml).text())
                    .col(ColumnDef::new(EmailTemplate::Variables).json())
                    .col(ColumnDef::new(EmailTemplate::IsSystem).boolean().not_null().default(false))
                    .col(ColumnDef::new(EmailTemplate::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(EmailTemplate::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EmailTemplate::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum EmailTemplate {
    Table,
    Id,
    Name,
    Subject,
    BodyText,
    BodyHtml,
    Variables,
    IsSystem,
    CreatedAt,
    UpdatedAt,
}
