use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_workspace_artifacts(manager).await?;
        create_updated_at_triggers(manager, &["workspace_artifacts"]).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(WorkspaceArtifacts::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_workspace_artifacts(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(WorkspaceArtifacts::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(WorkspaceArtifacts::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::WorkspaceId)
                        .uuid()
                        .not_null(),
                )
                .col(ColumnDef::new(WorkspaceArtifacts::RunId).uuid().null())
                .col(
                    ColumnDef::new(WorkspaceArtifacts::WorkspaceTaskId)
                        .uuid()
                        .null(),
                )
                .col(ColumnDef::new(WorkspaceArtifacts::RunTaskId).uuid().null())
                .col(ColumnDef::new(WorkspaceArtifacts::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(WorkspaceArtifacts::ArtifactType)
                        .string_len(64)
                        .not_null()
                        .default("report"),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::Title)
                        .string_len(255)
                        .not_null(),
                )
                .col(ColumnDef::new(WorkspaceArtifacts::Content).text().null())
                .col(ColumnDef::new(WorkspaceArtifacts::Preview).text().null())
                .col(
                    ColumnDef::new(WorkspaceArtifacts::MimeType)
                        .string_len(128)
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::Format)
                        .string_len(64)
                        .not_null()
                        .default("markdown"),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::StorageKind)
                        .string_len(32)
                        .not_null()
                        .default("inline"),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::Bucket)
                        .string_len(128)
                        .null(),
                )
                .col(ColumnDef::new(WorkspaceArtifacts::ObjectKey).text().null())
                .col(
                    ColumnDef::new(WorkspaceArtifacts::FileName)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::FileSize)
                        .big_integer()
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::Sha256)
                        .string_len(128)
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::Source)
                        .string_len(64)
                        .not_null()
                        .default("agent_session"),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(WorkspaceArtifacts::ValidationStatus)
                        .string_len(32)
                        .not_null()
                        .default("unchecked"),
                )
                .col(jsonb_col(
                    WorkspaceArtifacts::ValidationReport,
                    "'{}'::jsonb",
                ))
                .col(jsonb_col(WorkspaceArtifacts::Metadata, "'{}'::jsonb"))
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_artifacts_workspace_id")
                        .from(WorkspaceArtifacts::Table, WorkspaceArtifacts::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_artifacts_run_id")
                        .from(WorkspaceArtifacts::Table, WorkspaceArtifacts::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_artifacts_workspace_task_id")
                        .from(
                            WorkspaceArtifacts::Table,
                            WorkspaceArtifacts::WorkspaceTaskId,
                        )
                        .to(WorkspaceTasks::Table, WorkspaceTasks::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_artifacts_run_task_id")
                        .from(WorkspaceArtifacts::Table, WorkspaceArtifacts::RunTaskId)
                        .to(WorkspaceRunTasks::Table, WorkspaceRunTasks::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_artifacts_user_id")
                        .from(WorkspaceArtifacts::Table, WorkspaceArtifacts::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_workspace_artifacts_workspace_run_created",
        WorkspaceArtifacts::Table,
        &[
            WorkspaceArtifacts::WorkspaceId,
            WorkspaceArtifacts::RunId,
            WorkspaceArtifacts::CreatedAt,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_artifacts_workspace_type_status",
        WorkspaceArtifacts::Table,
        &[
            WorkspaceArtifacts::WorkspaceId,
            WorkspaceArtifacts::ArtifactType,
            WorkspaceArtifacts::Status,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_artifacts_user_status_created",
        WorkspaceArtifacts::Table,
        &[
            WorkspaceArtifacts::UserId,
            WorkspaceArtifacts::Status,
            WorkspaceArtifacts::CreatedAt,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_artifacts_run_type_status",
        WorkspaceArtifacts::Table,
        &[
            WorkspaceArtifacts::RunId,
            WorkspaceArtifacts::ArtifactType,
            WorkspaceArtifacts::Status,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_artifacts_task",
        WorkspaceArtifacts::Table,
        &[WorkspaceArtifacts::WorkspaceTaskId],
    )
    .await
}

async fn create_index<T, C>(
    manager: &SchemaManager<'_>,
    name: &str,
    table: T,
    columns: &[C],
) -> Result<(), DbErr>
where
    T: Iden + Copy + 'static,
    C: Iden + Copy + 'static,
{
    let mut index = Index::create();
    index.name(name).table(table).if_not_exists();
    for column in columns {
        index.col(*column);
    }
    manager.create_index(index.to_owned()).await
}

fn jsonb_col<T>(column: T, default: &str) -> ColumnDef
where
    T: Iden + 'static,
{
    let mut col = ColumnDef::new(column);
    col.json_binary()
        .not_null()
        .default(Expr::cust(default.to_string()));
    col
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
enum Workspaces {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum WorkspaceRuns {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum WorkspaceTasks {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum WorkspaceRunTasks {
    Table,
    Id,
}

#[derive(DeriveIden, Clone, Copy)]
enum WorkspaceArtifacts {
    Table,
    Id,
    WorkspaceId,
    RunId,
    WorkspaceTaskId,
    RunTaskId,
    UserId,
    ArtifactType,
    Title,
    Content,
    Preview,
    MimeType,
    Format,
    StorageKind,
    Bucket,
    ObjectKey,
    FileName,
    FileSize,
    Sha256,
    Source,
    Status,
    ValidationStatus,
    ValidationReport,
    Metadata,
    CreatedAt,
}
