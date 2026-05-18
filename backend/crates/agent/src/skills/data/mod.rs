use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillResult, SkillSpec,
};

fn category() -> SkillCategory {
    SkillCategory::Data
}

include!("specs.rs");
include!("executor.rs");
include!("common_spec.rs");
include!("queries.rs");
include!("output_contract.rs");
include!("funnel_forecast.rs");
include!("dashboard.rs");
include!("market.rs");
include!("attribution.rs");
include!("forecast.rs");
include!("comprehensive.rs");
include!("common.rs");
