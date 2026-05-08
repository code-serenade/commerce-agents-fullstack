use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_product_competitors(manager).await?;
        create_competitor_changes(manager).await?;
        create_updated_at_triggers(manager, &["product_competitors"]).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(CompetitorChanges::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ProductCompetitors::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_product_competitors(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ProductCompetitors::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ProductCompetitors::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(ProductCompetitors::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(ProductCompetitors::ProductId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProductCompetitors::Name)
                        .string_len(255)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProductCompetitors::Platform)
                        .string_len(64)
                        .null(),
                )
                .col(ColumnDef::new(ProductCompetitors::Url).text().null())
                .col(ColumnDef::new(ProductCompetitors::Price).double().null())
                .col(ColumnDef::new(ProductCompetitors::Rating).double().null())
                .col(
                    ColumnDef::new(ProductCompetitors::MonthlySales)
                        .string_len(64)
                        .null(),
                )
                .col(ColumnDef::new(ProductCompetitors::Notes).text().null())
                .col(
                    ColumnDef::new(ProductCompetitors::SearchResults)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(ProductCompetitors::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(ProductCompetitors::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(last_searched_at_col())
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_product_competitors_user_id")
                        .from(ProductCompetitors::Table, ProductCompetitors::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_product_competitors_product_id")
                        .from(ProductCompetitors::Table, ProductCompetitors::ProductId)
                        .to(Products::Table, Products::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_product_competitors_product_status")
                .table(ProductCompetitors::Table)
                .col(ProductCompetitors::ProductId)
                .col(ProductCompetitors::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_competitors_product_platform")
                .table(ProductCompetitors::Table)
                .col(ProductCompetitors::ProductId)
                .col(ProductCompetitors::Platform)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_competitors_product_updated")
                .table(ProductCompetitors::Table)
                .col(ProductCompetitors::ProductId)
                .col(ProductCompetitors::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_competitors_user_updated")
                .table(ProductCompetitors::Table)
                .col(ProductCompetitors::UserId)
                .col(ProductCompetitors::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;

    Ok(())
}

async fn create_competitor_changes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(CompetitorChanges::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(CompetitorChanges::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(CompetitorChanges::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(CompetitorChanges::ProductId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CompetitorChanges::CompetitorId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(CompetitorChanges::ChangeType)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(CompetitorChanges::OldValue).text().null())
                .col(ColumnDef::new(CompetitorChanges::NewValue).text().null())
                .col(ColumnDef::new(CompetitorChanges::Summary).text().null())
                .col(
                    ColumnDef::new(CompetitorChanges::SourceType)
                        .string_len(64)
                        .not_null()
                        .default("manual"),
                )
                .col(
                    ColumnDef::new(CompetitorChanges::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(detected_at_col())
                .col(created_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_competitor_changes_user_id")
                        .from(CompetitorChanges::Table, CompetitorChanges::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_competitor_changes_product_id")
                        .from(CompetitorChanges::Table, CompetitorChanges::ProductId)
                        .to(Products::Table, Products::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_competitor_changes_competitor_id")
                        .from(CompetitorChanges::Table, CompetitorChanges::CompetitorId)
                        .to(ProductCompetitors::Table, ProductCompetitors::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_competitor_changes_product_detected")
                .table(CompetitorChanges::Table)
                .col(CompetitorChanges::ProductId)
                .col(CompetitorChanges::DetectedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_competitor_changes_competitor_detected")
                .table(CompetitorChanges::Table)
                .col(CompetitorChanges::CompetitorId)
                .col(CompetitorChanges::DetectedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_competitor_changes_user_detected")
                .table(CompetitorChanges::Table)
                .col(CompetitorChanges::UserId)
                .col(CompetitorChanges::DetectedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_competitor_changes_product_type")
                .table(CompetitorChanges::Table)
                .col(CompetitorChanges::ProductId)
                .col(CompetitorChanges::ChangeType)
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

fn last_searched_at_col() -> ColumnDef {
    let mut col = ColumnDef::new(ProductCompetitors::LastSearchedAt);
    col.timestamp_with_time_zone().null();
    col
}

fn detected_at_col() -> ColumnDef {
    let mut col = ColumnDef::new(CompetitorChanges::DetectedAt);
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
}

#[derive(DeriveIden)]
enum ProductCompetitors {
    Table,
    Id,
    UserId,
    ProductId,
    Name,
    Platform,
    Url,
    Price,
    Rating,
    MonthlySales,
    Notes,
    SearchResults,
    Status,
    Metadata,
    LastSearchedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum CompetitorChanges {
    Table,
    Id,
    UserId,
    ProductId,
    CompetitorId,
    ChangeType,
    OldValue,
    NewValue,
    Summary,
    SourceType,
    Metadata,
    DetectedAt,
}
