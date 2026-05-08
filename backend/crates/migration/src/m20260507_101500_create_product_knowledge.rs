use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProductKnowledge::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProductKnowledge::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProductKnowledge::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(ProductKnowledge::ProductId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductKnowledge::Title)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ProductKnowledge::ContentType)
                            .string_len(64)
                            .not_null()
                            .default("text"),
                    )
                    .col(ColumnDef::new(ProductKnowledge::Content).text().not_null())
                    .col(
                        ColumnDef::new(ProductKnowledge::SourceType)
                            .string_len(64)
                            .not_null()
                            .default("manual"),
                    )
                    .col(
                        ColumnDef::new(ProductKnowledge::SourceId)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ProductKnowledge::Confidence)
                            .double()
                            .not_null()
                            .default(0.8),
                    )
                    .col(
                        ColumnDef::new(ProductKnowledge::Status)
                            .string_len(32)
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(ProductKnowledge::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_knowledge_user_id")
                            .from(ProductKnowledge::Table, ProductKnowledge::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_knowledge_product_id")
                            .from(ProductKnowledge::Table, ProductKnowledge::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        create_indexes(manager).await?;
        create_updated_at_triggers(manager, &["product_knowledge"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ProductKnowledge::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_index(
            Index::create()
                .name("idx_product_knowledge_product_confidence")
                .table(ProductKnowledge::Table)
                .col(ProductKnowledge::ProductId)
                .col(ProductKnowledge::Confidence)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_knowledge_product_updated")
                .table(ProductKnowledge::Table)
                .col(ProductKnowledge::ProductId)
                .col(ProductKnowledge::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_knowledge_product_type")
                .table(ProductKnowledge::Table)
                .col(ProductKnowledge::ProductId)
                .col(ProductKnowledge::ContentType)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_knowledge_product_status")
                .table(ProductKnowledge::Table)
                .col(ProductKnowledge::ProductId)
                .col(ProductKnowledge::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_product_knowledge_user_updated")
                .table(ProductKnowledge::Table)
                .col(ProductKnowledge::UserId)
                .col(ProductKnowledge::UpdatedAt)
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
enum ProductKnowledge {
    Table,
    Id,
    UserId,
    ProductId,
    Title,
    ContentType,
    Content,
    SourceType,
    SourceId,
    Confidence,
    Status,
    Metadata,
    UpdatedAt,
}
