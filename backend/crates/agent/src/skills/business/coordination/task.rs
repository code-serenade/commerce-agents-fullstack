use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    common::{compact_text_for_task_body, compact_text_for_task_title, metadata_string_value},
    specs::coordination_create_task_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillResult, SkillSpec, object_params, optional_string_param, string_param,
};

pub struct CoordinationCreateTask {
    spec: SkillSpec,
}

impl CoordinationCreateTask {
    pub fn new() -> Self {
        Self {
            spec: coordination_create_task_spec(),
        }
    }
}

impl Default for CoordinationCreateTask {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationCreateTask {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let title = compact_text_for_task_title(&string_param(&params, "title")?);
        let description = optional_string_param(&params, "description")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty())
            .or_else(|| metadata_string_value(&context.metadata, "task_description"));
        let owner_role = normalize_owner_role(
            optional_string_param(&params, "owner_role")?
                .or_else(|| metadata_string_value(&context.metadata, "assignment_role"))
                .or_else(|| metadata_string_value(&context.metadata, "role_runtime")),
        );
        let priority = coordination_task_priority(&params)?;
        let acceptance_criteria = optional_string_param(&params, "acceptance_criteria")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty());
        let workspace_id = metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id"))
            .or_else(|| metadata_string_value(&context.metadata, "request.workspace.workspace_id"));

        let normalized = json!({
            "title": title,
            "description": description,
            "owner_role": owner_role,
            "priority": priority,
            "acceptance_criteria": acceptance_criteria,
            "workspace_id": workspace_id,
            "source": "session_skill",
        });

        Ok(crate::skills::SkillOutcome::new(json!({
            "status": "success",
            "task_creation": {
                "should_persist": true,
                "title": normalized.get("title").cloned().unwrap_or(Value::Null),
                "description": normalized.get("description").cloned().unwrap_or(Value::Null),
                "owner_role": normalized.get("owner_role").cloned().unwrap_or(Value::Null),
                "priority": normalized.get("priority").cloned().unwrap_or(Value::Null),
                "acceptance_criteria": normalized.get("acceptance_criteria").cloned().unwrap_or(Value::Null),
                "workspace_id": normalized.get("workspace_id").cloned().unwrap_or(Value::Null),
                "source": "session_skill"
            },
            "normalized_inputs": normalized,
            "no_side_effects": true
        }))
        .with_summary("工作任务草案已生成，可落到当前工作区")
        .with_metadata("task_creation_kind", json!("workspace_task"))
        .with_metadata("task_creation_should_persist", json!(true)))
    }
}

fn normalize_owner_role(value: Option<String>) -> String {
    let normalized = value
        .unwrap_or_else(|| "ops".to_string())
        .trim()
        .to_ascii_lowercase();
    match normalized.as_str() {
        "ops" | "data" | "service" | "design" | "accounting" | "engineering" | "web"
        | "creative" | "chat" => normalized,
        _ => "ops".to_string(),
    }
}

fn clamp_coordination_priority(value: i64) -> i64 {
    value.clamp(0, 10)
}

fn coordination_task_priority(params: &Value) -> Result<i64, crate::skills::SkillError> {
    let Some(value) = params.get("priority") else {
        return Ok(3);
    };
    match value {
        Value::Null => Ok(3),
        Value::Number(number) => number
            .as_i64()
            .map(clamp_coordination_priority)
            .ok_or_else(|| crate::skills::SkillError::invalid_parameter("priority", "integer")),
        Value::String(text) => Ok(normalize_coordination_priority_text(text).unwrap_or(3)),
        _ => Err(crate::skills::SkillError::invalid_parameter(
            "priority",
            "integer or priority label",
        )),
    }
}

fn normalize_coordination_priority_text(value: &str) -> Option<i64> {
    let text = value.trim().to_ascii_lowercase();
    if text.is_empty() {
        return None;
    }
    match text.as_str() {
        "0" => Some(0),
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        "6" => Some(6),
        "7" => Some(7),
        "8" => Some(8),
        "9" => Some(9),
        "10" => Some(10),
        "low" | "normal" | "普通" | "低" | "低优先级" => Some(3),
        "medium" | "important" | "中" | "中等" | "重要" | "较重要" => Some(6),
        "high" | "urgent" | "紧急" | "高" | "高优先级" | "critical" | "立即" | "最高" => {
            Some(8)
        }
        _ => text.parse::<i64>().ok().map(clamp_coordination_priority),
    }
}
