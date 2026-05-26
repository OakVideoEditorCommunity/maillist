use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TotpCredential::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TotpCredential::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TotpCredential::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(TotpCredential::Secret)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(TotpCredential::Issuer).string_len(255))
                    .col(ColumnDef::new(TotpCredential::AccountName).string_len(255))
                    .col(
                        ColumnDef::new(TotpCredential::Algorithm)
                            .string_len(10)
                            .not_null()
                            .default("SHA1"),
                    )
                    .col(
                        ColumnDef::new(TotpCredential::Digits)
                            .integer()
                            .not_null()
                            .default(6),
                    )
                    .col(
                        ColumnDef::new(TotpCredential::Period)
                            .integer()
                            .not_null()
                            .default(30),
                    )
                    .col(
                        ColumnDef::new(TotpCredential::Verified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(TotpCredential::BackupCodes).json())
                    .col(
                        ColumnDef::new(TotpCredential::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TotpCredential::LastUsedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_totp_user")
                            .from(TotpCredential::Table, TotpCredential::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TotpCredential::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TotpCredential {
    Table,
    Id,
    UserId,
    Secret,
    Issuer,
    AccountName,
    Algorithm,
    Digits,
    Period,
    Verified,
    BackupCodes,
    CreatedAt,
    LastUsedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
