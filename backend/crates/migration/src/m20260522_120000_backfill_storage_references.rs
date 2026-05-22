use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        update_product_asset_object_uniqueness(manager).await?;
        backfill_storage_objects(manager).await?;
        backfill_storage_refs(manager).await?;
        refresh_storage_object_ref_counts(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DELETE FROM storage_object_refs
WHERE metadata->>'backfilledFrom' IN ('conversation_files', 'product_assets');

DELETE FROM storage_objects
WHERE metadata->>'backfilled' = 'true'
  AND NOT EXISTS (
    SELECT 1 FROM storage_object_refs
    WHERE storage_object_refs.storage_object_id = storage_objects.id
  );
"#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS uk_product_assets_active_object;")
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_assets_user_object_key
ON product_assets (user_id, object_key);
"#,
            )
            .await?;

        Ok(())
    }
}

async fn update_product_asset_object_uniqueness(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared("DROP INDEX IF EXISTS idx_product_assets_user_object_key;")
        .await?;
    manager
        .get_connection()
        .execute_unprepared(
            r#"
CREATE UNIQUE INDEX IF NOT EXISTS uk_product_assets_active_object
ON product_assets (product_id, bucket, object_key)
WHERE status = 'active';
"#,
        )
        .await?;
    Ok(())
}

async fn backfill_storage_objects(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            r#"
INSERT INTO storage_objects (
    id,
    user_id,
    bucket,
    object_key,
    sha256,
    mime_type,
    file_size,
    storage_provider,
    status,
    ref_count,
    last_referenced_at,
    metadata,
    created_at,
    updated_at
)
SELECT
    md5(concat('storage_object:', user_id::text, ':', bucket, ':', object_key))::uuid,
    user_id,
    bucket,
    object_key,
    max(sha256),
    max(mime_type),
    max(file_size),
    's3',
    'active',
    0,
    now(),
    jsonb_build_object('backfilled', true, 'backfilledFrom', 'conversation_files'),
    min(created_at),
    now()
FROM conversation_files
WHERE status = 'active'
GROUP BY user_id, bucket, object_key
ON CONFLICT (user_id, bucket, object_key) DO NOTHING;

INSERT INTO storage_objects (
    id,
    user_id,
    bucket,
    object_key,
    sha256,
    mime_type,
    file_size,
    storage_provider,
    status,
    ref_count,
    last_referenced_at,
    metadata,
    created_at,
    updated_at
)
SELECT
    md5(concat('storage_object:', user_id::text, ':', bucket, ':', object_key))::uuid,
    user_id,
    bucket,
    object_key,
    null,
    max(mime_type),
    max(file_size),
    's3',
    'active',
    0,
    now(),
    jsonb_build_object('backfilled', true, 'backfilledFrom', 'product_assets'),
    min(created_at),
    now()
FROM product_assets
WHERE status = 'active'
GROUP BY user_id, bucket, object_key
ON CONFLICT (user_id, bucket, object_key) DO NOTHING;
"#,
        )
        .await?;
    Ok(())
}

async fn backfill_storage_refs(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            r#"
INSERT INTO storage_object_refs (
    id,
    storage_object_id,
    user_id,
    source_type,
    source_id,
    status,
    metadata,
    created_at,
    updated_at
)
SELECT
    md5(concat('storage_ref:conversation_file:', f.id::text))::uuid,
    o.id,
    f.user_id,
    'conversation_file',
    f.id,
    'active',
    jsonb_build_object('backfilledFrom', 'conversation_files'),
    f.created_at,
    now()
FROM conversation_files f
JOIN storage_objects o
  ON o.user_id = f.user_id
 AND o.bucket = f.bucket
 AND o.object_key = f.object_key
WHERE f.status = 'active'
ON CONFLICT DO NOTHING;

INSERT INTO storage_object_refs (
    id,
    storage_object_id,
    user_id,
    source_type,
    source_id,
    status,
    metadata,
    created_at,
    updated_at
)
SELECT
    md5(concat('storage_ref:product_asset:', a.id::text))::uuid,
    o.id,
    a.user_id,
    'product_asset',
    a.id,
    'active',
    jsonb_build_object('backfilledFrom', 'product_assets'),
    a.created_at,
    now()
FROM product_assets a
JOIN storage_objects o
  ON o.user_id = a.user_id
 AND o.bucket = a.bucket
 AND o.object_key = a.object_key
WHERE a.status = 'active'
ON CONFLICT DO NOTHING;
"#,
        )
        .await?;
    Ok(())
}

async fn refresh_storage_object_ref_counts(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            r#"
UPDATE storage_objects o
SET
    ref_count = counts.ref_count,
    status = CASE WHEN counts.ref_count > 0 THEN 'active' ELSE 'orphaned' END,
    orphaned_at = CASE WHEN counts.ref_count > 0 THEN null ELSE COALESCE(o.orphaned_at, now()) END,
    last_referenced_at = CASE WHEN counts.ref_count > 0 THEN now() ELSE o.last_referenced_at END,
    updated_at = now()
FROM (
    SELECT o.id, count(r.id)::bigint AS ref_count
    FROM storage_objects o
    LEFT JOIN storage_object_refs r
      ON r.storage_object_id = o.id
     AND r.status = 'active'
    GROUP BY o.id
) counts
WHERE o.id = counts.id;
"#,
        )
        .await?;
    Ok(())
}
