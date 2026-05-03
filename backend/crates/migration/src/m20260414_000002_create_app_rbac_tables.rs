use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AppRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppRoles::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AppRoles::Name).string().not_null())
                    .col(ColumnDef::new(AppRoles::Code).string().not_null())
                    .col(created_at_col())
                    .index(
                        Index::create()
                            .name("uk_app_roles_code")
                            .col(AppRoles::Code)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AppPermissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppPermissions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AppPermissions::Code).string().not_null())
                    .col(ColumnDef::new(AppPermissions::Name).string().not_null())
                    .col(ColumnDef::new(AppPermissions::ParentCode).string().null())
                    .col(
                        ColumnDef::new(AppPermissions::Sort)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(AppPermissions::Kind).string().not_null())
                    .col(created_at_col())
                    .col(updated_at_col())
                    .index(
                        Index::create()
                            .name("uk_app_permissions_code")
                            .col(AppPermissions::Code)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_app_permissions_parent_code")
                            .from(AppPermissions::Table, AppPermissions::ParentCode)
                            .to(AppPermissions::Table, AppPermissions::Code)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AppUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppUsers::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AppUsers::DisplayId)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AppUsers::DisplayName)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AppUsers::Remark).string_len(255).null())
                    .col(ColumnDef::new(AppUsers::Status).string().not_null())
                    .col(created_at_col())
                    .col(updated_at_col())
                    .index(
                        Index::create()
                            .name("uk_app_users_display_id")
                            .col(AppUsers::DisplayId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AppUserRoles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AppUserRoles::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(AppUserRoles::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(created_at_col())
                    .primary_key(
                        Index::create()
                            .name("pk_app_user_roles")
                            .col(AppUserRoles::UserId)
                            .col(AppUserRoles::RoleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_app_user_roles_user_id")
                            .from(AppUserRoles::Table, AppUserRoles::UserId)
                            .to(AppUsers::Table, AppUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_app_user_roles_role_id")
                            .from(AppUserRoles::Table, AppUserRoles::RoleId)
                            .to(AppRoles::Table, AppRoles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AppRolePermissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppRolePermissions::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AppRolePermissions::PermissionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(created_at_col())
                    .primary_key(
                        Index::create()
                            .name("pk_app_role_permissions")
                            .col(AppRolePermissions::RoleId)
                            .col(AppRolePermissions::PermissionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_app_role_permissions_role_id")
                            .from(AppRolePermissions::Table, AppRolePermissions::RoleId)
                            .to(AppRoles::Table, AppRoles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_app_role_permissions_permission_id")
                            .from(AppRolePermissions::Table, AppRolePermissions::PermissionId)
                            .to(AppPermissions::Table, AppPermissions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        create_updated_at_triggers(manager, &["app_permissions", "app_users"]).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(AppRolePermissions::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AppUserRoles::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AppUsers::Table).cascade().to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AppPermissions::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AppRoles::Table).cascade().to_owned())
            .await?;

        Ok(())
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
enum AppRoles {
    Table,
    Id,
    Name,
    Code,
}

#[derive(DeriveIden)]
enum AppPermissions {
    Table,
    Id,
    Code,
    Name,
    ParentCode,
    Sort,
    Kind,
}

#[derive(DeriveIden)]
enum AppUsers {
    Table,
    UserId,
    DisplayId,
    DisplayName,
    Remark,
    Status,
}

#[derive(DeriveIden)]
enum AppUserRoles {
    Table,
    UserId,
    RoleId,
}

#[derive(DeriveIden)]
enum AppRolePermissions {
    Table,
    RoleId,
    PermissionId,
}
