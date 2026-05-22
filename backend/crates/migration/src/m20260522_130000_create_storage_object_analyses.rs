use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StorageObjectAnalyses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::StorageObjectId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::AnalysisKind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::Status)
                            .string_len(32)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::PromptKey)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::PromptVersion)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::LlmName)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::LlmModel)
                            .string_len(128)
                            .null(),
                    )
                    .col(ColumnDef::new(StorageObjectAnalyses::Summary).text().null())
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::ResultJson)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::ErrorMessage)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::StartedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StorageObjectAnalyses::FinishedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_storage_object_analyses_object_id")
                            .from(
                                StorageObjectAnalyses::Table,
                                StorageObjectAnalyses::StorageObjectId,
                            )
                            .to(StorageObjects::Table, StorageObjects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uk_storage_object_analyses_identity")
                    .table(StorageObjectAnalyses::Table)
                    .col(StorageObjectAnalyses::StorageObjectId)
                    .col(StorageObjectAnalyses::AnalysisKind)
                    .col(StorageObjectAnalyses::PromptKey)
                    .col(StorageObjectAnalyses::PromptVersion)
                    .col(StorageObjectAnalyses::LlmName)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_storage_object_analyses_status_updated")
                    .table(StorageObjectAnalyses::Table)
                    .col(StorageObjectAnalyses::Status)
                    .col(StorageObjectAnalyses::UpdatedAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_storage_object_analyses_object_status")
                    .table(StorageObjectAnalyses::Table)
                    .col(StorageObjectAnalyses::StorageObjectId)
                    .col(StorageObjectAnalyses::Status)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        create_updated_at_trigger(manager, "storage_object_analyses").await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(StorageObjectAnalyses::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
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

async fn create_updated_at_trigger(
    manager: &SchemaManager<'_>,
    table_name: &str,
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
    Ok(())
}

#[derive(DeriveIden)]
enum AuditColumns {
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum StorageObjects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum StorageObjectAnalyses {
    Table,
    Id,
    StorageObjectId,
    AnalysisKind,
    Status,
    PromptKey,
    PromptVersion,
    LlmName,
    LlmModel,
    Summary,
    ResultJson,
    ErrorMessage,
    StartedAt,
    FinishedAt,
    UpdatedAt,
}
