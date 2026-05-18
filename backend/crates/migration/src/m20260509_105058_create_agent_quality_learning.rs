use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_agent_quality_checks(manager).await?;
        create_agent_trust_scores(manager).await?;
        create_agent_learning_records(manager).await?;
        create_updated_at_triggers(manager, &["agent_trust_scores", "agent_learning_records"])
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(AgentLearningRecords::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AgentTrustScores::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AgentQualityChecks::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_agent_quality_checks(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(AgentQualityChecks::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(AgentQualityChecks::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(AgentQualityChecks::WorkspaceId)
                        .uuid()
                        .not_null(),
                )
                .col(ColumnDef::new(AgentQualityChecks::RunId).uuid().not_null())
                .col(ColumnDef::new(AgentQualityChecks::ArtifactId).uuid().null())
                .col(ColumnDef::new(AgentQualityChecks::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(AgentQualityChecks::Role)
                        .string_len(64)
                        .not_null()
                        .default("agent"),
                )
                .col(
                    ColumnDef::new(AgentQualityChecks::Score)
                        .double()
                        .not_null()
                        .default(0.0),
                )
                .col(
                    ColumnDef::new(AgentQualityChecks::Passed)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(jsonb_col(AgentQualityChecks::Dimensions, "'{}'::jsonb"))
                .col(jsonb_col(AgentQualityChecks::Issues, "'[]'::jsonb"))
                .col(jsonb_col(AgentQualityChecks::Suggestions, "'[]'::jsonb"))
                .col(
                    ColumnDef::new(AgentQualityChecks::Source)
                        .string_len(64)
                        .not_null()
                        .default("agent_session"),
                )
                .col(jsonb_col(AgentQualityChecks::Metadata, "'{}'::jsonb"))
                .col(created_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_quality_checks_workspace_id")
                        .from(AgentQualityChecks::Table, AgentQualityChecks::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_quality_checks_run_id")
                        .from(AgentQualityChecks::Table, AgentQualityChecks::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_quality_checks_artifact_id")
                        .from(AgentQualityChecks::Table, AgentQualityChecks::ArtifactId)
                        .to(WorkspaceArtifacts::Table, WorkspaceArtifacts::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_quality_checks_user_id")
                        .from(AgentQualityChecks::Table, AgentQualityChecks::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_agent_quality_user_run_created",
        AgentQualityChecks::Table,
        &[
            AgentQualityChecks::UserId,
            AgentQualityChecks::RunId,
            AgentQualityChecks::CreatedAt,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_agent_quality_workspace_created",
        AgentQualityChecks::Table,
        &[
            AgentQualityChecks::WorkspaceId,
            AgentQualityChecks::CreatedAt,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_agent_quality_run_created",
        AgentQualityChecks::Table,
        &[AgentQualityChecks::RunId, AgentQualityChecks::CreatedAt],
    )
    .await
}

async fn create_agent_trust_scores(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(AgentTrustScores::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(AgentTrustScores::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(AgentTrustScores::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(AgentTrustScores::Role)
                        .string_len(64)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(AgentTrustScores::SkillId)
                        .string_len(128)
                        .null(),
                )
                .col(
                    ColumnDef::new(AgentTrustScores::Score)
                        .double()
                        .not_null()
                        .default(0.5),
                )
                .col(
                    ColumnDef::new(AgentTrustScores::TrustLevel)
                        .string_len(32)
                        .not_null()
                        .default("moderate"),
                )
                .col(
                    ColumnDef::new(AgentTrustScores::PreviousScore)
                        .double()
                        .null(),
                )
                .col(
                    ColumnDef::new(AgentTrustScores::PreviousLevel)
                        .string_len(32)
                        .null(),
                )
                .col(
                    ColumnDef::new(AgentTrustScores::Delta)
                        .double()
                        .not_null()
                        .default(0.0),
                )
                .col(ColumnDef::new(AgentTrustScores::Reason).text().null())
                .col(
                    ColumnDef::new(AgentTrustScores::Source)
                        .string_len(64)
                        .not_null()
                        .default("quality_check"),
                )
                .col(
                    ColumnDef::new(AgentTrustScores::QualityCheckId)
                        .uuid()
                        .null(),
                )
                .col(jsonb_col(AgentTrustScores::Metadata, "'{}'::jsonb"))
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_trust_scores_user_id")
                        .from(AgentTrustScores::Table, AgentTrustScores::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_trust_scores_quality_check_id")
                        .from(AgentTrustScores::Table, AgentTrustScores::QualityCheckId)
                        .to(AgentQualityChecks::Table, AgentQualityChecks::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_agent_trust_user_role_updated",
        AgentTrustScores::Table,
        &[
            AgentTrustScores::UserId,
            AgentTrustScores::Role,
            AgentTrustScores::UpdatedAt,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_agent_trust_user_role_skill",
        AgentTrustScores::Table,
        &[
            AgentTrustScores::UserId,
            AgentTrustScores::Role,
            AgentTrustScores::SkillId,
        ],
    )
    .await
}

async fn create_agent_learning_records(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(AgentLearningRecords::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(AgentLearningRecords::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::UserId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::WorkspaceId)
                        .uuid()
                        .null(),
                )
                .col(ColumnDef::new(AgentLearningRecords::RunId).uuid().null())
                .col(
                    ColumnDef::new(AgentLearningRecords::QualityCheckId)
                        .uuid()
                        .null(),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::Role)
                        .string_len(64)
                        .not_null()
                        .default("agent"),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::Category)
                        .string_len(32)
                        .not_null()
                        .default("general"),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::Content)
                        .text()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::Confidence)
                        .double()
                        .not_null()
                        .default(0.5),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::SourceAction)
                        .string_len(64)
                        .not_null()
                        .default(""),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::FeedbackRating)
                        .integer()
                        .null(),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::FeedbackComment)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(AgentLearningRecords::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(jsonb_col(AgentLearningRecords::Metadata, "'{}'::jsonb"))
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_learning_records_user_id")
                        .from(AgentLearningRecords::Table, AgentLearningRecords::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_learning_records_workspace_id")
                        .from(
                            AgentLearningRecords::Table,
                            AgentLearningRecords::WorkspaceId,
                        )
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_learning_records_run_id")
                        .from(AgentLearningRecords::Table, AgentLearningRecords::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_agent_learning_records_quality_check_id")
                        .from(
                            AgentLearningRecords::Table,
                            AgentLearningRecords::QualityCheckId,
                        )
                        .to(AgentQualityChecks::Table, AgentQualityChecks::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_agent_learning_user_role_category_confidence",
        AgentLearningRecords::Table,
        &[
            AgentLearningRecords::UserId,
            AgentLearningRecords::Role,
            AgentLearningRecords::Category,
            AgentLearningRecords::Confidence,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_agent_learning_workspace_run",
        AgentLearningRecords::Table,
        &[
            AgentLearningRecords::WorkspaceId,
            AgentLearningRecords::RunId,
        ],
    )
    .await?;
    create_index(
        manager,
        "idx_agent_learning_user_status_updated",
        AgentLearningRecords::Table,
        &[
            AgentLearningRecords::UserId,
            AgentLearningRecords::Status,
            AgentLearningRecords::UpdatedAt,
        ],
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
enum WorkspaceArtifacts {
    Table,
    Id,
}

#[derive(DeriveIden, Clone, Copy)]
enum AgentQualityChecks {
    Table,
    Id,
    WorkspaceId,
    RunId,
    ArtifactId,
    UserId,
    Role,
    Score,
    Passed,
    Dimensions,
    Issues,
    Suggestions,
    Source,
    Metadata,
    CreatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum AgentTrustScores {
    Table,
    Id,
    UserId,
    Role,
    SkillId,
    Score,
    TrustLevel,
    PreviousScore,
    PreviousLevel,
    Delta,
    Reason,
    Source,
    QualityCheckId,
    Metadata,
    UpdatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum AgentLearningRecords {
    Table,
    Id,
    UserId,
    WorkspaceId,
    RunId,
    QualityCheckId,
    Role,
    Category,
    Content,
    Confidence,
    SourceAction,
    FeedbackRating,
    FeedbackComment,
    Status,
    Metadata,
    UpdatedAt,
}
