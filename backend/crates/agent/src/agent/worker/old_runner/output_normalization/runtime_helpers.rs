use super::*;

pub(super) fn skill_executor_unavailable_result(request_id: String, skill_id: String) -> SkillExecutionResult {
    SkillExecutionResult {
        request_id,
        skill_id: SkillId::new(skill_id),
        status: SkillExecutionStatus::Failed,
        output: None,
        summary: None,
        metadata: serde_json::Map::new(),
        error: Some("skill executor unavailable".to_string()),
    }
}

pub(super) fn compact_event_text(value: String) -> String {
    let trimmed = value.trim();
    const MAX_LEN: usize = 240;
    if trimmed.chars().count() <= MAX_LEN {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(MAX_LEN).collect::<String>();
    output.push_str("...");
    output
}

pub(super) fn metadata_string(
    metadata: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    metadata.get(key).and_then(|value| {
        value
            .as_str()
            .map(ToString::to_string)
            .or_else(|| value.as_object().map(|_| value.to_string()))
    })
}

pub(super) fn skill_status_name(status: SkillExecutionStatus) -> &'static str {
    match status {
        SkillExecutionStatus::Success => "success",
        SkillExecutionStatus::Deferred => "deferred",
        SkillExecutionStatus::PendingApproval => "pending_approval",
        SkillExecutionStatus::NotFound => "not_found",
        SkillExecutionStatus::ValidationError => "validation_error",
        SkillExecutionStatus::Failed => "failed",
    }
}

