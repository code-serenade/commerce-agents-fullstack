use crate::core::protocol::MessageId;

pub(super) fn push_unique(items: &mut Vec<String>, value: String) {
    if value.trim().is_empty() || items.iter().any(|item| item == &value) {
        return;
    }
    items.push(value);
}

pub(super) fn new_task_id() -> String {
    next_id("task")
}

pub(super) fn next_message_id() -> MessageId {
    MessageId(next_id("msg"))
}

fn next_id(prefix: &str) -> String {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let sequence = NEXT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{millis}-{sequence}")
}

pub(super) fn compact_event_text(value: String) -> String {
    const MAX_LEN: usize = 240;
    if value.chars().count() <= MAX_LEN {
        return value;
    }
    let mut text = value.chars().take(MAX_LEN).collect::<String>();
    text.push_str("...");
    text
}

pub(super) fn metadata_string(
    metadata: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    metadata
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

pub(super) fn skill_status_name(status: crate::skills::SkillExecutionStatus) -> &'static str {
    match status {
        crate::skills::SkillExecutionStatus::Success => "success",
        crate::skills::SkillExecutionStatus::Deferred => "deferred",
        crate::skills::SkillExecutionStatus::PendingApproval => "pending_approval",
        crate::skills::SkillExecutionStatus::NotFound => "not_found",
        crate::skills::SkillExecutionStatus::ValidationError => "validation_error",
        crate::skills::SkillExecutionStatus::Failed => "failed",
    }
}
