use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StudentLearningQuestionGaps::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::ConversationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::AssistantMessageId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::FrameworkVersion)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::SelectorVersion)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::KnowledgeCode)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::ReasonCode)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::TopicSignals)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .col(
                        ColumnDef::new(StudentLearningQuestionGaps::ScopeSignals)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(audit_col(StudentLearningQuestionGaps::CreatedAt))
                    .col(audit_col(StudentLearningQuestionGaps::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_learning_question_gaps_user")
                            .from(
                                StudentLearningQuestionGaps::Table,
                                StudentLearningQuestionGaps::UserId,
                            )
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_learning_question_gaps_conversation")
                            .from(
                                StudentLearningQuestionGaps::Table,
                                StudentLearningQuestionGaps::ConversationId,
                            )
                            .to(Conversations::Table, Conversations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_learning_question_gaps_assistant_message")
                            .from(
                                StudentLearningQuestionGaps::Table,
                                StudentLearningQuestionGaps::AssistantMessageId,
                            )
                            .to(ConversationMessages::Table, ConversationMessages::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_student_learning_question_gaps_domain_reason_created ON student_learning_question_gaps (knowledge_code, reason_code, created_at);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(StudentLearningQuestionGaps::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum StudentLearningQuestionGaps {
    Table,
    Id,
    UserId,
    ConversationId,
    AssistantMessageId,
    FrameworkVersion,
    SelectorVersion,
    KnowledgeCode,
    ReasonCode,
    TopicSignals,
    ScopeSignals,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AppUsers {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum Conversations {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ConversationMessages {
    Table,
    Id,
}

fn audit_col<T: IntoIden>(name: T) -> ColumnDef {
    let mut col = ColumnDef::new(name);
    col.timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp());
    col
}
