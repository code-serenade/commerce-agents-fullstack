mod attribution;
mod common;
mod common_spec;
mod comprehensive;
mod dashboard;
mod executor;
mod forecast;
mod funnel_forecast;
mod market;
mod output_contract;
mod queries;
mod specs;

use crate::skills::SkillCategory;

fn category() -> SkillCategory {
    SkillCategory::Data
}

pub use executor::{DataSkill, execute};
pub use specs::specs;
