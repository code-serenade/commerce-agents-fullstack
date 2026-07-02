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
mod m20260509_095358_create_workspace_artifacts;
mod m20260509_105058_create_agent_quality_learning;
mod m20260512_090000_create_agent_skill_runs;
mod m20260522_100000_update_conversation_file_object_uniqueness;
mod m20260522_110000_create_storage_references;
mod m20260522_120000_backfill_storage_references;
mod m20260522_130000_create_storage_object_analyses;
mod m20260522_140000_storage_objects_sha256_identity;
mod m20260525_090000_add_conversation_message_strategy_enums;
mod m20260608_160000_add_app_user_registration_profile;
mod m20260619_090000_create_team_usage_quota;
mod m20260619_110000_create_account_usage_quota;
mod m20260624_090000_raise_question_quota_default;
mod m20260630_144220_create_product_access_analytics;

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
            Box::new(m20260509_095358_create_workspace_artifacts::Migration),
            Box::new(m20260509_105058_create_agent_quality_learning::Migration),
            Box::new(m20260512_090000_create_agent_skill_runs::Migration),
            Box::new(m20260522_100000_update_conversation_file_object_uniqueness::Migration),
            Box::new(m20260522_110000_create_storage_references::Migration),
            Box::new(m20260522_120000_backfill_storage_references::Migration),
            Box::new(m20260522_130000_create_storage_object_analyses::Migration),
            Box::new(m20260522_140000_storage_objects_sha256_identity::Migration),
            Box::new(m20260525_090000_add_conversation_message_strategy_enums::Migration),
            Box::new(m20260608_160000_add_app_user_registration_profile::Migration),
            Box::new(m20260619_090000_create_team_usage_quota::Migration),
            Box::new(m20260619_110000_create_account_usage_quota::Migration),
            Box::new(m20260624_090000_raise_question_quota_default::Migration),
            Box::new(m20260630_144220_create_product_access_analytics::Migration),
        ]
    }
}
