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
ALTER TABLE account_usage_quotas
    ALTER COLUMN question_limit SET DEFAULT 100;

ALTER TABLE team_usage_quotas
    ALTER COLUMN question_limit SET DEFAULT 100;
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
ALTER TABLE account_usage_quotas
    ALTER COLUMN question_limit SET DEFAULT 50;

ALTER TABLE team_usage_quotas
    ALTER COLUMN question_limit SET DEFAULT 50;
"#,
            )
            .await?;

        Ok(())
    }
}
