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

mod common;
mod dsr;
mod escalation;
mod faq;
mod nps;
mod nps_driver;
mod playbook_helpers;
mod product_query;
mod return_handler;
mod sentiment;
mod specs;
mod ticket;

use common::*;
pub use dsr::ServiceDsrImprovement;
use dsr::*;
pub use escalation::ServiceEscalationFlow;
pub use faq::ServiceFaqPlaybook;
pub use nps::{ServiceNpsAnalyzer, service_nps_analyzer_spec};
pub use nps_driver::{ServiceNpsDriverAnalysis, service_nps_driver_analysis_spec};
use playbook_helpers::*;
pub use product_query::ServiceQueryProduct;
use product_query::*;
pub use return_handler::ServiceReturnHandler;
pub use sentiment::{ServiceSentimentAnalyzer, service_sentiment_analyzer_spec};
pub use specs::{
    service_dsr_improvement_spec, service_escalation_flow_spec, service_faq_playbook_spec,
    service_query_product_spec, service_return_handler_spec, service_ticket_handler_spec, specs,
};
pub use ticket::ServiceTicketHandler;
