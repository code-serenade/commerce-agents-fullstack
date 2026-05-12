use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentSkillRuns::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentSkillRuns::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AgentSkillRuns::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(AgentSkillRuns::WorkspaceId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AgentSkillRuns::RunId).uuid().not_null())
                    .col(
                        ColumnDef::new(AgentSkillRuns::SessionId)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentSkillRuns::WorkerId)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentSkillRuns::TaskId)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentSkillRuns::RequestId)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentSkillRuns::SkillId)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentSkillRuns::Role)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentSkillRuns::Status)
                            .string_len(32)
                            .not_null()
                            .default("requested"),
                    )
                    .col(ColumnDef::new(AgentSkillRuns::Reason).text().null())
                    .col(ColumnDef::new(AgentSkillRuns::Summary).text().null())
                    .col(ColumnDef::new(AgentSkillRuns::Error).text().null())
                    .col(jsonb_col(AgentSkillRuns::Input, "'{}'::jsonb"))
                    .col(jsonb_col(AgentSkillRuns::Output, "'{}'::jsonb"))
                    .col(
                        ColumnDef::new(AgentSkillRuns::ExecutionMode)
                            .string_len(32)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentSkillRuns::RiskLevel)
                            .string_len(32)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentSkillRuns::ApprovalRequired)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(jsonb_col(AgentSkillRuns::Metadata, "'{}'::jsonb"))
                    .col(timestamp_col(AgentSkillRuns::RequestedAt))
                    .col(
                        ColumnDef::new(AgentSkillRuns::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(timestamp_col(AgentSkillRuns::CreatedAt))
                    .col(timestamp_col(AgentSkillRuns::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_skill_runs_user_id")
                            .from(AgentSkillRuns::Table, AgentSkillRuns::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_skill_runs_workspace_id")
                            .from(AgentSkillRuns::Table, AgentSkillRuns::WorkspaceId)
                            .to(Workspaces::Table, Workspaces::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_skill_runs_run_id")
                            .from(AgentSkillRuns::Table, AgentSkillRuns::RunId)
                            .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        create_indexes(manager).await?;
        create_updated_at_triggers(manager, &["agent_skill_runs"]).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(AgentSkillRuns::Table)
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
                .name("uidx_agent_skill_runs_request")
                .table(AgentSkillRuns::Table)
                .col(AgentSkillRuns::UserId)
                .col(AgentSkillRuns::WorkspaceId)
                .col(AgentSkillRuns::RunId)
                .col(AgentSkillRuns::RequestId)
                .unique()
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    create_index(
        manager,
        "idx_agent_skill_runs_user_run_requested",
        &[
            AgentSkillRuns::UserId,
            AgentSkillRuns::RunId,
            AgentSkillRuns::RequestedAt,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_agent_skill_runs_workspace_run_status",
        &[
            AgentSkillRuns::WorkspaceId,
            AgentSkillRuns::RunId,
            AgentSkillRuns::Status,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_agent_skill_runs_user_skill_requested",
        &[
            AgentSkillRuns::UserId,
            AgentSkillRuns::SkillId,
            AgentSkillRuns::RequestedAt,
        ],
    )
    .await
}

async fn create_index(
    manager: &SchemaManager<'_>,
    name: &str,
    columns: &[AgentSkillRuns],
) -> Result<(), DbErr> {
    let mut index = Index::create();
    index
        .name(name)
        .table(AgentSkillRuns::Table)
        .if_not_exists();
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

fn timestamp_col<T>(column: T) -> ColumnDef
where
    T: Iden + 'static,
{
    let mut col = ColumnDef::new(column);
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

#[derive(DeriveIden, Clone, Copy)]
enum AgentSkillRuns {
    Table,
    Id,
    UserId,
    WorkspaceId,
    RunId,
    SessionId,
    WorkerId,
    TaskId,
    RequestId,
    SkillId,
    Role,
    Status,
    Reason,
    Summary,
    Error,
    Input,
    Output,
    ExecutionMode,
    RiskLevel,
    ApprovalRequired,
    Metadata,
    RequestedAt,
    CompletedAt,
    CreatedAt,
    UpdatedAt,
}
