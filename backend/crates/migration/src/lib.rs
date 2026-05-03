pub use sea_orm_migration::prelude::*;

mod m20260414_000001_create_admin_rbac_tables;
mod m20260414_000002_create_app_rbac_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260414_000001_create_admin_rbac_tables::Migration),
            Box::new(m20260414_000002_create_app_rbac_tables::Migration),
        ]
    }
}
