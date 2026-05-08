use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Campaigns::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Campaigns::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Campaigns::UserId).uuid().not_null())
                    .col(ColumnDef::new(Campaigns::ProductId).uuid().null())
                    .col(ColumnDef::new(Campaigns::Name).string_len(255).not_null())
                    .col(
                        ColumnDef::new(Campaigns::CampaignType)
                            .string_len(64)
                            .null(),
                    )
                    .col(ColumnDef::new(Campaigns::Description).text().null())
                    .col(
                        ColumnDef::new(Campaigns::Budget)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .col(ColumnDef::new(Campaigns::StartDate).string_len(32).null())
                    .col(ColumnDef::new(Campaigns::EndDate).string_len(32).null())
                    .col(
                        ColumnDef::new(Campaigns::Status)
                            .string_len(32)
                            .not_null()
                            .default("draft"),
                    )
                    .col(
                        ColumnDef::new(Campaigns::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(
                        ColumnDef::new(Campaigns::Output)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(
                        ColumnDef::new(Campaigns::ActionMetadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_campaigns_user_id")
                            .from(Campaigns::Table, Campaigns::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_campaigns_product_id")
                            .from(Campaigns::Table, Campaigns::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        create_indexes(manager).await?;
        create_updated_at_triggers(manager, &["campaigns"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Campaigns::Table).cascade().to_owned())
            .await
    }
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_index(
            Index::create()
                .name("idx_campaigns_user_status")
                .table(Campaigns::Table)
                .col(Campaigns::UserId)
                .col(Campaigns::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_campaigns_user_updated")
                .table(Campaigns::Table)
                .col(Campaigns::UserId)
                .col(Campaigns::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_campaigns_product_status")
                .table(Campaigns::Table)
                .col(Campaigns::ProductId)
                .col(Campaigns::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_campaigns_user_type")
                .table(Campaigns::Table)
                .col(Campaigns::UserId)
                .col(Campaigns::CampaignType)
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
enum Campaigns {
    Table,
    Id,
    UserId,
    ProductId,
    Name,
    CampaignType,
    Description,
    Budget,
    StartDate,
    EndDate,
    Status,
    Metadata,
    Output,
    ActionMetadata,
    UpdatedAt,
}
