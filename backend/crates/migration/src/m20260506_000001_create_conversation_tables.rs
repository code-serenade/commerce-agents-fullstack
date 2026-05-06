use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Conversations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Conversations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Conversations::UserId).uuid().not_null())
                    .col(ColumnDef::new(Conversations::Title).string_len(200).null())
                    .col(
                        ColumnDef::new(Conversations::Status)
                            .string_len(32)
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(Conversations::Source)
                            .string_len(32)
                            .not_null()
                            .default("web"),
                    )
                    .col(
                        ColumnDef::new(Conversations::AgentId)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Conversations::ModelName)
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Conversations::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .col(
                        ColumnDef::new(Conversations::ArchivedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversations_user_id")
                            .from(Conversations::Table, Conversations::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ConversationMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ConversationMessages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::ConversationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::Role)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::ContentText)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::ContentJson)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::Status)
                            .string_len(32)
                            .not_null()
                            .default("completed"),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::Sequence)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::TokenCount)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ConversationMessages::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversation_messages_conversation_id")
                            .from(
                                ConversationMessages::Table,
                                ConversationMessages::ConversationId,
                            )
                            .to(Conversations::Table, Conversations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversation_messages_user_id")
                            .from(ConversationMessages::Table, ConversationMessages::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ConversationFiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ConversationFiles::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ConversationFiles::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(ConversationFiles::ConversationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ConversationFiles::MessageId).uuid().null())
                    .col(
                        ColumnDef::new(ConversationFiles::Bucket)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::ObjectKey)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::FileName)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::MimeType)
                            .string_len(120)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::FileSize)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::Sha256)
                            .string_len(64)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::Visibility)
                            .string_len(32)
                            .not_null()
                            .default("private"),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::StorageProvider)
                            .string_len(32)
                            .not_null()
                            .default("s3"),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::Status)
                            .string_len(32)
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(ConversationFiles::Metadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(created_at_col())
                    .col(
                        ColumnDef::new(ConversationFiles::DeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversation_files_user_id")
                            .from(ConversationFiles::Table, ConversationFiles::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversation_files_conversation_id")
                            .from(ConversationFiles::Table, ConversationFiles::ConversationId)
                            .to(Conversations::Table, Conversations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversation_files_message_id")
                            .from(ConversationFiles::Table, ConversationFiles::MessageId)
                            .to(ConversationMessages::Table, ConversationMessages::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        create_indexes(manager).await?;
        create_updated_at_triggers(manager, &["conversations", "conversation_messages"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ConversationFiles::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ConversationMessages::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Conversations::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

async fn create_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_index(
            Index::create()
                .name("idx_conversations_user_updated")
                .table(Conversations::Table)
                .col(Conversations::UserId)
                .col(Conversations::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_conversations_user_created")
                .table(Conversations::Table)
                .col(Conversations::UserId)
                .col(Conversations::CreatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_conversations_user_status")
                .table(Conversations::Table)
                .col(Conversations::UserId)
                .col(Conversations::Status)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("uk_conversation_messages_sequence")
                .table(ConversationMessages::Table)
                .col(ConversationMessages::ConversationId)
                .col(ConversationMessages::Sequence)
                .unique()
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_conversation_messages_created")
                .table(ConversationMessages::Table)
                .col(ConversationMessages::ConversationId)
                .col(ConversationMessages::CreatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_conversation_messages_updated")
                .table(ConversationMessages::Table)
                .col(ConversationMessages::ConversationId)
                .col(ConversationMessages::UpdatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_conversation_files_conversation")
                .table(ConversationFiles::Table)
                .col(ConversationFiles::ConversationId)
                .col(ConversationFiles::CreatedAt)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_conversation_files_message")
                .table(ConversationFiles::Table)
                .col(ConversationFiles::MessageId)
                .if_not_exists()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("uk_conversation_files_object")
                .table(ConversationFiles::Table)
                .col(ConversationFiles::Bucket)
                .col(ConversationFiles::ObjectKey)
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
enum Conversations {
    Table,
    Id,
    UserId,
    Title,
    Status,
    Source,
    AgentId,
    ModelName,
    Metadata,
    CreatedAt,
    UpdatedAt,
    ArchivedAt,
}

#[derive(DeriveIden)]
enum ConversationMessages {
    Table,
    Id,
    ConversationId,
    UserId,
    Role,
    ContentText,
    ContentJson,
    Status,
    Sequence,
    TokenCount,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ConversationFiles {
    Table,
    Id,
    UserId,
    ConversationId,
    MessageId,
    Bucket,
    ObjectKey,
    FileName,
    MimeType,
    FileSize,
    Sha256,
    Visibility,
    StorageProvider,
    Status,
    Metadata,
    CreatedAt,
    DeletedAt,
}
