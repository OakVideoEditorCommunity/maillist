use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Domain::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Domain::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Domain::Name)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Domain::SmtpHost).string_len(255))
                    .col(ColumnDef::new(Domain::SmtpPort).integer())
                    .col(ColumnDef::new(Domain::SmtpUsername).string_len(255))
                    .col(ColumnDef::new(Domain::SmtpPassword).string_len(255))
                    .col(ColumnDef::new(Domain::DkimSelector).string_len(255))
                    .col(ColumnDef::new(Domain::DkimPrivateKey).text())
                    .col(
                        ColumnDef::new(Domain::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Domain::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Domain::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Domain {
    Table,
    Id,
    Name,
    SmtpHost,
    SmtpPort,
    SmtpUsername,
    SmtpPassword,
    DkimSelector,
    DkimPrivateKey,
    CreatedAt,
    UpdatedAt,
}
