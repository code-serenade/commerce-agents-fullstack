use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillInputField, SkillOutcome, SkillPriority, SkillResult,
    SkillSpec, object_params, optional_bool_param, optional_f64_param, optional_i64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("engineering".to_owned())
}

include!("specs.rs");
include!("native_macro.rs");
include!("normalize.rs");
include!("reports.rs");
include!("system_check.rs");
include!("sla_monitor.rs");
