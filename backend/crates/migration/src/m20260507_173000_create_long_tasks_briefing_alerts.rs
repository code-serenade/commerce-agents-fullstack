use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_long_running_tasks(manager).await?;
        create_daily_briefings(manager).await?;
        create_background_events(manager).await?;
        create_alerts(manager).await?;
        create_updated_at_triggers(
            manager,
            &["long_running_tasks", "daily_briefings", "alerts"],
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alerts::Table).cascade().to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(BackgroundEvents::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(DailyBriefings::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(LongRunningTasks::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_long_running_tasks(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(LongRunningTasks::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(LongRunningTasks::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(LongRunningTasks::UserId).uuid().not_null())
                .col(ColumnDef::new(LongRunningTasks::WorkspaceId).uuid().null())
                .col(ColumnDef::new(LongRunningTasks::RunId).uuid().null())
                .col(
                    ColumnDef::new(LongRunningTasks::TaskType)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(LongRunningTasks::Status)
                        .string_len(32)
                        .not_null()
                        .default("pending"),
                )
                .col(
                    ColumnDef::new(LongRunningTasks::Title)
                        .string_len(255)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(LongRunningTasks::Progress)
                        .integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(LongRunningTasks::TotalSteps)
                        .integer()
                        .not_null()
                        .default(100),
                )
                .col(
                    ColumnDef::new(LongRunningTasks::CurrentStep)
                        .string_len(255)
                        .null(),
                )
                .col(ColumnDef::new(LongRunningTasks::Role).string_len(64).null())
                .col(
                    ColumnDef::new(LongRunningTasks::InputParams)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(LongRunningTasks::Result)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(ColumnDef::new(LongRunningTasks::ErrorMessage).text().null())
                .col(
                    ColumnDef::new(LongRunningTasks::StartedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(LongRunningTasks::CompletedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_long_running_tasks_user_id")
                        .from(LongRunningTasks::Table, LongRunningTasks::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_long_running_tasks_workspace_id")
                        .from(LongRunningTasks::Table, LongRunningTasks::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_long_running_tasks_run_id")
                        .from(LongRunningTasks::Table, LongRunningTasks::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_long_running_tasks_user_status",
        LongRunningTasks::Table,
        &[
            LongRunningTasks::UserId,
            LongRunningTasks::Status,
            LongRunningTasks::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_long_running_tasks_workspace_status",
        LongRunningTasks::Table,
        &[
            LongRunningTasks::WorkspaceId,
            LongRunningTasks::Status,
            LongRunningTasks::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_long_running_tasks_run_status",
        LongRunningTasks::Table,
        &[
            LongRunningTasks::RunId,
            LongRunningTasks::Status,
            LongRunningTasks::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_long_running_tasks_type_status",
        LongRunningTasks::Table,
        &[
            LongRunningTasks::TaskType,
            LongRunningTasks::Status,
            LongRunningTasks::CreatedAt,
        ],
        false,
    )
    .await
}

async fn create_daily_briefings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(DailyBriefings::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(DailyBriefings::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(DailyBriefings::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(DailyBriefings::BriefingDate)
                        .string_len(32)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DailyBriefings::Title)
                        .string_len(255)
                        .not_null(),
                )
                .col(ColumnDef::new(DailyBriefings::Summary).text().null())
                .col(
                    ColumnDef::new(DailyBriefings::Sections)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'[]'::jsonb")),
                )
                .col(
                    ColumnDef::new(DailyBriefings::ProductSummary)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(DailyBriefings::AlertSummary)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(DailyBriefings::TaskSummary)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(DailyBriefings::Source)
                        .string_len(64)
                        .not_null()
                        .default("manual"),
                )
                .col(
                    ColumnDef::new(DailyBriefings::Status)
                        .string_len(32)
                        .not_null()
                        .default("ready"),
                )
                .col(
                    ColumnDef::new(DailyBriefings::Archived)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_daily_briefings_user_id")
                        .from(DailyBriefings::Table, DailyBriefings::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_daily_briefings_user_date_unique",
        DailyBriefings::Table,
        &[DailyBriefings::UserId, DailyBriefings::BriefingDate],
        true,
    )
    .await?;
    create_index(
        manager,
        "idx_daily_briefings_user_archived_date",
        DailyBriefings::Table,
        &[
            DailyBriefings::UserId,
            DailyBriefings::Archived,
            DailyBriefings::BriefingDate,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_daily_briefings_user_status_created",
        DailyBriefings::Table,
        &[
            DailyBriefings::UserId,
            DailyBriefings::Status,
            DailyBriefings::CreatedAt,
        ],
        false,
    )
    .await
}

async fn create_background_events(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(BackgroundEvents::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(BackgroundEvents::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(BackgroundEvents::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(BackgroundEvents::ConversationId)
                        .uuid()
                        .null(),
                )
                .col(
                    ColumnDef::new(BackgroundEvents::EventType)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(BackgroundEvents::Title)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(BackgroundEvents::Data)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(BackgroundEvents::Consumed)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(
                    ColumnDef::new(BackgroundEvents::ConsumedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(created_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_background_events_user_id")
                        .from(BackgroundEvents::Table, BackgroundEvents::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_background_events_conversation_id")
                        .from(BackgroundEvents::Table, BackgroundEvents::ConversationId)
                        .to(Conversations::Table, Conversations::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_background_events_user_consumed",
        BackgroundEvents::Table,
        &[
            BackgroundEvents::UserId,
            BackgroundEvents::Consumed,
            BackgroundEvents::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_background_events_user_type",
        BackgroundEvents::Table,
        &[
            BackgroundEvents::UserId,
            BackgroundEvents::EventType,
            BackgroundEvents::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_background_events_conversation",
        BackgroundEvents::Table,
        &[
            BackgroundEvents::ConversationId,
            BackgroundEvents::CreatedAt,
        ],
        false,
    )
    .await
}

async fn create_alerts(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Alerts::Table)
                .if_not_exists()
                .col(ColumnDef::new(Alerts::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Alerts::UserId).uuid().not_null())
                .col(ColumnDef::new(Alerts::WorkspaceId).uuid().null())
                .col(ColumnDef::new(Alerts::RunId).uuid().null())
                .col(ColumnDef::new(Alerts::AlertType).string_len(64).not_null())
                .col(
                    ColumnDef::new(Alerts::Severity)
                        .string_len(32)
                        .not_null()
                        .default("info"),
                )
                .col(ColumnDef::new(Alerts::Message).text().not_null())
                .col(ColumnDef::new(Alerts::Role).string_len(64).null())
                .col(
                    ColumnDef::new(Alerts::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(ColumnDef::new(Alerts::DedupKey).string_len(255).null())
                .col(
                    ColumnDef::new(Alerts::SnoozedUntil)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alerts::Payload)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(Alerts::ActedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alerts::DismissedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_alerts_user_id")
                        .from(Alerts::Table, Alerts::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_alerts_workspace_id")
                        .from(Alerts::Table, Alerts::WorkspaceId)
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_alerts_run_id")
                        .from(Alerts::Table, Alerts::RunId)
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_alerts_user_status_severity",
        Alerts::Table,
        &[
            Alerts::UserId,
            Alerts::Status,
            Alerts::Severity,
            Alerts::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_alerts_user_type_status",
        Alerts::Table,
        &[
            Alerts::UserId,
            Alerts::AlertType,
            Alerts::Status,
            Alerts::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_alerts_workspace_status",
        Alerts::Table,
        &[Alerts::WorkspaceId, Alerts::Status, Alerts::CreatedAt],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_alerts_run_status",
        Alerts::Table,
        &[Alerts::RunId, Alerts::Status, Alerts::CreatedAt],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_alerts_user_dedup_status",
        Alerts::Table,
        &[Alerts::UserId, Alerts::DedupKey, Alerts::Status],
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

#[derive(DeriveIden, Clone, Copy)]
enum LongRunningTasks {
    Table,
    Id,
    UserId,
    WorkspaceId,
    RunId,
    TaskType,
    Status,
    Title,
    Progress,
    TotalSteps,
    CurrentStep,
    Role,
    InputParams,
    Result,
    ErrorMessage,
    StartedAt,
    CompletedAt,
    CreatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum DailyBriefings {
    Table,
    Id,
    UserId,
    BriefingDate,
    Title,
    Summary,
    Sections,
    ProductSummary,
    AlertSummary,
    TaskSummary,
    Source,
    Status,
    Archived,
    CreatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum BackgroundEvents {
    Table,
    Id,
    UserId,
    ConversationId,
    EventType,
    Title,
    Data,
    Consumed,
    ConsumedAt,
    CreatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum Alerts {
    Table,
    Id,
    UserId,
    WorkspaceId,
    RunId,
    AlertType,
    Severity,
    Message,
    Role,
    Status,
    DedupKey,
    SnoozedUntil,
    Payload,
    ActedAt,
    DismissedAt,
    CreatedAt,
}
