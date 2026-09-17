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
                DROP INDEX IF EXISTS uk_student_learning_probes_active;
                CREATE UNIQUE INDEX IF NOT EXISTS uk_student_learning_probes_active_message
                    ON student_learning_probes (user_id, assistant_message_id)
                    WHERE status IN ('planned', 'offered') AND assistant_message_id IS NOT NULL;
                CREATE UNIQUE INDEX IF NOT EXISTS uk_student_learning_probes_active_legacy
                    ON student_learning_probes (user_id, knowledge_code)
                    WHERE status IN ('planned', 'offered') AND assistant_message_id IS NULL;
                "#,
            )
            .await
            .map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP INDEX IF EXISTS uk_student_learning_probes_active_message;
                DROP INDEX IF EXISTS uk_student_learning_probes_active_legacy;
                WITH ranked AS (
                    SELECT id,
                           ROW_NUMBER() OVER (
                               PARTITION BY user_id, knowledge_code
                               ORDER BY created_at DESC, id DESC
                           ) AS position
                    FROM student_learning_probes
                    WHERE status IN ('planned', 'offered')
                )
                UPDATE student_learning_probes AS probes
                SET status = 'superseded', updated_at = CURRENT_TIMESTAMP
                FROM ranked
                WHERE probes.id = ranked.id AND ranked.position > 1;
                CREATE UNIQUE INDEX IF NOT EXISTS uk_student_learning_probes_active
                    ON student_learning_probes (user_id, knowledge_code)
                    WHERE status IN ('planned', 'offered');
                "#,
            )
            .await
            .map(|_| ())
    }
}
