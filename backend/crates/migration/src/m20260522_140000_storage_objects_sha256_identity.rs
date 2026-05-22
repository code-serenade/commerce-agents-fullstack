use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
UPDATE storage_object_refs r
SET metadata = COALESCE(r.metadata, '{}'::jsonb)
    || jsonb_build_object(
        'bucket', o.bucket,
        'objectKey', o.object_key,
        'legacyStorageObjectId', o.id::text
    )
FROM storage_objects o
WHERE r.storage_object_id = o.id
  AND (r.metadata->>'objectKey') IS NULL;

CREATE TEMP TABLE storage_object_canonical AS
SELECT
    sha256,
    min(id::text)::uuid AS canonical_id,
    max(mime_type) AS mime_type,
    max(file_size) AS file_size,
    jsonb_build_object(
        'migratedToSha256Identity', true,
        'legacyObjectCount', count(*)
    ) AS metadata,
    min(created_at) AS created_at,
    now() AS updated_at
FROM storage_objects
WHERE sha256 IS NOT NULL AND trim(sha256) <> ''
GROUP BY sha256;

UPDATE storage_object_refs r
SET storage_object_id = c.canonical_id
FROM storage_objects o
JOIN storage_object_canonical c ON c.sha256 = o.sha256
WHERE r.storage_object_id = o.id
  AND r.storage_object_id <> c.canonical_id;

UPDATE storage_object_analyses a
SET storage_object_id = c.canonical_id
FROM storage_objects o
JOIN storage_object_canonical c ON c.sha256 = o.sha256
WHERE a.storage_object_id = o.id
  AND a.storage_object_id <> c.canonical_id
  AND NOT EXISTS (
      SELECT 1
      FROM storage_object_analyses existing
      WHERE existing.storage_object_id = c.canonical_id
        AND existing.analysis_kind = a.analysis_kind
        AND existing.prompt_key = a.prompt_key
        AND existing.prompt_version = a.prompt_version
        AND existing.llm_name = a.llm_name
  );

DELETE FROM storage_object_analyses a
USING storage_objects o, storage_object_canonical c
WHERE a.storage_object_id = o.id
  AND c.sha256 = o.sha256
  AND a.storage_object_id <> c.canonical_id;

DELETE FROM storage_objects o
USING storage_object_canonical c
WHERE o.sha256 = c.sha256
  AND o.id <> c.canonical_id;

UPDATE storage_objects o
SET
    mime_type = c.mime_type,
    file_size = c.file_size,
    metadata = COALESCE(o.metadata, '{}'::jsonb) || c.metadata,
    created_at = c.created_at,
    updated_at = now()
FROM storage_object_canonical c
WHERE o.id = c.canonical_id;

DELETE FROM storage_objects
WHERE sha256 IS NULL OR trim(sha256) = '';

DROP INDEX IF EXISTS uk_storage_objects_user_object;
DROP INDEX IF EXISTS idx_storage_objects_user_status_updated;
DROP INDEX IF EXISTS idx_storage_objects_status_orphaned;
DROP INDEX IF EXISTS idx_storage_objects_sha256;

ALTER TABLE storage_objects DROP CONSTRAINT IF EXISTS fk_storage_objects_user_id;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS user_id;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS bucket;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS object_key;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS storage_provider;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS status;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS ref_count;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS last_referenced_at;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS orphaned_at;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS deleted_at;
ALTER TABLE storage_objects ALTER COLUMN sha256 SET NOT NULL;

DROP INDEX IF EXISTS uk_storage_objects_sha256;
ALTER TABLE storage_objects DROP CONSTRAINT IF EXISTS storage_objects_pkey CASCADE;
ALTER TABLE storage_objects ADD PRIMARY KEY (sha256);
ALTER TABLE storage_object_refs ADD COLUMN storage_object_sha256 varchar(64);
UPDATE storage_object_refs r
SET storage_object_sha256 = o.sha256
FROM storage_objects o
WHERE r.storage_object_id = o.id;
ALTER TABLE storage_object_refs ALTER COLUMN storage_object_sha256 SET NOT NULL;
ALTER TABLE storage_object_refs DROP COLUMN storage_object_id;

ALTER TABLE storage_object_analyses ADD COLUMN storage_object_sha256 varchar(64);
UPDATE storage_object_analyses a
SET storage_object_sha256 = o.sha256
FROM storage_objects o
WHERE a.storage_object_id = o.id;
ALTER TABLE storage_object_analyses ALTER COLUMN storage_object_sha256 SET NOT NULL;
ALTER TABLE storage_object_analyses DROP COLUMN storage_object_id;
ALTER TABLE storage_objects DROP COLUMN IF EXISTS id;

CREATE INDEX IF NOT EXISTS idx_storage_object_refs_object_status
ON storage_object_refs (storage_object_sha256, status);
CREATE UNIQUE INDEX IF NOT EXISTS uk_storage_object_refs_active_object_source
ON storage_object_refs (storage_object_sha256, source_type, source_id)
WHERE status = 'active';
CREATE UNIQUE INDEX IF NOT EXISTS uk_storage_object_analyses_identity
ON storage_object_analyses (
    storage_object_sha256,
    analysis_kind,
    prompt_key,
    prompt_version,
    llm_name
);
CREATE INDEX IF NOT EXISTS idx_storage_object_analyses_object_status
ON storage_object_analyses (storage_object_sha256, status);
ALTER TABLE storage_object_refs
ADD CONSTRAINT fk_storage_object_refs_object_sha256
FOREIGN KEY (storage_object_sha256) REFERENCES storage_objects(sha256)
ON DELETE CASCADE;
ALTER TABLE storage_object_analyses
ADD CONSTRAINT fk_storage_object_analyses_object_sha256
FOREIGN KEY (storage_object_sha256) REFERENCES storage_objects(sha256)
ON DELETE CASCADE;
"#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
