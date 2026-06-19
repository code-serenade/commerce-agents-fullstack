use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TeamUsageQuotas::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamUsageQuotas::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TeamUsageQuotas::SchoolName)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TeamUsageQuotas::TeamName)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TeamUsageQuotas::QuestionLimit)
                            .integer()
                            .not_null()
                            .default(50),
                    )
                    .col(ColumnDef::new(TeamUsageQuotas::TokenLimit).integer().null())
                    .col(
                        ColumnDef::new(TeamUsageQuotas::Status)
                            .string_len(32)
                            .not_null()
                            .default("enabled"),
                    )
                    .col(
                        ColumnDef::new(TeamUsageQuotas::Remark)
                            .string_len(255)
                            .null(),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .index(
                        Index::create()
                            .name("uk_team_usage_quotas_team")
                            .col(TeamUsageQuotas::SchoolName)
                            .col(TeamUsageQuotas::TeamName)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AgentUsageEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentUsageEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AgentUsageEvents::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(AgentUsageEvents::SchoolName)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::TeamName)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::ConversationId)
                            .uuid()
                            .null(),
                    )
                    .col(ColumnDef::new(AgentUsageEvents::WorkspaceId).uuid().null())
                    .col(ColumnDef::new(AgentUsageEvents::RunId).uuid().null())
                    .col(
                        ColumnDef::new(AgentUsageEvents::SessionId)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::AgentId)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::QuestionCount)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::PromptTokens)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::CompletionTokens)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::TotalTokens)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::Status)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentUsageEvents::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_usage_events_user_id")
                            .from(AgentUsageEvents::Table, AgentUsageEvents::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        create_indexes(manager).await?;
        seed_admin_permission(manager).await?;
        create_updated_at_triggers(manager, &["team_usage_quotas", "agent_usage_events"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DELETE FROM admin_menus WHERE permission_code = 'usage:team_usage';
DELETE FROM admin_permissions WHERE code = 'usage:team_usage';
DELETE FROM admin_permissions WHERE code = 'usage';
"#,
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AgentUsageEvents::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(TeamUsageQuotas::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_index(
            Index::create()
                .name("idx_team_usage_quotas_status_updated")
                .table(TeamUsageQuotas::Table)
                .col(TeamUsageQuotas::Status)
                .col(TeamUsageQuotas::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_agent_usage_events_team_created")
                .table(AgentUsageEvents::Table)
                .col(AgentUsageEvents::SchoolName)
                .col(AgentUsageEvents::TeamName)
                .col(AgentUsageEvents::CreatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_agent_usage_events_user_created")
                .table(AgentUsageEvents::Table)
                .col(AgentUsageEvents::UserId)
                .col(AgentUsageEvents::CreatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_agent_usage_events_session")
                .table(AgentUsageEvents::Table)
                .col(AgentUsageEvents::SessionId)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_agent_usage_events_conversation")
                .table(AgentUsageEvents::Table)
                .col(AgentUsageEvents::ConversationId)
                .if_not_exists()
                .to_owned(),
        )
        .await?;

    Ok(())
}

async fn seed_admin_permission(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            r#"
INSERT INTO admin_permissions (code, name, parent_code, sort, kind)
VALUES ('usage', 'Usage', NULL, 30, 'group')
ON CONFLICT (code) DO NOTHING;

INSERT INTO admin_permissions (code, name, parent_code, sort, kind)
VALUES ('usage:team_usage', 'Team Usage', 'usage', 10, 'action')
ON CONFLICT (code) DO NOTHING;

INSERT INTO admin_menus (name, parent_id, permission_code)
SELECT 'Team Usage', NULL, 'usage:team_usage'
WHERE NOT EXISTS (
    SELECT 1 FROM admin_menus WHERE permission_code = 'usage:team_usage'
);

INSERT INTO admin_role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM admin_roles r
CROSS JOIN admin_permissions p
WHERE r.code IN ('root', 'admin')
  AND p.code = 'usage:team_usage'
ON CONFLICT DO NOTHING;
"#,
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
enum TeamUsageQuotas {
    Table,
    Id,
    SchoolName,
    TeamName,
    QuestionLimit,
    TokenLimit,
    Status,
    Remark,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AgentUsageEvents {
    Table,
    Id,
    UserId,
    SchoolName,
    TeamName,
    ConversationId,
    WorkspaceId,
    RunId,
    SessionId,
    AgentId,
    QuestionCount,
    PromptTokens,
    CompletionTokens,
    TotalTokens,
    Status,
    Metadata,
    CreatedAt,
}
