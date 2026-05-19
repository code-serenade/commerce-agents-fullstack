use serde_json::{Value, json};

use super::category;
use crate::skills::{SkillError, SkillInputField, SkillPriority, SkillSpec};

pub(super) fn spec(
    id: &str,
    name: &str,
    description: &str,
    priority: SkillPriority,
    inputs: Vec<SkillInputField>,
    tags: &[&str],
) -> SkillSpec {
    let mut skill = SkillSpec::new(id, name, description)
        .with_category(category())
        .with_priority(priority);
    for input in inputs {
        skill = skill.with_input(input);
    }
    for tag in tags {
        skill = skill.with_tag(*tag);
    }
    skill
}

pub(super) fn field(name: &str, kind: &str, description: &str, required: bool) -> SkillInputField {
    let schema = match kind {
        "string" => json!({"type": "string"}),
        "integer" => json!({"type": "integer"}),
        "number" => json!({"type": "number"}),
        "boolean" => json!({"type": "boolean"}),
        "array<number>" => json!({"type": "array", "items": {"type": "number"}}),
        "array<integer>" => json!({"type": "array", "items": {"type": "integer"}}),
        "array<object>" => json!({"type": "array", "items": {"type": "object"}}),
        _ => json!({"type": "object"}),
    };
    if required {
        SkillInputField::required(name, description, schema)
    } else {
        SkillInputField::optional(name, description, schema)
    }
}

pub(super) fn ensure_object(input: &Value) -> Result<(), SkillError> {
    if input.is_object() {
        Ok(())
    } else {
        Err(SkillError::invalid_parameter("$", "object"))
    }
}

pub(super) fn deferred(skill: &str, reason: &str, input: &Value) -> Value {
    json!({
        "status": "deferred",
        "skill": skill,
        "deferred_reason": reason,
        "received_params": input,
        "side_effects": "none",
        "expected_outputs": ["结构化指标", "诊断摘要", "可执行建议"]
    })
}
