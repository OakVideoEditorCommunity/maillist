use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuthSession::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AuthSession::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AuthSession::UserId).uuid().not_null())
                    .col(ColumnDef::new(AuthSession::SessionToken).string_len(255).not_null().unique_key())
                    .col(ColumnDef::new(AuthSession::Step).string_len(20).not_null().default("password"))
                    .col(ColumnDef::new(AuthSession::MfaType).string_len(20))
                    .col(ColumnDef::new(AuthSession::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AuthSession::IpAddress).string_len(45))
                    .col(ColumnDef::new(AuthSession::UserAgent).string_len(512))
                    .col(ColumnDef::new(AuthSession::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_auth_session_user")
                            .from(AuthSession::Table, AuthSession::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuthSession::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuthSession {
    Table,
    Id,
    UserId,
    SessionToken,
    Step,
    MfaType,
    ExpiresAt,
    IpAddress,
    UserAgent,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
