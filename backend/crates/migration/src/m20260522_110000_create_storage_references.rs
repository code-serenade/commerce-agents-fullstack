use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_storage_objects(manager).await?;
        create_storage_object_refs(manager).await?;
        create_updated_at_triggers(manager, &["storage_objects", "storage_object_refs"]).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(StorageObjectRefs::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(StorageObjects::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_storage_objects(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(StorageObjects::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(StorageObjects::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(StorageObjects::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(StorageObjects::Bucket)
                        .string_len(128)
                        .not_null(),
                )
                .col(ColumnDef::new(StorageObjects::ObjectKey).text().not_null())
                .col(ColumnDef::new(StorageObjects::Sha256).string_len(64).null())
                .col(
                    ColumnDef::new(StorageObjects::MimeType)
                        .string_len(128)
                        .null(),
                )
                .col(
                    ColumnDef::new(StorageObjects::FileSize)
                        .big_integer()
                        .null(),
                )
                .col(
                    ColumnDef::new(StorageObjects::StorageProvider)
                        .string_len(32)
                        .not_null()
                        .default("s3"),
                )
                .col(
                    ColumnDef::new(StorageObjects::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(StorageObjects::RefCount)
                        .big_integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(StorageObjects::LastReferencedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(StorageObjects::OrphanedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(StorageObjects::DeletedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(StorageObjects::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_storage_objects_user_id")
                        .from(StorageObjects::Table, StorageObjects::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("uk_storage_objects_user_object")
                .table(StorageObjects::Table)
                .col(StorageObjects::UserId)
                .col(StorageObjects::Bucket)
                .col(StorageObjects::ObjectKey)
                .unique()
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_storage_objects_user_status_updated")
                .table(StorageObjects::Table)
                .col(StorageObjects::UserId)
                .col(StorageObjects::Status)
                .col(StorageObjects::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_storage_objects_status_orphaned")
                .table(StorageObjects::Table)
                .col(StorageObjects::Status)
                .col(StorageObjects::OrphanedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_storage_objects_sha256")
                .table(StorageObjects::Table)
                .col(StorageObjects::Sha256)
                .if_not_exists()
                .to_owned(),
        )
        .await
}

async fn create_storage_object_refs(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(StorageObjectRefs::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(StorageObjectRefs::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(StorageObjectRefs::StorageObjectId)
                        .uuid()
                        .not_null(),
                )
                .col(ColumnDef::new(StorageObjectRefs::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(StorageObjectRefs::SourceType)
                        .string_len(64)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(StorageObjectRefs::SourceId)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(StorageObjectRefs::Status)
                        .string_len(32)
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(StorageObjectRefs::ReleasedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(StorageObjectRefs::Metadata)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(created_at_col())
                .col(updated_at_col())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_storage_object_refs_object_id")
                        .from(StorageObjectRefs::Table, StorageObjectRefs::StorageObjectId)
                        .to(StorageObjects::Table, StorageObjects::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_storage_object_refs_user_id")
                        .from(StorageObjectRefs::Table, StorageObjectRefs::UserId)
                        .to(AppUsers::Table, AppUsers::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .get_connection()
        .execute_unprepared(
            r#"
CREATE UNIQUE INDEX IF NOT EXISTS uk_storage_object_refs_active_source
ON storage_object_refs (source_type, source_id)
WHERE status = 'active';
"#,
        )
        .await?;
    manager
        .get_connection()
        .execute_unprepared(
            r#"
CREATE UNIQUE INDEX IF NOT EXISTS uk_storage_object_refs_active_object_source
ON storage_object_refs (storage_object_id, source_type, source_id)
WHERE status = 'active';
"#,
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_storage_object_refs_object_status")
                .table(StorageObjectRefs::Table)
                .col(StorageObjectRefs::StorageObjectId)
                .col(StorageObjectRefs::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_storage_object_refs_user_source_status")
                .table(StorageObjectRefs::Table)
                .col(StorageObjectRefs::UserId)
                .col(StorageObjectRefs::SourceType)
                .col(StorageObjectRefs::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await
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
enum StorageObjects {
    Table,
    Id,
    UserId,
    Bucket,
    ObjectKey,
    Sha256,
    MimeType,
    FileSize,
    StorageProvider,
    Status,
    RefCount,
    LastReferencedAt,
    OrphanedAt,
    DeletedAt,
    Metadata,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum StorageObjectRefs {
    Table,
    Id,
    StorageObjectId,
    UserId,
    SourceType,
    SourceId,
    Status,
    ReleasedAt,
    Metadata,
}
