use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

use super::types::{SkillAdapterKind, SkillExecutionMode, SkillExecutionStatus, SkillRiskLevel};
use crate::skills::{SkillCategory, SkillSpec};

pub fn execution_mode_for_spec(spec: &SkillSpec) -> SkillExecutionMode {
    if spec.tags.iter().any(|tag| tag == "deferred")
        && custom_category(spec) != Some("search")
        && !spec.tags.iter().any(|tag| tag == "pending_approval")
    {
        SkillExecutionMode::Deferred
    } else if custom_category(spec) == Some("search")
        || spec.tags.iter().any(|tag| tag == "adapter")
        || spec.tags.iter().any(|tag| tag == "pending_approval")
        || spec.id.starts_with("platform_")
    {
        SkillExecutionMode::Adapter
    } else if adapter_kinds_for_spec(spec).is_empty() {
        SkillExecutionMode::Native
    } else {
        SkillExecutionMode::Adapter
    }
}

pub fn adapter_kinds_for_spec(spec: &SkillSpec) -> BTreeSet<SkillAdapterKind> {
    let mut kinds = BTreeSet::new();
    let category = custom_category(spec);
    if category == Some("search") {
        kinds.insert(SkillAdapterKind::Search);
        kinds.insert(SkillAdapterKind::Llm);
    }
    if spec.tags.iter().any(|tag| tag == "content_engine") {
        kinds.insert(SkillAdapterKind::Llm);
    }
    if spec
        .tags
        .iter()
        .any(|tag| tag == "design_engine" || tag == "analysis_engine")
    {
        kinds.insert(SkillAdapterKind::Llm);
    }
    if category == Some("data") && spec.tags.iter().any(|tag| tag == "deferred") {
        kinds.insert(SkillAdapterKind::Db);
    }
    if spec.tags.iter().any(|tag| tag == "pending_approval") || spec.id.starts_with("platform_") {
        kinds.insert(SkillAdapterKind::Platform);
    }
    kinds
}

pub fn risk_level_for_spec(spec: &SkillSpec) -> SkillRiskLevel {
    if spec.tags.iter().any(|tag| tag == "pending_approval") || spec.id.starts_with("platform_") {
        SkillRiskLevel::High
    } else if spec.tags.iter().any(|tag| tag == "deferred")
        || custom_category(spec) == Some("search")
    {
        SkillRiskLevel::Medium
    } else {
        SkillRiskLevel::Low
    }
}

pub(super) fn add_runtime_metadata(
    metadata: &mut Map<String, Value>,
    spec: &SkillSpec,
    mode: SkillExecutionMode,
) {
    metadata.insert("execution_mode".to_string(), json!(mode));
    metadata.insert("risk_level".to_string(), json!(risk_level_for_spec(spec)));
    metadata.insert(
        "approval_required".to_string(),
        json!(risk_level_for_spec(spec) == SkillRiskLevel::High),
    );
    let adapters = adapter_kinds_for_spec(spec).into_iter().collect::<Vec<_>>();
    if !adapters.is_empty() {
        metadata.insert("adapter_requirements".to_string(), json!(adapters));
    }
}

pub(super) fn status_from_output(output: &Value) -> Option<SkillExecutionStatus> {
    let status = output.get("status")?.as_str()?;
    match status {
        "deferred" => Some(SkillExecutionStatus::Deferred),
        "pending_approval" => Some(SkillExecutionStatus::PendingApproval),
        _ => None,
    }
}

fn custom_category(spec: &SkillSpec) -> Option<&str> {
    match &spec.category {
        SkillCategory::Custom(value) => Some(value.as_str()),
        _ => None,
    }
}
