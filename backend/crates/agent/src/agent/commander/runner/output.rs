use utils::llm_json;

use crate::{
    agent::commander::{
        no_user_facing_content_message, sanitize_user_facing_text, user_facing_report_text,
    },
    core::protocol::{MessageId, TaskId},
    roles::{RoleAssignment, RoleContribution},
};

pub(super) fn final_worker_message(_task_id: &TaskId, output: Option<&str>) -> String {
    output
        .filter(|value| !value.trim().is_empty())
        .map(extract_worker_content)
        .and_then(|content| sanitize_user_facing_text(&content))
        .unwrap_or_else(no_user_facing_content_message)
}

pub(super) fn direct_chat_report_output(report: &crate::core::protocol::WorkerReport) -> String {
    user_facing_report_text(report).unwrap_or_else(no_user_facing_content_message)
}

pub(super) fn build_contribution(
    assignment: &RoleAssignment,
    task_id: &TaskId,
    output: &str,
) -> RoleContribution {
    let content = sanitize_user_facing_text(&extract_worker_content(output)).unwrap_or_default();
    RoleContribution::new(assignment, task_id.clone(), content)
}

fn extract_worker_content(output: &str) -> String {
    let trimmed = output.trim();
    let Ok(value) = llm_json::parse_llm_json_value(trimmed) else {
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

#[cfg(test)]
mod tests {
    use super::{build_contribution, final_worker_message};
    use crate::{
        core::protocol::{TaskId, WorkerId},
        roles::{RoleAssignment, RoleId},
    };

    #[test]
    fn final_worker_message_sanitizes_fast_route_output() {
        let output = final_worker_message(
            &TaskId("task-fast".to_string()),
            Some(
                "内容创意(creative): \
                 【可直接测试的讲解（占位符需替换为真实信息）】突出【真实品类，如挂耳/速溶/\
                 胶囊】。\n备注：教学讲解。",
            ),
        );

        assert!(output.contains("突出真实品类。"));
        assert!(output.contains("说明：教学讲解。"));
        assert!(!output.contains("内容创意(creative)"));
        assert!(!output.contains("占位符"));
        assert!(!output.contains("备注："));
    }

    #[test]
    fn final_worker_message_uses_readable_empty_fallback() {
        let output = final_worker_message(&TaskId("task-empty".to_string()), None);

        assert!(output.contains("没有拿到可直接展示的完整内容"));
        assert!(!output.contains("task-empty"));
        assert!(!output.contains("任务已完成"));
    }

    #[test]
    fn build_contribution_sanitizes_collaboration_content() {
        let assignment = RoleAssignment {
            role_id: RoleId("role.creative".to_string()),
            runtime_role: "creative".to_string(),
            worker_id: WorkerId("worker.creative".to_string()),
        };

        let contribution = build_contribution(
            &assignment,
            &TaskId("task-creative".to_string()),
            "creative(worker.creative): \
             【客服话术（占位符需替换为真实信息）】先确认用户诉求。\n备注：只用于教学。",
        );

        assert_eq!(contribution.content, "先确认用户诉求。\n说明：只用于教学。");
    }
}
