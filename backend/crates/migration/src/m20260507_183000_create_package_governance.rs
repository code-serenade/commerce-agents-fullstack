use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_package_registry(manager).await?;
        create_package_state_history(manager).await?;
        create_workspace_run_package_locks(manager).await?;
        create_updated_at_triggers(manager, &["package_registry"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(WorkspaceRunPackageLocks::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(PackageStateHistory::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(PackageRegistry::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_package_registry(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(PackageRegistry::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(PackageRegistry::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(PackageRegistry::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(PackageRegistry::PackageType)
                        .string_len(32)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(PackageRegistry::PackageId)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(PackageRegistry::PackageName)
                        .string_len(255)
                        .not_null()
                        .default(""),
                )
                .col(
                    ColumnDef::new(PackageRegistry::Version)
                        .string_len(64)
                        .not_null()
                        .default("0.1.0"),
                )
                .col(
                    ColumnDef::new(PackageRegistry::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(ColumnDef::new(PackageRegistry::ManifestPath).text().null())
                .col(jsonb_col(PackageRegistry::Dependencies, "'{}'::jsonb"))
                .col(jsonb_col(PackageRegistry::Compatibility, "'{}'::jsonb"))
                .col(jsonb_col(PackageRegistry::Metadata, "'{}'::jsonb"))
                .col(ColumnDef::new(PackageRegistry::UpdatedBy).uuid().null())
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_package_registry_user_id")
                        .from(PackageRegistry::Table, PackageRegistry::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_package_registry_updated_by")
                        .from(PackageRegistry::Table, PackageRegistry::UpdatedBy)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_package_registry_user_type_package_unique",
        PackageRegistry::Table,
        &[
            PackageRegistry::UserId,
            PackageRegistry::PackageType,
            PackageRegistry::PackageId,
        ],
        true,
    )
    .await?;
    create_index(
        manager,
        "idx_package_registry_user_status",
        PackageRegistry::Table,
        &[
            PackageRegistry::UserId,
            PackageRegistry::Status,
            PackageRegistry::UpdatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_package_registry_user_type_status",
        PackageRegistry::Table,
        &[
            PackageRegistry::UserId,
            PackageRegistry::PackageType,
            PackageRegistry::Status,
            PackageRegistry::UpdatedAt,
        ],
        false,
    )
    .await
}

async fn create_package_state_history(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(PackageStateHistory::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(PackageStateHistory::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(PackageStateHistory::UserId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(PackageStateHistory::PackageRegistryId)
                        .uuid()
                        .null(),
                )
                .col(
                    ColumnDef::new(PackageStateHistory::PackageType)
                        .string_len(32)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(PackageStateHistory::PackageId)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(PackageStateHistory::FromStatus)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(PackageStateHistory::ToStatus)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(ColumnDef::new(PackageStateHistory::Reason).text().null())
                .col(ColumnDef::new(PackageStateHistory::ChangedBy).uuid().null())
                .col(jsonb_col(PackageStateHistory::Metadata, "'{}'::jsonb"))
                .col(created_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_package_state_history_user_id")
                        .from(PackageStateHistory::Table, PackageStateHistory::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_package_state_history_registry_id")
                        .from(
                            PackageStateHistory::Table,
                            PackageStateHistory::PackageRegistryId,
                        )
                        .to(PackageRegistry::Table, PackageRegistry::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_package_state_history_changed_by")
                        .from(PackageStateHistory::Table, PackageStateHistory::ChangedBy)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_package_state_history_user_package_created",
        PackageStateHistory::Table,
        &[
            PackageStateHistory::UserId,
            PackageStateHistory::PackageType,
            PackageStateHistory::PackageId,
            PackageStateHistory::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_package_state_history_registry_created",
        PackageStateHistory::Table,
        &[
            PackageStateHistory::PackageRegistryId,
            PackageStateHistory::CreatedAt,
        ],
        false,
    )
    .await
}

async fn create_workspace_run_package_locks(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(WorkspaceRunPackageLocks::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::RunId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::WorkspaceId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::UserId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::PackageRegistryId)
                        .uuid()
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::PackageType)
                        .string_len(32)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::PackageId)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::PackageName)
                        .string_len(255)
                        .not_null()
                        .default(""),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::LockedVersion)
                        .string_len(64)
                        .not_null()
                        .default("0.1.0"),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::LockedStatus)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::Required)
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::LockSource)
                        .string_len(128)
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::ManifestPath)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(WorkspaceRunPackageLocks::Notes)
                        .text()
                        .null(),
                )
                .col(jsonb_col(WorkspaceRunPackageLocks::Metadata, "'{}'::jsonb"))
                .col(created_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_run_package_locks_run_id")
                        .from(
                            WorkspaceRunPackageLocks::Table,
                            WorkspaceRunPackageLocks::RunId,
                        )
                        .to(WorkspaceRuns::Table, WorkspaceRuns::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_run_package_locks_workspace_id")
                        .from(
                            WorkspaceRunPackageLocks::Table,
                            WorkspaceRunPackageLocks::WorkspaceId,
                        )
                        .to(Workspaces::Table, Workspaces::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_run_package_locks_user_id")
                        .from(
                            WorkspaceRunPackageLocks::Table,
                            WorkspaceRunPackageLocks::UserId,
                        )
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_workspace_run_package_locks_registry_id")
                        .from(
                            WorkspaceRunPackageLocks::Table,
                            WorkspaceRunPackageLocks::PackageRegistryId,
                        )
                        .to(PackageRegistry::Table, PackageRegistry::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    create_index(
        manager,
        "idx_workspace_run_package_locks_run_package_unique",
        WorkspaceRunPackageLocks::Table,
        &[
            WorkspaceRunPackageLocks::RunId,
            WorkspaceRunPackageLocks::PackageType,
            WorkspaceRunPackageLocks::PackageId,
        ],
        true,
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_run_package_locks_user_workspace_run",
        WorkspaceRunPackageLocks::Table,
        &[
            WorkspaceRunPackageLocks::UserId,
            WorkspaceRunPackageLocks::WorkspaceId,
            WorkspaceRunPackageLocks::RunId,
            WorkspaceRunPackageLocks::CreatedAt,
        ],
        false,
    )
    .await?;
    create_index(
        manager,
        "idx_workspace_run_package_locks_registry",
        WorkspaceRunPackageLocks::Table,
        &[WorkspaceRunPackageLocks::PackageRegistryId],
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

#[derive(DeriveIden, Clone, Copy)]
enum PackageRegistry {
    Table,
    Id,
    UserId,
    PackageType,
    PackageId,
    PackageName,
    Version,
    Status,
    ManifestPath,
    Dependencies,
    Compatibility,
    Metadata,
    UpdatedBy,
    UpdatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum PackageStateHistory {
    Table,
    Id,
    UserId,
    PackageRegistryId,
    PackageType,
    PackageId,
    FromStatus,
    ToStatus,
    Reason,
    ChangedBy,
    Metadata,
    CreatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum WorkspaceRunPackageLocks {
    Table,
    Id,
    RunId,
    WorkspaceId,
    UserId,
    PackageRegistryId,
    PackageType,
    PackageId,
    PackageName,
    LockedVersion,
    LockedStatus,
    Required,
    LockSource,
    ManifestPath,
    Notes,
    Metadata,
    CreatedAt,
}
