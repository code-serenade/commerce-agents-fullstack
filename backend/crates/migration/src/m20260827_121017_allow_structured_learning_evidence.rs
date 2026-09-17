use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(StudentLearningEvidence::Table)
                    .modify_column(
                        ColumnDef::new(StudentLearningEvidence::SourceMessageId)
                            .uuid()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE UNIQUE INDEX IF NOT EXISTS uk_student_learning_evidence_probe \
                 ON student_learning_evidence (probe_id) WHERE probe_id IS NOT NULL;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS uk_student_learning_evidence_probe; \
                 DELETE FROM student_learning_evidence WHERE source_message_id IS NULL;",
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(StudentLearningEvidence::Table)
                    .modify_column(
                        ColumnDef::new(StudentLearningEvidence::SourceMessageId)
                            .uuid()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum StudentLearningEvidence {
    Table,
    SourceMessageId,
}
