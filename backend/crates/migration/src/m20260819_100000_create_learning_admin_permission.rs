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
INSERT INTO admin_permissions (code, name, parent_code, sort, kind)
VALUES ('learning', 'Learning', NULL, 50, 'group')
ON CONFLICT (code) DO NOTHING;

INSERT INTO admin_permissions (code, name, parent_code, sort, kind)
VALUES ('learning:read', 'Learning Read', 'learning', 10, 'action')
ON CONFLICT (code) DO NOTHING;

INSERT INTO admin_menus (name, parent_id, permission_code)
SELECT 'Learning Evaluation', NULL, 'learning:read'
WHERE NOT EXISTS (SELECT 1 FROM admin_menus WHERE permission_code = 'learning:read');

INSERT INTO admin_role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM admin_roles r
CROSS JOIN admin_permissions p
WHERE r.code IN ('root', 'admin')
  AND p.code IN ('learning', 'learning:read')
ON CONFLICT DO NOTHING;
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
DELETE FROM admin_menus WHERE permission_code = 'learning:read';

DELETE FROM admin_role_permissions
WHERE permission_id IN (
    SELECT id FROM admin_permissions WHERE code IN ('learning', 'learning:read')
);

DELETE FROM admin_permissions
WHERE code IN ('learning', 'learning:read');
"#,
            )
            .await?;
        Ok(())
    }
}
