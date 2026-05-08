use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_product_assets(manager).await?;
        create_product_materials(manager).await?;
        create_updated_at_triggers(manager, &["product_assets", "product_materials"]).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ProductMaterials::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ProductAssets::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_product_assets(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ProductAssets::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ProductAssets::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(ProductAssets::UserId).uuid().not_null())
                .col(ColumnDef::new(ProductAssets::ProductId).uuid().not_null())
                .col(
                    ColumnDef::new(ProductAssets::Bucket)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProductAssets::ObjectKey)
                        .string_len(1024)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProductAssets::AssetType)
                        .string_len(64)
                        .not_null()
                        .default("docs"),
                )
                .col(
                    ColumnDef::new(ProductAssets::FileName)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(ProductAssets::MimeType)
                        .string_len(128)
                        .null(),
                )
                .col(ColumnDef::new(ProductAssets::FileSize).big_integer().null())
                .col(ColumnDef::new(ProductAssets::Description).text().null())
                .col(
                    ColumnDef::new(ProductAssets::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(ProductAssets::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_product_assets_user_id")
                        .from(ProductAssets::Table, ProductAssets::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_product_assets_product_id")
                        .from(ProductAssets::Table, ProductAssets::ProductId)
                        .to(Products::Table, Products::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_product_assets_product_status")
                .table(ProductAssets::Table)
                .col(ProductAssets::ProductId)
                .col(ProductAssets::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_assets_product_type")
                .table(ProductAssets::Table)
                .col(ProductAssets::ProductId)
                .col(ProductAssets::AssetType)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_assets_product_updated")
                .table(ProductAssets::Table)
                .col(ProductAssets::ProductId)
                .col(ProductAssets::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_assets_user_updated")
                .table(ProductAssets::Table)
                .col(ProductAssets::UserId)
                .col(ProductAssets::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_assets_user_object_key")
                .table(ProductAssets::Table)
                .col(ProductAssets::UserId)
                .col(ProductAssets::ObjectKey)
                .unique()
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    Ok(())
}

async fn create_product_materials(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ProductMaterials::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ProductMaterials::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(ProductMaterials::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(ProductMaterials::ProductId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProductMaterials::MaterialType)
                        .string_len(64)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ProductMaterials::Title)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(ProductMaterials::Version)
                        .integer()
                        .not_null()
                        .default(1),
                )
                .col(
                    ColumnDef::new(ProductMaterials::Content)
                        .text()
                        .not_null()
                        .default(""),
                )
                .col(
                    ColumnDef::new(ProductMaterials::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(ProductMaterials::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_product_materials_user_id")
                        .from(ProductMaterials::Table, ProductMaterials::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_product_materials_product_id")
                        .from(ProductMaterials::Table, ProductMaterials::ProductId)
                        .to(Products::Table, Products::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_product_materials_product_status")
                .table(ProductMaterials::Table)
                .col(ProductMaterials::ProductId)
                .col(ProductMaterials::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_materials_product_type_version")
                .table(ProductMaterials::Table)
                .col(ProductMaterials::ProductId)
                .col(ProductMaterials::MaterialType)
                .col(ProductMaterials::Version)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_materials_product_updated")
                .table(ProductMaterials::Table)
                .col(ProductMaterials::ProductId)
                .col(ProductMaterials::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_materials_user_updated")
                .table(ProductMaterials::Table)
                .col(ProductMaterials::UserId)
                .col(ProductMaterials::UpdatedAt)
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
}

#[derive(DeriveIden)]
enum ProductAssets {
    Table,
    Id,
    UserId,
    ProductId,
    Bucket,
    ObjectKey,
    AssetType,
    FileName,
    MimeType,
    FileSize,
    Description,
    Status,
    Metadata,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProductMaterials {
    Table,
    Id,
    UserId,
    ProductId,
    MaterialType,
    Title,
    Version,
    Content,
    Status,
    Metadata,
    UpdatedAt,
}
