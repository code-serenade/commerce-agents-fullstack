use crate::skills::SkillCategory;

fn category() -> SkillCategory {
    SkillCategory::Custom("engineering".to_owned())
}

mod helpers;
mod native_macro;
mod normalize;
mod reports;
mod sla_monitor;
mod specs;
mod system_check;

pub use native_macro::{
    EngineeringArchReview, EngineeringBugAnalysis, EngineeringPerfOptimize,
    EngineeringTechSelection,
};
pub use sla_monitor::EngineeringSlaMonitor;
pub use specs::specs;
pub use system_check::EngineeringSystemCheck;
