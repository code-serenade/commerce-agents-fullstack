use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_platform_connections(manager).await?;
        create_channel_connections(manager).await?;
        create_store_metrics(manager).await?;
        create_updated_at_triggers(manager, &["platform_connections", "channel_connections"])
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(StoreMetrics::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ChannelConnections::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(PlatformConnections::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_platform_connections(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(PlatformConnections::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(PlatformConnections::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::UserId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::Platform)
                        .string_len(64)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::AppKey)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::AppSecretEnc)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::AccessTokenEnc)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::RefreshTokenEnc)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::TokenExpiresAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::ShopId)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::ShopName)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::Enabled)
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .col(
                    ColumnDef::new(PlatformConnections::Status)
                        .string_len(32)
                        .not_null()
                        .default("configured"),
                )
                .col(
                    ColumnDef::new(PlatformConnections::LastTestedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::TestResult)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(PlatformConnections::LastSyncedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::SyncStatus)
                        .string_len(32)
                        .not_null()
                        .default("idle"),
                )
                .col(
                    ColumnDef::new(PlatformConnections::SyncMessage)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(PlatformConnections::Extra)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_platform_connections_user_id")
                        .from(PlatformConnections::Table, PlatformConnections::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_platform_connections_user_platform",
        PlatformConnections::Table,
        &[PlatformConnections::UserId, PlatformConnections::Platform],
        true,
    )
    .await?;
    create_index(
        manager,
        "idx_platform_connections_user_enabled",
        PlatformConnections::Table,
        &[PlatformConnections::UserId, PlatformConnections::Enabled],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_platform_connections_user_updated",
        PlatformConnections::Table,
        &[PlatformConnections::UserId, PlatformConnections::UpdatedAt],
        false,
    )
    .await
}

async fn create_channel_connections(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ChannelConnections::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ChannelConnections::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(ChannelConnections::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(ChannelConnections::Channel)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(ChannelConnections::WebhookUrl).text().null())
                .col(
                    ColumnDef::new(ChannelConnections::BotTokenEnc)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(ChannelConnections::ChatId)
                        .string_len(255)
                        .null(),
                )
                .col(ColumnDef::new(ChannelConnections::SecretEnc).text().null())
                .col(
                    ColumnDef::new(ChannelConnections::ExtraCredentials)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(
                    ColumnDef::new(ChannelConnections::Enabled)
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .col(
                    ColumnDef::new(ChannelConnections::Status)
                        .string_len(32)
                        .not_null()
                        .default("configured"),
                )
                .col(
                    ColumnDef::new(ChannelConnections::LastTestedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(ColumnDef::new(ChannelConnections::LastError).text().null())
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_channel_connections_user_id")
                        .from(ChannelConnections::Table, ChannelConnections::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_channel_connections_user_channel",
        ChannelConnections::Table,
        &[ChannelConnections::UserId, ChannelConnections::Channel],
        true,
    )
    .await?;
    create_index(
        manager,
        "idx_channel_connections_user_enabled",
        ChannelConnections::Table,
        &[ChannelConnections::UserId, ChannelConnections::Enabled],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_channel_connections_user_updated",
        ChannelConnections::Table,
        &[ChannelConnections::UserId, ChannelConnections::UpdatedAt],
        false,
    )
    .await
}

async fn create_store_metrics(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(StoreMetrics::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(StoreMetrics::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(StoreMetrics::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(StoreMetrics::Platform)
                        .string_len(64)
                        .not_null()
                        .default("general"),
                )
                .col(
                    ColumnDef::new(StoreMetrics::MetricDate)
                        .string_len(32)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(StoreMetrics::MetricKey)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(StoreMetrics::MetricValue)
                        .double()
                        .not_null()
                        .default(0.0),
                )
                .col(
                    ColumnDef::new(StoreMetrics::DimensionKey)
                        .string_len(128)
                        .not_null()
                        .default(""),
                )
                .col(
                    ColumnDef::new(StoreMetrics::DimensionValue)
                        .string_len(255)
                        .not_null()
                        .default(""),
                )
                .col(
                    ColumnDef::new(StoreMetrics::Source)
                        .string_len(64)
                        .not_null()
                        .default("import"),
                )
                .col(created_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_store_metrics_user_id")
                        .from(StoreMetrics::Table, StoreMetrics::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_store_metrics_user_date_platform",
        StoreMetrics::Table,
        &[
            StoreMetrics::UserId,
            StoreMetrics::MetricDate,
            StoreMetrics::Platform,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_store_metrics_dedup",
        StoreMetrics::Table,
        &[
            StoreMetrics::UserId,
            StoreMetrics::Platform,
            StoreMetrics::MetricDate,
            StoreMetrics::MetricKey,
            StoreMetrics::DimensionKey,
            StoreMetrics::DimensionValue,
        ],
        true,
    )
    .await
}

async fn create_index<T>(
    manager: &SchemaManager<'_>,
    name: &str,
    table: T,
    columns: &[T],
    unique: bool,
) -> Result<(), DbErr>
where
    T: Iden + Copy + 'static,
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

#[derive(DeriveIden, Copy, Clone)]
enum PlatformConnections {
    Table,
    Id,
    UserId,
    Platform,
    AppKey,
    AppSecretEnc,
    AccessTokenEnc,
    RefreshTokenEnc,
    TokenExpiresAt,
    ShopId,
    ShopName,
    Enabled,
    Status,
    LastTestedAt,
    TestResult,
    LastSyncedAt,
    SyncStatus,
    SyncMessage,
    Extra,
    UpdatedAt,
}

#[derive(DeriveIden, Copy, Clone)]
enum ChannelConnections {
    Table,
    Id,
    UserId,
    Channel,
    WebhookUrl,
    BotTokenEnc,
    ChatId,
    SecretEnc,
    ExtraCredentials,
    Enabled,
    Status,
    LastTestedAt,
    LastError,
    UpdatedAt,
}

#[derive(DeriveIden, Copy, Clone)]
enum StoreMetrics {
    Table,
    Id,
    UserId,
    Platform,
    MetricDate,
    MetricKey,
    MetricValue,
    DimensionKey,
    DimensionValue,
    Source,
}
