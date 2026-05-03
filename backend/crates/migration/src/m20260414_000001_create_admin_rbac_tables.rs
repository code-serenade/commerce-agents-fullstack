use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AdminRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdminRoles::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AdminRoles::Name).string().not_null())
                    .col(ColumnDef::new(AdminRoles::Code).string().not_null())
                    .col(created_at_col())
                    .index(
                        Index::create()
                            .name("uk_admin_roles_code")
                            .col(AdminRoles::Code)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AdminPermissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdminPermissions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AdminPermissions::Code).string().not_null())
                    .col(ColumnDef::new(AdminPermissions::Name).string().not_null())
                    .col(ColumnDef::new(AdminPermissions::ParentCode).string().null())
                    .col(
                        ColumnDef::new(AdminPermissions::Sort)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(AdminPermissions::Kind).string().not_null())
                    .col(created_at_col())
                    .col(updated_at_col())
                    .index(
                        Index::create()
                            .name("uk_admin_permissions_code")
                            .col(AdminPermissions::Code)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_admin_permissions_parent_code")
                            .from(AdminPermissions::Table, AdminPermissions::ParentCode)
                            .to(AdminPermissions::Table, AdminPermissions::Code)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AdminUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdminUsers::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AdminUsers::DisplayId)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AdminUsers::DisplayName)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AdminUsers::Remark).string_len(255).null())
                    .col(ColumnDef::new(AdminUsers::Status).string().not_null())
                    .col(created_at_col())
                    .col(updated_at_col())
                    .index(
                        Index::create()
                            .name("uk_admin_users_display_id")
                            .col(AdminUsers::DisplayId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AdminUserRoles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AdminUserRoles::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(AdminUserRoles::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(created_at_col())
                    .primary_key(
                        Index::create()
                            .name("pk_admin_user_roles")
                            .col(AdminUserRoles::UserId)
                            .col(AdminUserRoles::RoleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_admin_user_roles_user_id")
                            .from(AdminUserRoles::Table, AdminUserRoles::UserId)
                            .to(AdminUsers::Table, AdminUsers::UserId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_admin_user_roles_role_id")
                            .from(AdminUserRoles::Table, AdminUserRoles::RoleId)
                            .to(AdminRoles::Table, AdminRoles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AdminRolePermissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdminRolePermissions::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AdminRolePermissions::PermissionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(created_at_col())
                    .primary_key(
                        Index::create()
                            .name("pk_admin_role_permissions")
                            .col(AdminRolePermissions::RoleId)
                            .col(AdminRolePermissions::PermissionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_admin_role_permissions_role_id")
                            .from(AdminRolePermissions::Table, AdminRolePermissions::RoleId)
                            .to(AdminRoles::Table, AdminRoles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_admin_role_permissions_permission_id")
                            .from(
                                AdminRolePermissions::Table,
                                AdminRolePermissions::PermissionId,
                            )
                            .to(AdminPermissions::Table, AdminPermissions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AdminMenus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdminMenus::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AdminMenus::Name).string().not_null())
                    .col(ColumnDef::new(AdminMenus::ParentId).big_integer().null())
                    .col(ColumnDef::new(AdminMenus::PermissionCode).string().null())
                    .col(created_at_col())
                    .col(updated_at_col())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_admin_menus_parent_id")
                            .from(AdminMenus::Table, AdminMenus::ParentId)
                            .to(AdminMenus::Table, AdminMenus::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_admin_menus_permission_code")
                            .from(AdminMenus::Table, AdminMenus::PermissionCode)
                            .to(AdminPermissions::Table, AdminPermissions::Code)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        create_updated_at_triggers(
            manager,
            &["admin_permissions", "admin_users", "admin_menus"],
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AdminMenus::Table).cascade().to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AdminRolePermissions::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AdminUserRoles::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AdminUsers::Table).cascade().to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AdminPermissions::Table)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AdminRoles::Table).cascade().to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS set_updated_at();")
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
enum AdminRoles {
    Table,
    Id,
    Name,
    Code,
}

#[derive(DeriveIden)]
enum AdminPermissions {
    Table,
    Id,
    Code,
    Name,
    ParentCode,
    Sort,
    Kind,
}

#[derive(DeriveIden)]
enum AdminUsers {
    Table,
    UserId,
    DisplayId,
    DisplayName,
    Remark,
    Status,
}

#[derive(DeriveIden)]
enum AdminUserRoles {
    Table,
    UserId,
    RoleId,
}

#[derive(DeriveIden)]
enum AdminRolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(DeriveIden)]
enum AdminMenus {
    Table,
    Id,
    Name,
    ParentId,
    PermissionCode,
}
