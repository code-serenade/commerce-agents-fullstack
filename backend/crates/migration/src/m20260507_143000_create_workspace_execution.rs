use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_workspaces(manager).await?;
        create_workspace_tasks(manager).await?;
        create_workspace_runs(manager).await?;
        create_workspace_run_tasks(manager).await?;
        create_execution_queue(manager).await?;
        create_updated_at_triggers(
            manager,
            &[
                "workspaces",
                "workspace_tasks",
                "workspace_runs",
                "workspace_run_tasks",
            ],
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ExecutionQueue::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(WorkspaceRunTasks::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(WorkspaceRuns::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(WorkspaceTasks::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Workspaces::Table).cascade().to_owned())
            .await
    }
}

async fn create_workspaces(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Workspaces::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Workspaces::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(Workspaces::UserId).uuid().not_null())
                .col(ColumnDef::new(Workspaces::ProductId).uuid().null())
                .col(ColumnDef::new(Workspaces::CampaignId).uuid().null())
                .col(ColumnDef::new(Workspaces::Title).string_len(255).not_null())
                .col(
                    ColumnDef::new(Workspaces::WorkspaceType)
                        .string_len(64)
                        .not_null()
                        .default("general"),
                )
                .col(
                    ColumnDef::new(Workspaces::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(Workspaces::Config)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspaces_user_id")
                        .from(Workspaces::Table, Workspaces::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspaces_product_id")
                        .from(Workspaces::Table, Workspaces::ProductId)
                        .to(Products::Table, Products::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspaces_campaign_id")
                        .from(Workspaces::Table, Workspaces::CampaignId)
                        .to(Campaigns::Table, Campaigns::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_workspaces_user_status",
        Workspaces::Table,
        &[Workspaces::UserId, Workspaces::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_workspaces_product_status",
        Workspaces::Table,
        &[Workspaces::ProductId, Workspaces::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_workspaces_campaign_status",
        Workspaces::Table,
        &[Workspaces::CampaignId, Workspaces::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_workspaces_user_updated",
        Workspaces::Table,
        &[Workspaces::UserId, Workspaces::UpdatedAt],
    )
    .await
}

async fn create_workspace_tasks(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(WorkspaceTasks::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(WorkspaceTasks::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(WorkspaceTasks::WorkspaceId)
                        .uuid()
                        .not_null(),
                )
                .col(ColumnDef::new(WorkspaceTasks::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(WorkspaceTasks::Title)
                        .string_len(255)
                        .not_null(),
                )
                .col(ColumnDef::new(WorkspaceTasks::Description).text().null())
                .col(
                    ColumnDef::new(WorkspaceTasks::OwnerRole)
                        .string_len(64)
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceTasks::Status)
                        .string_len(32)
                        .not_null()
                        .default("pending"),
                )
                .col(
                    ColumnDef::new(WorkspaceTasks::Priority)
                        .integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(WorkspaceTasks::DependsOn)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'[]'::jsonb")),
                )
                .col(
                    ColumnDef::new(WorkspaceTasks::AcceptanceCriteria)
                        .text()
                        .null(),
                )
                .col(ColumnDef::new(WorkspaceTasks::Result).text().null())
                .col(
                    ColumnDef::new(WorkspaceTasks::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_tasks_workspace_id")
                        .from(WorkspaceTasks::Table, WorkspaceTasks::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_tasks_user_id")
                        .from(WorkspaceTasks::Table, WorkspaceTasks::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_workspace_tasks_workspace_status",
        WorkspaceTasks::Table,
        &[WorkspaceTasks::WorkspaceId, WorkspaceTasks::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_tasks_user_status",
        WorkspaceTasks::Table,
        &[WorkspaceTasks::UserId, WorkspaceTasks::Status],
    )
    .await
}

async fn create_workspace_runs(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(WorkspaceRuns::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(WorkspaceRuns::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(WorkspaceRuns::WorkspaceId).uuid().not_null())
                .col(ColumnDef::new(WorkspaceRuns::UserId).uuid().not_null())
                .col(ColumnDef::new(WorkspaceRuns::Goal).text().not_null())
                .col(
                    ColumnDef::new(WorkspaceRuns::Requirements)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'[]'::jsonb")),
                )
                .col(
                    ColumnDef::new(WorkspaceRuns::Status)
                        .string_len(32)
                        .not_null()
                        .default("planned"),
                )
                .col(
                    ColumnDef::new(WorkspaceRuns::Source)
                        .string_len(64)
                        .not_null()
                        .default("manual"),
                )
                .col(ColumnDef::new(WorkspaceRuns::Summary).text().null())
                .col(
                    ColumnDef::new(WorkspaceRuns::Report)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(WorkspaceRuns::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_runs_workspace_id")
                        .from(WorkspaceRuns::Table, WorkspaceRuns::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_runs_user_id")
                        .from(WorkspaceRuns::Table, WorkspaceRuns::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_workspace_runs_workspace_status",
        WorkspaceRuns::Table,
        &[WorkspaceRuns::WorkspaceId, WorkspaceRuns::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_runs_user_status",
        WorkspaceRuns::Table,
        &[WorkspaceRuns::UserId, WorkspaceRuns::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_runs_workspace_updated",
        WorkspaceRuns::Table,
        &[WorkspaceRuns::WorkspaceId, WorkspaceRuns::UpdatedAt],
    )
    .await
}

async fn create_workspace_run_tasks(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(WorkspaceRunTasks::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(WorkspaceRunTasks::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(WorkspaceRunTasks::RunId).uuid().not_null())
                .col(
                    ColumnDef::new(WorkspaceRunTasks::WorkspaceId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::WorkspaceTaskId)
                        .uuid()
                        .null(),
                )
                .col(ColumnDef::new(WorkspaceRunTasks::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(WorkspaceRunTasks::Seq)
                        .integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::OwnerRole)
                        .string_len(64)
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::Status)
                        .string_len(32)
                        .not_null()
                        .default("pending"),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::Attempts)
                        .integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::AcceptanceScore)
                        .double()
                        .not_null()
                        .default(0.0),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::AcceptancePassed)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::AcceptanceNotes)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::ExecutionNotes)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunTasks::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_run_tasks_run_id")
                        .from(WorkspaceRunTasks::Table, WorkspaceRunTasks::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_run_tasks_workspace_id")
                        .from(WorkspaceRunTasks::Table, WorkspaceRunTasks::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_run_tasks_workspace_task_id")
                        .from(WorkspaceRunTasks::Table, WorkspaceRunTasks::WorkspaceTaskId)
                        .to(WorkspaceTasks::Table, WorkspaceTasks::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_run_tasks_user_id")
                        .from(WorkspaceRunTasks::Table, WorkspaceRunTasks::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_workspace_run_tasks_run_status",
        WorkspaceRunTasks::Table,
        &[WorkspaceRunTasks::RunId, WorkspaceRunTasks::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_run_tasks_workspace_status",
        WorkspaceRunTasks::Table,
        &[WorkspaceRunTasks::WorkspaceId, WorkspaceRunTasks::Status],
    )
    .await
}

async fn create_execution_queue(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ExecutionQueue::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ExecutionQueue::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(ExecutionQueue::UserId).uuid().not_null())
                .col(ColumnDef::new(ExecutionQueue::WorkspaceId).uuid().null())
                .col(ColumnDef::new(ExecutionQueue::RunId).uuid().null())
                .col(
                    ColumnDef::new(ExecutionQueue::WorkspaceTaskId)
                        .uuid()
                        .null(),
                )
                .col(
                    ColumnDef::new(ExecutionQueue::ActionType)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ExecutionQueue::Description)
                        .text()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ExecutionQueue::Platform)
                        .string_len(64)
                        .null(),
                )
                .col(
                    ColumnDef::new(ExecutionQueue::Payload)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(ColumnDef::new(ExecutionQueue::ExpectedImpact).text().null())
                .col(
                    ColumnDef::new(ExecutionQueue::Status)
                        .string_len(32)
                        .not_null()
                        .default("pending"),
                )
                .col(ColumnDef::new(ExecutionQueue::DecisionReason).text().null())
                .col(
                    ColumnDef::new(ExecutionQueue::Result)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(
                    ColumnDef::new(ExecutionQueue::DecidedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_execution_queue_user_id")
                        .from(ExecutionQueue::Table, ExecutionQueue::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_execution_queue_workspace_id")
                        .from(ExecutionQueue::Table, ExecutionQueue::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_execution_queue_run_id")
                        .from(ExecutionQueue::Table, ExecutionQueue::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_execution_queue_workspace_task_id")
                        .from(ExecutionQueue::Table, ExecutionQueue::WorkspaceTaskId)
                        .to(WorkspaceTasks::Table, WorkspaceTasks::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_execution_queue_user_status",
        ExecutionQueue::Table,
        &[ExecutionQueue::UserId, ExecutionQueue::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_execution_queue_workspace_status",
        ExecutionQueue::Table,
        &[ExecutionQueue::WorkspaceId, ExecutionQueue::Status],
    )
    .await?;
    create_index(
        manager,
        "idx_execution_queue_run_status",
        ExecutionQueue::Table,
        &[ExecutionQueue::RunId, ExecutionQueue::Status],
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
}

#[derive(DeriveIden, Clone, Copy)]
enum Workspaces {
    Table,
    Id,
    UserId,
    ProductId,
    CampaignId,
    Title,
    WorkspaceType,
    Status,
    Config,
    UpdatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum WorkspaceTasks {
    Table,
    Id,
    WorkspaceId,
    UserId,
    Title,
    Description,
    OwnerRole,
    Status,
    Priority,
    DependsOn,
    AcceptanceCriteria,
    Result,
    Metadata,
}

#[derive(DeriveIden, Clone, Copy)]
enum WorkspaceRuns {
    Table,
    Id,
    WorkspaceId,
    UserId,
    Goal,
    Requirements,
    Status,
    Source,
    Summary,
    Report,
    Metadata,
    UpdatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum WorkspaceRunTasks {
    Table,
    Id,
    RunId,
    WorkspaceId,
    WorkspaceTaskId,
    UserId,
    Seq,
    OwnerRole,
    Status,
    Attempts,
    AcceptanceScore,
    AcceptancePassed,
    AcceptanceNotes,
    ExecutionNotes,
    Metadata,
}

#[derive(DeriveIden, Clone, Copy)]
enum ExecutionQueue {
    Table,
    Id,
    UserId,
    WorkspaceId,
    RunId,
    WorkspaceTaskId,
    ActionType,
    Description,
    Platform,
    Payload,
    ExpectedImpact,
    Status,
    DecisionReason,
    Result,
    DecidedAt,
}
