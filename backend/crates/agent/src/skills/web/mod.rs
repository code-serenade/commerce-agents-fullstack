use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillResult, SkillSpec, object_params, optional_f64_param, optional_i64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("web".to_owned())
}

include!("specs.rs");
include!("title_scorer.rs");
include!("native_macro.rs");
include!("conversion.rs");
include!("normalize.rs");
include!("outcomes.rs");
include!("helpers.rs");
