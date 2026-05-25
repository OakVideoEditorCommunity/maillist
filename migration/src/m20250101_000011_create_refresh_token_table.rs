use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RefreshToken::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RefreshToken::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(RefreshToken::UserId).uuid().not_null())
                    .col(ColumnDef::new(RefreshToken::TokenHash).string_len(255).not_null().unique_key())
                    .col(ColumnDef::new(RefreshToken::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(RefreshToken::RevokedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(RefreshToken::IpAddress).string_len(45))
                    .col(ColumnDef::new(RefreshToken::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_refresh_token_user")
                            .from(RefreshToken::Table, RefreshToken::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RefreshToken {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
    IpAddress,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
