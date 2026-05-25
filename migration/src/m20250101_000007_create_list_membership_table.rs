use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ListMembership::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ListMembership::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ListMembership::UserId).uuid().not_null())
                    .col(ColumnDef::new(ListMembership::ListId).uuid().not_null())
                    .col(ColumnDef::new(ListMembership::Role).string_len(20).not_null().default("subscriber"))
                    .col(ColumnDef::new(ListMembership::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_membership_user")
                            .from(ListMembership::Table, ListMembership::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_membership_list")
                            .from(ListMembership::Table, ListMembership::ListId)
                            .to(MailingList::Table, MailingList::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ListMembership::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ListMembership {
    Table,
    Id,
    UserId,
    ListId,
    Role,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum MailingList {
    Table,
    Id,
}
