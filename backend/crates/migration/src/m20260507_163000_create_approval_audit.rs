use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_tool_approval_queue(manager).await?;
        create_audit_logs(manager).await?;
        create_updated_at_triggers(manager, &["tool_approval_queue"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).cascade().to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ToolApprovalQueue::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_tool_approval_queue(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ToolApprovalQueue::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ToolApprovalQueue::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(ToolApprovalQueue::UserId).uuid().not_null())
                .col(ColumnDef::new(ToolApprovalQueue::WorkspaceId).uuid().null())
                .col(ColumnDef::new(ToolApprovalQueue::RunId).uuid().null())
                .col(
                    ColumnDef::new(ToolApprovalQueue::ConversationId)
                        .uuid()
                        .null(),
                )
                .col(
                    ColumnDef::new(ToolApprovalQueue::RequestRole)
                        .string_len(64)
                        .null(),
                )
                .col(
                    ColumnDef::new(ToolApprovalQueue::SkillName)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(ToolApprovalQueue::ToolCallId)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(ToolApprovalQueue::RiskLevel)
                        .string_len(16)
                        .not_null()
                        .default("L1"),
                )
                .col(
                    ColumnDef::new(ToolApprovalQueue::Args)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(ToolApprovalQueue::Status)
                        .string_len(32)
                        .not_null()
                        .default("pending"),
                )
                .col(ColumnDef::new(ToolApprovalQueue::Reason).text().null())
                .col(
                    ColumnDef::new(ToolApprovalQueue::ActionKey)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(ToolApprovalQueue::Result)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(ColumnDef::new(ToolApprovalQueue::ApprovedBy).uuid().null())
                .col(
                    ColumnDef::new(ToolApprovalQueue::ApprovedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(ToolApprovalQueue::ExecutedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_tool_approval_queue_user_id")
                        .from(ToolApprovalQueue::Table, ToolApprovalQueue::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_tool_approval_queue_workspace_id")
                        .from(ToolApprovalQueue::Table, ToolApprovalQueue::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_tool_approval_queue_run_id")
                        .from(ToolApprovalQueue::Table, ToolApprovalQueue::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_tool_approval_queue_conversation_id")
                        .from(ToolApprovalQueue::Table, ToolApprovalQueue::ConversationId)
                        .to(Conversations::Table, Conversations::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_tool_approval_queue_approved_by")
                        .from(ToolApprovalQueue::Table, ToolApprovalQueue::ApprovedBy)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_tool_approval_user_status",
        ToolApprovalQueue::Table,
        &[ToolApprovalQueue::UserId, ToolApprovalQueue::Status],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_tool_approval_workspace_status",
        ToolApprovalQueue::Table,
        &[
            ToolApprovalQueue::WorkspaceId,
            ToolApprovalQueue::Status,
            ToolApprovalQueue::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_tool_approval_run_status",
        ToolApprovalQueue::Table,
        &[
            ToolApprovalQueue::RunId,
            ToolApprovalQueue::Status,
            ToolApprovalQueue::CreatedAt,
        ],
        false,
    )
    .await?;
    manager
        .get_connection()
        .execute_unprepared(
            r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_tool_approval_action_key
ON tool_approval_queue(action_key)
WHERE action_key IS NOT NULL AND action_key <> '';
"#,
        )
        .await?;
    Ok(())
}

async fn create_audit_logs(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(AuditLogs::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(AuditLogs::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(AuditLogs::UserId).uuid().not_null())
                .col(ColumnDef::new(AuditLogs::ActorUserId).uuid().not_null())
                .col(ColumnDef::new(AuditLogs::WorkspaceId).uuid().null())
                .col(ColumnDef::new(AuditLogs::RunId).uuid().null())
                .col(ColumnDef::new(AuditLogs::ExecutionQueueId).uuid().null())
                .col(ColumnDef::new(AuditLogs::ToolApprovalId).uuid().null())
                .col(ColumnDef::new(AuditLogs::Action).string_len(128).not_null())
                .col(
                    ColumnDef::new(AuditLogs::TargetType)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(AuditLogs::TargetId).string_len(255).null())
                .col(
                    ColumnDef::new(AuditLogs::Status)
                        .string_len(32)
                        .not_null()
                        .default("success"),
                )
                .col(ColumnDef::new(AuditLogs::Reason).text().null())
                .col(ColumnDef::new(AuditLogs::RequestId).string_len(255).null())
                .col(ColumnDef::new(AuditLogs::TraceId).string_len(255).null())
                .col(
                    ColumnDef::new(AuditLogs::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_audit_logs_user_id")
                        .from(AuditLogs::Table, AuditLogs::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_audit_logs_actor_user_id")
                        .from(AuditLogs::Table, AuditLogs::ActorUserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_audit_logs_workspace_id")
                        .from(AuditLogs::Table, AuditLogs::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_audit_logs_run_id")
                        .from(AuditLogs::Table, AuditLogs::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_audit_logs_execution_queue_id")
                        .from(AuditLogs::Table, AuditLogs::ExecutionQueueId)
                        .to(ExecutionQueue::Table, ExecutionQueue::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_audit_logs_tool_approval_id")
                        .from(AuditLogs::Table, AuditLogs::ToolApprovalId)
                        .to(ToolApprovalQueue::Table, ToolApprovalQueue::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_audit_logs_user_created",
        AuditLogs::Table,
        &[AuditLogs::UserId, AuditLogs::CreatedAt],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_audit_logs_actor_created",
        AuditLogs::Table,
        &[AuditLogs::ActorUserId, AuditLogs::CreatedAt],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_audit_logs_workspace_created",
        AuditLogs::Table,
        &[AuditLogs::WorkspaceId, AuditLogs::CreatedAt],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_audit_logs_action_created",
        AuditLogs::Table,
        &[AuditLogs::Action, AuditLogs::CreatedAt],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_audit_logs_execution_created",
        AuditLogs::Table,
        &[AuditLogs::ExecutionQueueId, AuditLogs::CreatedAt],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_audit_logs_tool_approval_created",
        AuditLogs::Table,
        &[AuditLogs::ToolApprovalId, AuditLogs::CreatedAt],
        false,
    )
    .await
}

async fn create_index<T, C>(
    manager: &SchemaManager<'_>,
    name: &str,
    table: T,
    columns: &[C],
    unique: bool,
) -> Result<(), DbErr>
where
    T: Iden + Copy + 'static,
    C: Iden + Copy + 'static,
{
    let mut index = Index::create();
    index.name(name).table(table).if_not_exists();
    if unique {
        index.unique();
    }
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
enum Conversations {
    Table,
    Id,
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
enum ExecutionQueue {
    Table,
    Id,
}

#[derive(DeriveIden, Clone, Copy)]
enum ToolApprovalQueue {
    Table,
    Id,
    UserId,
    WorkspaceId,
    RunId,
    ConversationId,
    RequestRole,
    SkillName,
    ToolCallId,
    RiskLevel,
    Args,
    Status,
    Reason,
    ActionKey,
    Result,
    ApprovedBy,
    ApprovedAt,
    ExecutedAt,
    CreatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum AuditLogs {
    Table,
    Id,
    UserId,
    ActorUserId,
    WorkspaceId,
    RunId,
    ExecutionQueueId,
    ToolApprovalId,
    Action,
    TargetType,
    TargetId,
    Status,
    Reason,
    RequestId,
    TraceId,
    Metadata,
    CreatedAt,
}
