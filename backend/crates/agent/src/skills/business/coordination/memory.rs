use async_trait::async_trait;
use serde_json::{Map, Value, json};

use super::{
    common::{compact_task_text_value, compact_text_for_task_body, metadata_string_value},
    specs::coordination_save_memory_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params,
    optional_string_param, string_param,
};

pub struct CoordinationSaveMemory {
    spec: SkillSpec,
}

impl CoordinationSaveMemory {
    pub fn new() -> Self {
        Self {
            spec: coordination_save_memory_spec(),
        }
    }
}

impl Default for CoordinationSaveMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationSaveMemory {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let key = normalize_memory_key(&string_param(&params, "key")?);
        let content = compact_text_for_task_body(&string_param(&params, "content")?);
        let memory_type = normalize_memory_type(
            optional_string_param(&params, "memory_type")?.unwrap_or_else(|| {
                infer_memory_type_from_text(&content, &context.metadata).to_string()
            }),
        );
        let workspace_id = metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id"))
            .or_else(|| metadata_string_value(&context.metadata, "request.workspace.workspace_id"));
        let role = metadata_string_value(&context.metadata, "role_runtime")
            .or_else(|| metadata_string_value(&context.metadata, "assignment_role"))
            .unwrap_or_else(|| "agent".to_string());

        let normalized = json!({
            "key": key,
            "content": content,
            "memory_type": memory_type,
            "workspace_id": workspace_id,
            "role": role,
            "source": "session_skill",
        });

        Ok(SkillOutcome::new(json!({
            "status": "success",
            "memory_save": {
                "should_persist": true,
                "key": normalized.get("key").cloned().unwrap_or(Value::Null),
                "content": normalized.get("content").cloned().unwrap_or(Value::Null),
                "memory_type": normalized.get("memory_type").cloned().unwrap_or(Value::Null),
                "workspace_id": normalized.get("workspace_id").cloned().unwrap_or(Value::Null),
                "role": normalized.get("role").cloned().unwrap_or(Value::Null),
                "source": "session_skill"
            },
            "normalized_inputs": normalized,
            "no_side_effects": true
        }))
        .with_summary(format!("工作记忆草案已生成：{}", key))
        .with_metadata("memory_save_kind", json!("workspace_learning_record"))
        .with_metadata("memory_save_should_persist", json!(true)))
    }
}

fn normalize_memory_key(value: &str) -> String {
    let mut normalized = value
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else if ch.is_ascii() {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();
    while normalized.contains("__") {
        normalized = normalized.replace("__", "_");
    }
    normalized = normalized.trim_matches('_').to_string();
    if normalized.is_empty() {
        "memory".to_string()
    } else {
        compact_task_text_value(&normalized, 120)
    }
}

fn normalize_memory_type(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "fact" | "事实" => "fact".to_string(),
        "decision" | "决策" => "decision".to_string(),
        "insight" | "洞察" => "insight".to_string(),
        "warning" | "风险" => "warning".to_string(),
        _ => "fact".to_string(),
    }
}

fn infer_memory_type_from_text(value: &str, metadata: &Map<String, Value>) -> &'static str {
    let text = value.to_ascii_lowercase();
    if text.contains("风险")
        || text.contains("warning")
        || metadata_string_value(metadata, "request_reason")
            .as_deref()
            .is_some_and(|reason| reason.contains("风险"))
    {
        "warning"
    } else if text.contains("决策") || text.contains("决定") || text.contains("decision") {
        "decision"
    } else if text.contains("洞察") || text.contains("insight") || text.contains("发现") {
        "insight"
    } else {
        "fact"
    }
}
