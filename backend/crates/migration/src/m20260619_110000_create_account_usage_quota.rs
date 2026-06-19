use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccountUsageQuotas::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AccountUsageQuotas::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AccountUsageQuotas::UserId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AccountUsageQuotas::QuestionLimit)
                            .integer()
                            .not_null()
                            .default(50),
                    )
                    .col(
                        ColumnDef::new(AccountUsageQuotas::Status)
                            .string_len(32)
                            .not_null()
                            .default("enabled"),
                    )
                    .col(
                        ColumnDef::new(AccountUsageQuotas::Remark)
                            .string_len(255)
                            .null(),
                    )
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_account_usage_quotas_user_id")
                            .from(AccountUsageQuotas::Table, AccountUsageQuotas::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_account_usage_quotas_status_updated")
                    .table(AccountUsageQuotas::Table)
                    .col(AccountUsageQuotas::Status)
                    .col(AccountUsageQuotas::UpdatedAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
        create_updated_at_trigger(manager).await?;
        update_usage_admin_labels(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(AccountUsageQuotas::Table)
                    .cascade()
                    .to_owned(),
            )
            .await
    }
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
DROP TRIGGER IF EXISTS trg_account_usage_quotas_updated_at ON account_usage_quotas;
CREATE TRIGGER trg_account_usage_quotas_updated_at
BEFORE UPDATE ON account_usage_quotas
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
"#,
        )
        .await?;
    Ok(())
}

async fn update_usage_admin_labels(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            r#"
UPDATE admin_permissions
SET name = 'Account Usage'
WHERE code = 'usage:team_usage';

UPDATE admin_menus
SET name = 'Account Usage'
WHERE permission_code = 'usage:team_usage';
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
enum AccountUsageQuotas {
    Table,
    Id,
    UserId,
    QuestionLimit,
    Status,
    Remark,
    UpdatedAt,
}
