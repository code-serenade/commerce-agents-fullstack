use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentMemories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentMemories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AgentMemories::UserId).uuid().not_null())
                    .col(ColumnDef::new(AgentMemories::ConversationId).uuid().null())
                    .col(ColumnDef::new(AgentMemories::SourceMessageId).uuid().null())
                    .col(
                        ColumnDef::new(AgentMemories::SessionId)
                            .string_len(200)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AgentMemories::AgentId)
                            .string_len(100)
                            .not_null()
                            .default("web-agent"),
                    )
                    .col(
                        ColumnDef::new(AgentMemories::Kind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AgentMemories::Key).string_len(200).null())
                    .col(ColumnDef::new(AgentMemories::Content).text().not_null())
                    .col(
                        ColumnDef::new(AgentMemories::Confidence)
                            .double()
                            .not_null()
                            .default(1.0),
                    )
                    .col(
                        ColumnDef::new(AgentMemories::Tags)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .col(
                        ColumnDef::new(AgentMemories::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(
                        ColumnDef::new(AgentMemories::Status)
                            .string_len(32)
                            .not_null()
                            .default("active"),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_memories_user_id")
                            .from(AgentMemories::Table, AgentMemories::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_memories_conversation_id")
                            .from(AgentMemories::Table, AgentMemories::ConversationId)
                            .to(Conversations::Table, Conversations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_memories_source_message_id")
                            .from(AgentMemories::Table, AgentMemories::SourceMessageId)
                            .to(ConversationMessages::Table, ConversationMessages::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        create_indexes(manager).await?;
        create_updated_at_trigger(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(AgentMemories::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_index(
            Index::create()
                .name("idx_agent_memories_user_updated")
                .table(AgentMemories::Table)
                .col(AgentMemories::UserId)
                .col(AgentMemories::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_agent_memories_conversation_updated")
                .table(AgentMemories::Table)
                .col(AgentMemories::ConversationId)
                .col(AgentMemories::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_agent_memories_scope")
                .table(AgentMemories::Table)
                .col(AgentMemories::UserId)
                .col(AgentMemories::ConversationId)
                .col(AgentMemories::AgentId)
                .col(AgentMemories::Kind)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("uk_agent_memories_scope_key")
                .table(AgentMemories::Table)
                .col(AgentMemories::UserId)
                .col(AgentMemories::ConversationId)
                .col(AgentMemories::AgentId)
                .col(AgentMemories::Kind)
                .col(AgentMemories::Key)
                .unique()
                .if_not_exists()
                .to_owned(),
        )
        .await?;

    Ok(())
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

async fn create_updated_at_trigger(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
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

DROP TRIGGER IF EXISTS trg_agent_memories_updated_at ON agent_memories;
CREATE TRIGGER trg_agent_memories_updated_at
BEFORE UPDATE ON agent_memories
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
"#,
        )
        .await?;
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
enum Conversations {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ConversationMessages {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum AgentMemories {
    Table,
    Id,
    UserId,
    ConversationId,
    SourceMessageId,
    SessionId,
    AgentId,
    Kind,
    Key,
    Content,
    Confidence,
    Tags,
    Metadata,
    Status,
    UpdatedAt,
}
