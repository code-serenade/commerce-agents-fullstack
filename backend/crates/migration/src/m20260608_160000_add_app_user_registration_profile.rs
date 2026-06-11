use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AppUsers::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(AppUsers::SchoolName).string_len(128).null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(AppUsers::TeamName).string_len(128).null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AppUsers::Table)
                    .drop_column(AppUsers::TeamName)
                    .drop_column(AppUsers::SchoolName)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AppUsers {
    Table,
    SchoolName,
    TeamName,
}
