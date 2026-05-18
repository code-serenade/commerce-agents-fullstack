use std::collections::BTreeMap;

use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillResult, SkillSpec, object_params, optional_f64_param, optional_i64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("service".to_owned())
}

fn string_schema() -> Value {
    json!({"type": "string"})
}

fn number_schema() -> Value {
    json!({"type": "number"})
}

fn integer_schema() -> Value {
    json!({"type": "integer"})
}

fn product_id_schema() -> Value {
    json!({"oneOf": [{"type": "integer"}, {"type": "string"}]})
}

fn string_array_schema() -> Value {
    json!({"type": "array", "items": {"type": "string"}})
}

fn object_schema() -> Value {
    json!({"type": "object"})
}

fn outcome(output: Value, summary: impl Into<String>) -> SkillResult {
    Ok(SkillOutcome::new(output).with_summary(summary))
}

include!("specs.rs");
include!("return_handler.rs");
include!("ticket.rs");
include!("faq.rs");
include!("escalation.rs");
include!("product_query.rs");
include!("playbook_helpers.rs");
include!("dsr.rs");
include!("nps.rs");
include!("sentiment.rs");
include!("nps_driver.rs");
include!("common.rs");
