pub use sea_orm_migration::prelude::*;

mod m20260414_000001_create_admin_rbac_tables;
mod m20260414_000002_create_app_rbac_tables;
mod m20260506_000001_create_conversation_tables;
mod m20260506_000002_create_agent_memories;
mod m20260507_092206_create_products;
mod m20260507_101500_create_product_knowledge;
mod m20260507_111500_create_product_materials_assets;
mod m20260507_123000_create_product_competitors;
mod m20260507_133000_create_campaigns;
mod m20260507_143000_create_workspace_execution;
mod m20260507_153000_create_platform_connect;
mod m20260507_163000_create_approval_audit;
mod m20260507_173000_create_long_tasks_briefing_alerts;
mod m20260507_183000_create_package_governance;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260414_000001_create_admin_rbac_tables::Migration),
            Box::new(m20260414_000002_create_app_rbac_tables::Migration),
            Box::new(m20260506_000001_create_conversation_tables::Migration),
            Box::new(m20260506_000002_create_agent_memories::Migration),
            Box::new(m20260507_092206_create_products::Migration),
            Box::new(m20260507_101500_create_product_knowledge::Migration),
            Box::new(m20260507_111500_create_product_materials_assets::Migration),
            Box::new(m20260507_123000_create_product_competitors::Migration),
            Box::new(m20260507_133000_create_campaigns::Migration),
            Box::new(m20260507_143000_create_workspace_execution::Migration),
            Box::new(m20260507_153000_create_platform_connect::Migration),
            Box::new(m20260507_163000_create_approval_audit::Migration),
            Box::new(m20260507_173000_create_long_tasks_briefing_alerts::Migration),
            Box::new(m20260507_183000_create_package_governance::Migration),
        ]
    }
}
