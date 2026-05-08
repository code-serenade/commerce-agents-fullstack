use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Products::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Products::UserId).uuid().not_null())
                    .col(ColumnDef::new(Products::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Products::Platform).string_len(64).null())
                    .col(ColumnDef::new(Products::ExternalId).string_len(128).null())
                    .col(ColumnDef::new(Products::Category).string_len(128).null())
                    .col(ColumnDef::new(Products::Brand).string_len(128).null())
                    .col(
                        ColumnDef::new(Products::Status)
                            .string_len(32)
                            .not_null()
                            .default("active"),
                    )
                    .col(ColumnDef::new(Products::Description).text().null())
                    .col(ColumnDef::new(Products::Price).string_len(64).null())
                    .col(ColumnDef::new(Products::Currency).string_len(16).null())
                    .col(
                        ColumnDef::new(Products::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_products_user_id")
                            .from(Products::Table, Products::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        create_indexes(manager).await?;
        create_updated_at_triggers(manager, &["products"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Products::Table).cascade().to_owned())
            .await
    }
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_index(
            Index::create()
                .name("idx_products_user_updated")
                .table(Products::Table)
                .col(Products::UserId)
                .col(Products::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_products_user_status")
                .table(Products::Table)
                .col(Products::UserId)
                .col(Products::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_products_user_platform_external")
                .table(Products::Table)
                .col(Products::UserId)
                .col(Products::Platform)
                .col(Products::ExternalId)
                .if_not_exists()
                .to_owned(),
        )
        .await?;

    Ok(())
}

fn created_at_col() -> ColumnDef {
    let mut col = ColumnDef::new(AuditColumns::CreatedAt);
    col.timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp());
    col
}

fn updated_at_col() -> ColumnDef {
    let mut col = ColumnDef::new(AuditColumns::UpdatedAt);
    col.timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp());
    col
}

async fn create_updated_at_triggers(
    manager: &SchemaManager<'_>,
    table_names: &[&str],
) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            r#"
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
"#,
        )
        .await?;

    for table_name in table_names {
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"
DROP TRIGGER IF EXISTS trg_{table_name}_updated_at ON {table_name};
CREATE TRIGGER trg_{table_name}_updated_at
BEFORE UPDATE ON {table_name}
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
"#
            ))
            .await?;
    }

    Ok(())
}

#[derive(DeriveIden)]
enum AuditColumns {
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AppUsers {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    UserId,
    Name,
    Platform,
    ExternalId,
    Category,
    Brand,
    Status,
    Description,
    Price,
    Currency,
    Metadata,
    UpdatedAt,
}
