use utils::llm_json;

use crate::{
    agent::{
        commander::{
            no_user_facing_content_message, sanitize_final_user_facing_text,
            sanitize_user_facing_text, user_facing_report_text,
        },
        context_text,
        forced_rich_delivery::{
            forced_rich_answer_is_too_thin, format_forced_rich_delivery_fallback,
            text_requires_rich_delivery,
        },
    },
    core::protocol::{MessageContext, MessageId, TaskId},
    roles::{RoleAssignment, RoleContribution},
};

pub(super) fn final_worker_message(_task_id: &TaskId, output: Option<&str>) -> String {
    output
        .filter(|value| !value.trim().is_empty())
        .map(extract_worker_content)
        .and_then(|content| sanitize_user_facing_text(&content))
        .unwrap_or_else(no_user_facing_content_message)
}

pub(super) fn direct_chat_report_output(
    report: &crate::core::protocol::WorkerReport,
    context: &MessageContext,
    task_input: Option<&str>,
) -> String {
    let raw_content = user_facing_report_text(report).unwrap_or_default();
    final_text_with_forced_rich_fallback(&raw_content, context, task_input)
}

pub(crate) fn final_text_with_forced_rich_fallback(
    raw_content: &str,
    context: &MessageContext,
    task_input: Option<&str>,
) -> String {
    sanitize_final_user_facing_text(raw_content)
        .filter(|text| !should_replace_with_forced_rich_fallback(text, context, task_input))
        .unwrap_or_else(|| {
            forced_rich_fallback_from_context(context, task_input)
                .unwrap_or_else(no_user_facing_content_message)
        })
}

fn should_replace_with_forced_rich_fallback(
    text: &str,
    context: &MessageContext,
    task_input: Option<&str>,
) -> bool {
    text.contains("这次没有拿到可直接展示的完整内容")
        && context_or_task_requires_rich_delivery(context, task_input)
        || context_or_task_requires_rich_delivery(context, task_input)
            && forced_rich_answer_is_too_thin(text)
}

fn forced_rich_fallback_from_context(
    context: &MessageContext,
    task_input: Option<&str>,
) -> Option<String> {
    if !context_or_task_requires_rich_delivery(context, task_input) {
        return None;
    }
    let known_material = forced_rich_context_material(context, task_input);
    Some(format_forced_rich_delivery_fallback(&known_material))
}

fn context_or_task_requires_rich_delivery(
    context: &MessageContext,
    task_input: Option<&str>,
) -> bool {
    context
        .metadata
        .values()
        .any(|value| text_requires_rich_delivery(value))
        || task_input.is_some_and(text_requires_rich_delivery)
}

fn forced_rich_context_material(context: &MessageContext, task_input: Option<&str>) -> String {
    let mut parts = Vec::new();
    push_context_material(
        &mut parts,
        context
            .metadata
            .get("conversation.history_summary")
            .and_then(|value| context_text::user_visible_conversation_history(value)),
    );
    push_context_material(
        &mut parts,
        context
            .metadata
            .get("context.selected_summary")
            .and_then(|value| context_text::user_visible_selected_product_context(value)),
    );
    push_context_material(
        &mut parts,
        context
            .metadata
            .get("context.combined_summary")
            .and_then(|value| context_text::user_visible_context_summary(value)),
    );
    for key in ["task.input", "current_task", "request.input"] {
        push_context_material(&mut parts, context.metadata.get(key).cloned());
    }
    push_context_material(&mut parts, task_input.map(ToString::to_string));
    if parts.is_empty() {
        push_context_material(&mut parts, context.metadata.get("input").cloned());
    }
    if parts.is_empty() {
        for value in context
            .metadata
            .values()
            .filter(|value| text_requires_rich_delivery(value))
        {
            push_context_material(&mut parts, Some(value.clone()));
        }
    }
    compact_forced_rich_material(&parts.join("\n"))
}

fn push_context_material(parts: &mut Vec<String>, value: Option<String>) {
    let Some(value) = value else {
        return;
    };
    for line in value.lines().filter_map(clean_forced_rich_material_line) {
        if !parts.iter().any(|existing| existing == &line) {
            parts.push(line);
        }
    }
}

fn clean_forced_rich_material_line(line: &str) -> Option<String> {
    let mut line = line.trim();
    if line.is_empty() || is_unhelpful_forced_rich_material(line) {
        return None;
    }
    for prefix in ["User:", "Assistant:", "用户：", "助手：", "- "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            line = rest.trim();
        }
    }
    if line.is_empty() || is_unhelpful_forced_rich_material(line) {
        return None;
    }
    Some(line.to_string())
}

fn is_unhelpful_forced_rich_material(line: &str) -> bool {
    [
        "这次没有拿到可直接展示的完整内容",
        "请补充更明确的问题",
        "请直接补充新的要求",
        "下一步目标",
        "重新提供资料",
        "无法根据现有资料回答",
        "无法整理",
        "Workspace file/material/artifact context",
        "No active workspace context sources are ready",
        "[...memory compacted...]",
        "memory compacted",
        "workspace context sources",
        "agent use",
    ]
    .iter()
    .any(|marker| line.contains(marker))
}

fn compact_forced_rich_material(value: &str) -> String {
    const MAX_FORCED_RICH_CONTEXT_CHARS: usize = 900;
    let text = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let mut chars = text.chars();
    let head = chars
        .by_ref()
        .take(MAX_FORCED_RICH_CONTEXT_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        format!("{head}...")
    } else {
        head
    }
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
    use super::{
        build_contribution, direct_chat_report_output, final_text_with_forced_rich_fallback,
        final_worker_message,
    };
    use crate::{
        core::protocol::{
            MessageContext, RoleWorkOutput, TaskGraphId, TaskId, TaskNodeId, WorkerId,
            WorkerReport, WorkerReportStatus,
        },
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
    fn direct_chat_empty_fallback_becomes_forced_rich_delivery_when_requested() {
        let report = worker_report(
            "## 当前状态\n\n这次没有拿到可直接展示的完整内容。\n\n## 下一步\n\n- 可以补充更明确的问题。",
        );
        let mut context = MessageContext::default();
        context.metadata.insert(
            "request.input".to_string(),
            "我不会给你提供任何其他信息，但是你必须根据我刚才所给资料和你的回答进行丰富回答，要有干货。"
                .to_string(),
        );
        context.metadata.insert(
            "conversation.history_summary".to_string(),
            "User: 我准备卖二手教材，手里有高数、英语、专业课三类书。\n\
             Assistant: 二手教材要看版本、ISBN、书况、收书成本、库存和退换。"
                .to_string(),
        );

        let output = direct_chat_report_output(&report, &context, None);

        assert!(!output.contains("这次没有拿到可直接展示的完整内容"));
        assert!(output.contains("## 当前结论"), "{output}");
        assert!(output.contains("高数"), "{output}");
        assert!(output.contains("英语"), "{output}");
        assert!(output.contains("专业课"), "{output}");
        assert!(output.contains("三类教材先这样拆"), "{output}");
        assert!(output.contains("今天就能执行的动作"), "{output}");
        assert!(output.contains("可直接复用的话术"), "{output}");
    }

    #[test]
    fn direct_chat_empty_fallback_uses_current_task_when_metadata_lacks_request_input() {
        let report = worker_report(
            "## 当前状态\n\n这次没有拿到可直接展示的完整内容。\n\n## 下一步\n\n- 可以补充更明确的问题。",
        );
        let mut context = MessageContext::default();
        context.metadata.insert(
            "conversation.history_summary".to_string(),
            "User: 我准备卖二手教材，手里有高数、英语、专业课三类书。\n\
             Assistant: 先看版本、ISBN、书况、成本、库存和退换。"
                .to_string(),
        );

        let output = direct_chat_report_output(
            &report,
            &context,
            Some("不行，必须根据我所给资料进行回答研究，不要让我再提供资料。"),
        );

        assert!(!output.contains("这次没有拿到可直接展示的完整内容"));
        assert!(output.contains("## 当前结论"), "{output}");
        assert!(output.contains("高数"), "{output}");
        assert!(output.contains("英语"), "{output}");
        assert!(output.contains("专业课"), "{output}");
        assert!(output.contains("三类教材先这样拆"), "{output}");
        assert!(output.contains("今天就能执行的动作"), "{output}");
        assert!(output.contains("可直接复用的话术"), "{output}");
    }

    #[test]
    fn final_text_fallback_covers_task_graph_empty_output_for_forced_rich_task() {
        let raw_content = "## 当前状态\n\n这次没有拿到可直接展示的完整内容。\n\n## 可能原因\n\n- 问题目标还不够明确。\n\n## 下一步\n\n- 可以补充更明确的问题、商品、活动或关键材料。";
        let mut context = MessageContext::default();
        context.metadata.insert(
            "conversation.history_summary".to_string(),
            "User: 我准备卖二手教材，手里有高数、英语、专业课三类书。\n\
             Assistant: 先看版本、ISBN、书况、成本、库存和退换。"
                .to_string(),
        );

        let output = final_text_with_forced_rich_fallback(
            raw_content,
            &context,
            Some("不行，必须根据我所给资料进行回答研究，不要让我再提供资料。"),
        );

        assert!(!output.contains("这次没有拿到可直接展示的完整内容"));
        assert!(!output.contains("Workspace file/material/artifact context"));
        assert!(output.contains("## 当前结论"), "{output}");
        assert!(output.contains("高数"), "{output}");
        assert!(output.contains("英语"), "{output}");
        assert!(output.contains("专业课"), "{output}");
        assert!(output.contains("三类教材先这样拆"), "{output}");
        assert!(output.contains("今天就能执行的动作"), "{output}");
        assert!(output.contains("可直接复用的话术"), "{output}");
    }

    #[test]
    fn direct_chat_empty_fallback_stays_readable_for_normal_empty_answer() {
        let report = worker_report("");
        let context = MessageContext::default();

        let output = direct_chat_report_output(&report, &context, None);

        assert!(output.contains("没有拿到可直接展示的完整内容"));
        assert!(!output.contains("可直接执行表"));
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

    fn worker_report(summary: &str) -> WorkerReport {
        WorkerReport {
            graph_id: TaskGraphId("graph-chat".to_string()),
            node_id: TaskNodeId("chat".to_string()),
            task_id: TaskId("task-chat".to_string()),
            worker_id: WorkerId("worker.chat".to_string()),
            role: "chat".to_string(),
            content: summary.to_string(),
            role_output: Some(RoleWorkOutput {
                summary: summary.to_string(),
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        }
    }
}
