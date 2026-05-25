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
                    CREATE TYPE conversation_strategy_id AS ENUM ('learning', 'quick', 'execution');
                EXCEPTION
                    WHEN duplicate_object THEN NULL;
                END
                $$;
                DO $$
                BEGIN
                    CREATE TYPE conversation_response_mode AS ENUM ('learning', 'quick', 'execution');
                EXCEPTION
                    WHEN duplicate_object THEN NULL;
                END
                $$;
                DO $$
                BEGIN
                    CREATE TYPE conversation_routing_mode AS ENUM ('commander', 'fast');
                EXCEPTION
                    WHEN duplicate_object THEN NULL;
                END
                $$;
                DO $$
                BEGIN
                    CREATE TYPE conversation_collaboration_mode AS ENUM ('auto', 'single', 'manual');
                EXCEPTION
                    WHEN duplicate_object THEN NULL;
                END
                $$;
                DO $$
                BEGIN
                    CREATE TYPE conversation_learning_level AS ENUM ('beginner', 'higher_vocational', 'professional');
                EXCEPTION
                    WHEN duplicate_object THEN NULL;
                END
                $$;
                "#,
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ConversationMessages::Table)
                    .add_column(
                        ColumnDef::new(ConversationMessages::StrategyId)
                            .custom(EnumType::StrategyId)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(ConversationMessages::ResponseMode)
                            .custom(EnumType::ResponseMode)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(ConversationMessages::RoutingMode)
                            .custom(EnumType::RoutingMode)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(ConversationMessages::CollaborationMode)
                            .custom(EnumType::CollaborationMode)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(ConversationMessages::LearningLevel)
                            .custom(EnumType::LearningLevel)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE conversation_messages
                SET
                    strategy_id = CASE
                        WHEN content_json #>> '{strategy,strategyId}' IN ('learning', 'quick', 'execution')
                            THEN (content_json #>> '{strategy,strategyId}')::conversation_strategy_id
                        ELSE NULL
                    END,
                    response_mode = CASE
                        WHEN content_json #>> '{strategy,responseMode}' IN ('learning', 'quick', 'execution')
                            THEN (content_json #>> '{strategy,responseMode}')::conversation_response_mode
                        ELSE NULL
                    END,
                    routing_mode = CASE
                        WHEN content_json #>> '{strategy,routingMode}' IN ('commander', 'fast')
                            THEN (content_json #>> '{strategy,routingMode}')::conversation_routing_mode
                        ELSE NULL
                    END,
                    collaboration_mode = CASE
                        WHEN content_json #>> '{strategy,collaborationMode}' IN ('auto', 'single', 'manual')
                            THEN (content_json #>> '{strategy,collaborationMode}')::conversation_collaboration_mode
                        ELSE NULL
                    END,
                    learning_level = CASE
                        WHEN content_json #>> '{strategy,learningLevel}' IN ('beginner', 'higher_vocational', 'professional')
                            THEN (content_json #>> '{strategy,learningLevel}')::conversation_learning_level
                        ELSE NULL
                    END
                WHERE content_json ? 'strategy';
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ConversationMessages::Table)
                    .drop_column(ConversationMessages::LearningLevel)
                    .drop_column(ConversationMessages::CollaborationMode)
                    .drop_column(ConversationMessages::RoutingMode)
                    .drop_column(ConversationMessages::ResponseMode)
                    .drop_column(ConversationMessages::StrategyId)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TYPE IF EXISTS conversation_learning_level;
                DROP TYPE IF EXISTS conversation_collaboration_mode;
                DROP TYPE IF EXISTS conversation_routing_mode;
                DROP TYPE IF EXISTS conversation_response_mode;
                DROP TYPE IF EXISTS conversation_strategy_id;
                "#,
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ConversationMessages {
    Table,
    StrategyId,
    ResponseMode,
    RoutingMode,
    CollaborationMode,
    LearningLevel,
}

#[derive(DeriveIden)]
enum EnumType {
    #[sea_orm(iden = "conversation_strategy_id")]
    StrategyId,
    #[sea_orm(iden = "conversation_response_mode")]
    ResponseMode,
    #[sea_orm(iden = "conversation_routing_mode")]
    RoutingMode,
    #[sea_orm(iden = "conversation_collaboration_mode")]
    CollaborationMode,
    #[sea_orm(iden = "conversation_learning_level")]
    LearningLevel,
}
