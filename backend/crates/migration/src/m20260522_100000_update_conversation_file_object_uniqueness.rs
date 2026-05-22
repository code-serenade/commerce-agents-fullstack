use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS uk_conversation_files_object;")
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE UNIQUE INDEX IF NOT EXISTS uk_conversation_files_active_object
ON conversation_files (conversation_id, bucket, object_key)
WHERE status = 'active';
"#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS uk_conversation_files_active_object;")
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE UNIQUE INDEX IF NOT EXISTS uk_conversation_files_object
ON conversation_files (bucket, object_key);
"#,
            )
            .await?;

        Ok(())
    }
}
