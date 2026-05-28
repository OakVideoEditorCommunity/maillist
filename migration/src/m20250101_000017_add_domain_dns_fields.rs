use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Domain::Table)
                    .add_column_if_not_exists(ColumnDef::new(Domain::DkimPublicKey).text())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Domain::Table)
                    .add_column_if_not_exists(ColumnDef::new(Domain::SpfRecord).string_len(512))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Domain::Table)
                    .add_column_if_not_exists(ColumnDef::new(Domain::DmarcRecord).string_len(512))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Domain::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Domain::SpfVerified).boolean().default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Domain::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Domain::DkimVerified)
                            .boolean()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Domain::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Domain::DmarcVerified)
                            .boolean()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Domain::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Domain::DkimEnabled).boolean().default(false),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            Domain::DkimEnabled,
            Domain::DmarcVerified,
            Domain::DkimVerified,
            Domain::SpfVerified,
            Domain::DmarcRecord,
            Domain::SpfRecord,
            Domain::DkimPublicKey,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Domain::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Domain {
    Table,
    DkimPublicKey,
    SpfRecord,
    DmarcRecord,
    SpfVerified,
    DkimVerified,
    DmarcVerified,
    DkimEnabled,
}
