mod campaign;
mod common;
mod handoff;
mod memory;
mod orchestration;
mod platform;
mod skill_chain;
mod specs;
mod status;
mod task;

pub use campaign::CoordinationCreateCampaign;
pub use handoff::CoordinationAgentHandoff;
pub use memory::CoordinationSaveMemory;
pub use orchestration::CoordinationTaskOrchestration;
pub use platform::{
    CoordinationExternalAgentCall, PlatformSyncProducts, PlatformUpdateInventory,
    PlatformUpdatePrice,
};
pub use skill_chain::CoordinationSkillChainPlanner;
pub use specs::{
    coordination_agent_handoff_spec, coordination_create_campaign_spec,
    coordination_create_task_spec, coordination_external_agent_call_spec,
    coordination_save_memory_spec, coordination_skill_chain_planner_spec,
    coordination_status_query_spec, coordination_task_orchestration_spec,
    platform_sync_products_spec, platform_update_inventory_spec, platform_update_price_spec, specs,
};
pub use status::CoordinationStatusQuery;
pub use task::CoordinationCreateTask;
