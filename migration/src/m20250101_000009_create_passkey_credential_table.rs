use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PasskeyCredential::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PasskeyCredential::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PasskeyCredential::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(PasskeyCredential::CredentialId)
                            .binary()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(PasskeyCredential::PublicKey)
                            .binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PasskeyCredential::SignCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(PasskeyCredential::Aaguid).binary())
                    .col(ColumnDef::new(PasskeyCredential::DeviceName).string_len(255))
                    .col(ColumnDef::new(PasskeyCredential::Transports).json())
                    .col(
                        ColumnDef::new(PasskeyCredential::IsBackupEligible)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(PasskeyCredential::IsBackup)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(PasskeyCredential::LastUsedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(PasskeyCredential::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_passkey_user")
                            .from(PasskeyCredential::Table, PasskeyCredential::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PasskeyCredential::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PasskeyCredential {
    Table,
    Id,
    UserId,
    CredentialId,
    PublicKey,
    SignCount,
    Aaguid,
    DeviceName,
    Transports,
    IsBackupEligible,
    IsBackup,
    LastUsedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
