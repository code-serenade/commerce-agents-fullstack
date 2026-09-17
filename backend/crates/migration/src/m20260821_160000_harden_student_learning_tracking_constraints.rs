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
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_student_learning_probes_conversation') THEN
        ALTER TABLE student_learning_probes
            ADD CONSTRAINT fk_student_learning_probes_conversation
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_student_learning_probes_assistant_message') THEN
        ALTER TABLE student_learning_probes
            ADD CONSTRAINT fk_student_learning_probes_assistant_message
            FOREIGN KEY (assistant_message_id) REFERENCES conversation_messages(id) ON DELETE SET NULL NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_student_learning_probes_answer_message') THEN
        ALTER TABLE student_learning_probes
            ADD CONSTRAINT fk_student_learning_probes_answer_message
            FOREIGN KEY (answer_message_id) REFERENCES conversation_messages(id) ON DELETE SET NULL NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_student_learning_evidence_conversation') THEN
        ALTER TABLE student_learning_evidence
            ADD CONSTRAINT fk_student_learning_evidence_conversation
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_student_learning_evidence_source_message') THEN
        ALTER TABLE student_learning_evidence
            ADD CONSTRAINT fk_student_learning_evidence_source_message
            FOREIGN KEY (source_message_id) REFERENCES conversation_messages(id) ON DELETE CASCADE NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_student_learning_evidence_probe') THEN
        ALTER TABLE student_learning_evidence
            ADD CONSTRAINT fk_student_learning_evidence_probe
            FOREIGN KEY (probe_id) REFERENCES student_learning_probes(id) ON DELETE SET NULL NOT VALID;
    END IF;
END $$;
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
ALTER TABLE student_learning_evidence
    DROP CONSTRAINT IF EXISTS fk_student_learning_evidence_probe,
    DROP CONSTRAINT IF EXISTS fk_student_learning_evidence_source_message,
    DROP CONSTRAINT IF EXISTS fk_student_learning_evidence_conversation;
ALTER TABLE student_learning_probes
    DROP CONSTRAINT IF EXISTS fk_student_learning_probes_answer_message,
    DROP CONSTRAINT IF EXISTS fk_student_learning_probes_assistant_message,
    DROP CONSTRAINT IF EXISTS fk_student_learning_probes_conversation;
"#,
            )
            .await?;
        Ok(())
    }
}
