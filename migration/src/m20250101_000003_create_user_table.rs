use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(User::Email)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(User::PasswordHash).string_len(255))
                    .col(ColumnDef::new(User::Name).string_len(255))
                    .col(ColumnDef::new(User::AvatarUrl).string_len(512))
                    .col(
                        ColumnDef::new(User::Timezone)
                            .string_len(50)
                            .not_null()
                            .default("Asia/Shanghai"),
                    )
                    .col(
                        ColumnDef::new(User::Language)
                            .string_len(10)
                            .not_null()
                            .default("zh-CN"),
                    )
                    .col(
                        ColumnDef::new(User::IsSiteAdmin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(User::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(User::MfaEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(User::LastLoginAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    PasswordHash,
    Name,
    AvatarUrl,
    Timezone,
    Language,
    IsSiteAdmin,
    IsActive,
    MfaEnabled,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
}
