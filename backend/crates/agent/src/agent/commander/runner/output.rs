use crate::{
    core::protocol::{MessageId, TaskId},
    roles::{RoleAssignment, RoleContribution},
};

pub(super) fn final_worker_message(task_id: &TaskId, output: Option<&str>) -> String {
    output
        .filter(|value| !value.trim().is_empty())
        .map(extract_worker_content)
        .unwrap_or_else(|| format!("任务已完成: {}", task_id.0))
}

pub(super) fn direct_chat_report_output(report: &crate::core::protocol::WorkerReport) -> String {
    report
        .role_output
        .as_ref()
        .map(|output| output.summary.trim())
        .filter(|summary| !summary.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            let content = report.content.trim();
            (!content.is_empty()).then(|| content.to_string())
        })
        .unwrap_or_else(|| "我在，可以直接告诉我你想聊什么或需要我帮你做什么。".to_string())
}

pub(super) fn build_contribution(
    assignment: &RoleAssignment,
    task_id: &TaskId,
    output: &str,
) -> RoleContribution {
    RoleContribution::new(assignment, task_id.clone(), extract_worker_content(output))
}

fn extract_worker_content(output: &str) -> String {
    let trimmed = output.trim();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return trimmed.to_string();
    };

    value
        .pointer("/decision/action/parameters/answer")
        .and_then(|value| value.as_str())
        .or_else(|| {
            value
                .pointer("/role_output/summary")
                .and_then(|value| value.as_str())
        })
        .or_else(|| {
            value
                .pointer("/role_output")
                .and_then(|value| value.as_str())
        })
        .or_else(|| {
            value
                .pointer("/rationale/primary")
                .and_then(|value| value.as_str())
        })
        .map(ToString::to_string)
        .unwrap_or_else(|| trimmed.to_string())
}

pub(super) fn next_message_id() -> MessageId {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_MESSAGE_COUNTER: AtomicU64 = AtomicU64::new(1);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let sequence = NEXT_MESSAGE_COUNTER.fetch_add(1, Ordering::Relaxed);

    MessageId(format!("msg-{millis}-{sequence}"))
}
