use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AnalyticsEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AnalyticsEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::EventId)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::EventType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::OccurredAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::ReceivedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::ActorKind)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AnalyticsEvents::UserId).uuid().null())
                    .col(
                        ColumnDef::new(AnalyticsEvents::AnonymousId)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::SessionId)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::SchoolName)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::TeamName)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::RoutePath)
                            .string_len(256)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::PageTitle)
                            .string_len(256)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::Referrer)
                            .string_len(512)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::SourceKind)
                            .string_len(32)
                            .not_null()
                            .default("unknown"),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::SourceHost)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::UtmSource)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::UtmMedium)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::UtmCampaign)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::UtmContent)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::UtmTerm)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::FeatureKey)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::DeviceType)
                            .string_len(32)
                            .not_null()
                            .default("unknown"),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::BrowserName)
                            .string_len(64)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::OsName)
                            .string_len(64)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::ViewportWidth)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::ViewportHeight)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::Status)
                            .string_len(32)
                            .not_null()
                            .default("ok"),
                    )
                    .col(
                        ColumnDef::new(AnalyticsEvents::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AnalyticsSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AnalyticsSessions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::SessionId)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::ActorKind)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AnalyticsSessions::UserId).uuid().null())
                    .col(
                        ColumnDef::new(AnalyticsSessions::AnonymousId)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::SchoolName)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::TeamName)
                            .string_len(128)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::FirstSeenAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::LastSeenAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::EntryPath)
                            .string_len(256)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::ExitPath)
                            .string_len(256)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::SourceKind)
                            .string_len(32)
                            .not_null()
                            .default("unknown"),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::SourceHost)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::PageViewCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AnalyticsSessions::EventCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .to_owned(),
            )
            .await?;

        create_indexes(manager).await?;
        seed_admin_permission(manager).await?;
        create_updated_at_triggers(manager, &["analytics_events", "analytics_sessions"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DELETE FROM admin_menus WHERE permission_code IN (
    'analytics:overview',
    'analytics:accounts',
    'analytics:school_teams'
);
DELETE FROM admin_role_permissions
WHERE permission_id IN (
    SELECT id FROM admin_permissions
    WHERE code IN (
        'analytics:overview',
        'analytics:accounts',
        'analytics:school_teams',
        'analytics'
    )
);
DELETE FROM admin_permissions
WHERE code IN (
    'analytics:overview',
    'analytics:accounts',
    'analytics:school_teams',
    'analytics'
);
"#,
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AnalyticsSessions::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AnalyticsEvents::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for (name, table, columns) in [
        (
            "idx_analytics_events_type_received",
            AnalyticsEvents::Table,
            vec![AnalyticsEvents::EventType, AnalyticsEvents::ReceivedAt],
        ),
        (
            "idx_analytics_events_user_received",
            AnalyticsEvents::Table,
            vec![AnalyticsEvents::UserId, AnalyticsEvents::ReceivedAt],
        ),
        (
            "idx_analytics_events_session",
            AnalyticsEvents::Table,
            vec![AnalyticsEvents::SessionId],
        ),
        (
            "idx_analytics_events_school_team_received",
            AnalyticsEvents::Table,
            vec![
                AnalyticsEvents::SchoolName,
                AnalyticsEvents::TeamName,
                AnalyticsEvents::ReceivedAt,
            ],
        ),
        (
            "idx_analytics_events_source_received",
            AnalyticsEvents::Table,
            vec![AnalyticsEvents::SourceKind, AnalyticsEvents::ReceivedAt],
        ),
        (
            "idx_analytics_events_route_received",
            AnalyticsEvents::Table,
            vec![AnalyticsEvents::RoutePath, AnalyticsEvents::ReceivedAt],
        ),
    ] {
        let mut index = Index::create();
        index.name(name).table(table).if_not_exists();
        for column in columns {
            index.col(column);
        }
        manager.create_index(index.to_owned()).await?;
    }

    for (name, columns) in [
        (
            "idx_analytics_sessions_first_seen",
            vec![AnalyticsSessions::FirstSeenAt],
        ),
        (
            "idx_analytics_sessions_user_seen",
            vec![AnalyticsSessions::UserId, AnalyticsSessions::LastSeenAt],
        ),
        (
            "idx_analytics_sessions_school_team_seen",
            vec![
                AnalyticsSessions::SchoolName,
                AnalyticsSessions::TeamName,
                AnalyticsSessions::LastSeenAt,
            ],
        ),
        (
            "idx_analytics_sessions_source_seen",
            vec![
                AnalyticsSessions::SourceKind,
                AnalyticsSessions::FirstSeenAt,
            ],
        ),
    ] {
        let mut index = Index::create();
        index
            .name(name)
            .table(AnalyticsSessions::Table)
            .if_not_exists();
        for column in columns {
            index.col(column);
        }
        manager.create_index(index.to_owned()).await?;
    }

    Ok(())
}

async fn seed_admin_permission(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            r#"
INSERT INTO admin_permissions (code, name, parent_code, sort, kind)
VALUES ('analytics', 'Analytics', NULL, 40, 'group')
ON CONFLICT (code) DO NOTHING;

INSERT INTO admin_permissions (code, name, parent_code, sort, kind)
VALUES
    ('analytics:overview', 'Analytics Overview', 'analytics', 10, 'action'),
    ('analytics:accounts', 'Analytics Accounts', 'analytics', 20, 'action'),
    ('analytics:school_teams', 'Analytics School Teams', 'analytics', 30, 'action')
ON CONFLICT (code) DO NOTHING;

INSERT INTO admin_menus (name, parent_id, permission_code)
SELECT 'Analytics Overview', NULL, 'analytics:overview'
WHERE NOT EXISTS (
    SELECT 1 FROM admin_menus WHERE permission_code = 'analytics:overview'
);

INSERT INTO admin_menus (name, parent_id, permission_code)
SELECT 'Analytics Accounts', NULL, 'analytics:accounts'
WHERE NOT EXISTS (
    SELECT 1 FROM admin_menus WHERE permission_code = 'analytics:accounts'
);

INSERT INTO admin_menus (name, parent_id, permission_code)
SELECT 'Analytics School Teams', NULL, 'analytics:school_teams'
WHERE NOT EXISTS (
    SELECT 1 FROM admin_menus WHERE permission_code = 'analytics:school_teams'
);

INSERT INTO admin_role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM admin_roles r
CROSS JOIN admin_permissions p
WHERE r.code IN ('root', 'admin')
  AND p.code IN (
      'analytics',
      'analytics:overview',
      'analytics:accounts',
      'analytics:school_teams'
  )
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
enum AnalyticsEvents {
    Table,
    Id,
    EventId,
    EventType,
    OccurredAt,
    ReceivedAt,
    ActorKind,
    UserId,
    AnonymousId,
    SessionId,
    SchoolName,
    TeamName,
    RoutePath,
    PageTitle,
    Referrer,
    SourceKind,
    SourceHost,
    UtmSource,
    UtmMedium,
    UtmCampaign,
    UtmContent,
    UtmTerm,
    FeatureKey,
    DeviceType,
    BrowserName,
    OsName,
    ViewportWidth,
    ViewportHeight,
    Status,
    Metadata,
}

#[derive(DeriveIden)]
enum AnalyticsSessions {
    Table,
    Id,
    SessionId,
    ActorKind,
    UserId,
    AnonymousId,
    SchoolName,
    TeamName,
    FirstSeenAt,
    LastSeenAt,
    EntryPath,
    ExitPath,
    SourceKind,
    SourceHost,
    PageViewCount,
    EventCount,
}
