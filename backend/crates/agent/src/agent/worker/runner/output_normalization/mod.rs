mod benchmark;
mod category_boundary;
mod evidence;
mod runtime_noise;
mod text_syntax;
mod util;

use benchmark::sanitize_unsupported_benchmark_claims;
pub(super) use category_boundary::is_category_level_task;
use category_boundary::normalize_category_level_product_requests;
use evidence::{merge_unique, normalize_product_context_evidence, normalize_upstream_evidence};
use runtime_noise::{is_category_runtime_noise, sanitize_category_level_runtime_artifacts};
use text_syntax::normalize_user_facing_text;
use util::compact_task_text;

use super::{
    output::{format_role_work_output_content, merge_skill_results},
    policy::worker_policies,
    types::ParsedWorkerRoleOutput,
    util::push_unique,
};
use crate::{
    agent::{
        context_text,
        forced_rich_delivery::{
            forced_rich_answer_is_too_thin, format_forced_rich_delivery_fallback,
            text_requires_rich_delivery,
        },
        worker::Task,
    },
    core::protocol::RoleWorkOutput,
    skills::SkillExecutionResult,
};

const MAX_FORCED_RICH_KNOWN_MATERIAL_CHARS: usize = 900;

pub(super) fn finalize_worker_role_output(
    role: &str,
    task: &Task,
    parsed: ParsedWorkerRoleOutput,
    skill_results: &[SkillExecutionResult],
) -> ParsedWorkerRoleOutput {
    let parsed = merge_skill_results(parsed, skill_results);
    normalize_worker_role_output(role, task, parsed)
}

fn normalize_worker_role_output(
    role: &str,
    task: &Task,
    mut parsed: ParsedWorkerRoleOutput,
) -> ParsedWorkerRoleOutput {
    let output = parsed
        .role_output
        .get_or_insert_with(RoleWorkOutput::default);
    let output_policy = &worker_policies().output;
    let normalization_policy = &worker_policies().output_normalization;
    let task_line = compact_task_text(&task.description);

    if output.summary.trim().is_empty() {
        output.summary =
            normalization_policy.default_summary(output_policy.role_display_name(role), &task_line);
    }
    if role != "chat" && is_material_gap_dominant_summary(&output.summary) {
        output.summary =
            normalization_policy.default_summary(output_policy.role_display_name(role), &task_line);
    }

    if role == "chat" {
        if let Some(summary) = forced_rich_chat_fallback(task, &output.summary) {
            output.summary = summary;
        }
        output.findings.clear();
        output.recommendations.clear();
        output.evidence.clear();
        output.risks.clear();
        output.open_questions.clear();
        normalize_user_facing_text(output);
        parsed.evidence.clear();
        parsed.risks.clear();
        parsed.open_questions.clear();
        parsed.content = output.summary.trim().to_string();
        return parsed;
    }

    let deliverable_only = is_direct_deliverable_only_task(&task.description, &output.summary);

    if output.findings.is_empty() && !deliverable_only {
        output.findings = output_policy.default_role_findings(role, &task_line);
    }
    while !deliverable_only
        && output.findings.len() < output_policy.required_findings_for_role(role)
    {
        output.findings.push(output_policy.default_role_finding(
            role,
            output.findings.len(),
            &task_line,
        ));
    }
    if !deliverable_only {
        output_policy.normalize_role_specific_findings(role, &mut output.findings, &task_line);
    }

    if output.recommendations.is_empty() && !deliverable_only {
        output.recommendations = output_policy.default_role_recommendations(role, &task_line);
    }
    while !deliverable_only
        && output.recommendations.len() < output_policy.required_recommendations_for_role(role)
    {
        output
            .recommendations
            .push(output_policy.default_role_recommendation(
                role,
                output.recommendations.len(),
                &task_line,
            ));
    }
    if !deliverable_only {
        output_policy.normalize_role_specific_recommendations(
            role,
            &mut output.recommendations,
            &task_line,
        );
    }

    normalize_product_context_evidence(task, output);
    normalize_upstream_evidence(task, output);
    let category_level_boundary_applied =
        normalize_category_level_product_requests(role, task, output);
    if category_level_boundary_applied {
        sanitize_category_level_runtime_artifacts(output);
        parsed
            .open_questions
            .retain(|item| !is_category_runtime_noise(item));
        parsed.risks.retain(|item| !is_category_runtime_noise(item));
        parsed
            .evidence
            .retain(|item| !is_category_runtime_noise(item));
    }

    if sanitize_unsupported_benchmark_claims(output) {
        push_unique(
            &mut output.risks,
            normalization_policy
                .unsupported_benchmark()
                .risk()
                .to_string(),
        );
        while output.findings.len() < output_policy.required_findings_for_role(role) {
            output.findings.push(output_policy.default_role_finding(
                role,
                output.findings.len(),
                &task_line,
            ));
        }
        while output.recommendations.len() < output_policy.required_recommendations_for_role(role) {
            output
                .recommendations
                .push(output_policy.default_role_recommendation(
                    role,
                    output.recommendations.len(),
                    &task_line,
                ));
        }
    }

    if output.evidence.is_empty() {
        output
            .evidence
            .push(normalization_policy.default_evidence().to_string());
    }
    if output.risks.is_empty() && output_policy.role_requires_risk(role) {
        output.risks = output_policy.default_role_risks(role);
    }
    output_policy.normalize_role_specific_risks(role, &mut output.risks, &task_line);
    normalize_user_facing_text(output);

    parsed.evidence = merge_unique(parsed.evidence, output.evidence.clone());
    parsed.risks = merge_unique(parsed.risks, output.risks.clone());
    parsed.open_questions = merge_unique(parsed.open_questions, output.open_questions.clone());
    parsed.content = format_role_work_output_content(output);
    parsed
}

fn forced_rich_chat_fallback(task: &Task, summary: &str) -> Option<String> {
    if !task_requires_forced_rich_delivery(task) || !is_thin_forced_rich_summary(summary) {
        return None;
    }

    let known_material = forced_rich_known_material(task, summary);
    Some(format_forced_rich_delivery_fallback(&known_material))
}

fn task_requires_forced_rich_delivery(task: &Task) -> bool {
    let mut texts = vec![task.description.as_str()];
    if let Some(assignment) = &task.assignment {
        texts.push(assignment.root_task.as_str());
        texts.push(assignment.objective.as_str());
    }

    texts.iter().any(|text| text_requires_rich_delivery(text))
}

fn is_thin_forced_rich_summary(summary: &str) -> bool {
    forced_rich_answer_is_too_thin(summary)
}

fn forced_rich_known_material(task: &Task, summary: &str) -> String {
    let mut parts = Vec::new();

    push_context_material(
        &mut parts,
        task.context
            .metadata
            .get("conversation.history_summary")
            .and_then(|value| context_text::user_visible_conversation_history(value)),
    );
    push_context_material(
        &mut parts,
        task.context
            .metadata
            .get("context.selected_summary")
            .and_then(|value| context_text::user_visible_selected_product_context(value)),
    );
    push_context_material(
        &mut parts,
        task.context
            .metadata
            .get("context.combined_summary")
            .and_then(|value| context_text::user_visible_context_summary(value)),
    );

    if let Some(assignment) = &task.assignment {
        push_context_material(&mut parts, Some(assignment.root_task.clone()));
        push_context_material(&mut parts, Some(task.description.clone()));
    } else {
        push_context_material(&mut parts, Some(task.description.clone()));
    }
    push_context_material(&mut parts, Some(summary.to_string()));

    compact_known_material(&parts.join("\n"))
}

fn push_context_material(parts: &mut Vec<String>, value: Option<String>) {
    let Some(value) = value else {
        return;
    };
    for line in value.lines().filter_map(clean_known_material_line) {
        if !parts.iter().any(|existing| existing == &line) {
            parts.push(line);
        }
    }
}

fn clean_known_material_line(line: &str) -> Option<String> {
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
        "问题目标还不够明确",
        "协作步骤没有产出可直接展示的结果",
        "可以补充更明确的问题",
        "也可以换成普通问答",
        "请直接补充新的要求",
        "下一步目标",
        "重新提供资料",
        "无法根据现有资料回答",
        "无法整理",
    ]
    .iter()
    .any(|marker| line.contains(marker))
}

fn compact_known_material(value: &str) -> String {
    let text = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let mut chars = text.chars();
    let head = chars
        .by_ref()
        .take(MAX_FORCED_RICH_KNOWN_MATERIAL_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        format!("{head}...")
    } else {
        head
    }
}

fn is_material_gap_dominant_summary(value: &str) -> bool {
    let text = value.trim();
    if text.is_empty() {
        return false;
    }
    let mentions_material_gap = [
        "未收到可剖析的表格",
        "未收到可解析的表格",
        "未收到可理解的资料",
        "未收到可诊断的图片",
        "未收到可提取证据",
        "未收到可对比",
        "尚未收到可读取正文",
        "未读取到资料正文",
        "只收到资料元数据",
        "缺少可读取正文",
        "缺少 rows 数组",
        "缺少表格内容",
        "未提供附件或表格",
        "未找到附件或表格数据",
        "未找到可用的附件或表格数据",
        "等待用户提供附件或表格数据",
        "建议用户提供附件或表格数据",
        "无法直接抽取",
        "无法进行进一步分析",
        "无法提供具体的表格",
        "无法生成确定性",
        "暂无法产出结果",
    ]
    .iter()
    .any(|marker| text.contains(marker));
    if !mentions_material_gap {
        return false;
    }
    let has_business_answer = [
        "建议",
        "先看",
        "下一步",
        "检查",
        "优化",
        "标题",
        "主图",
        "指标",
        "清单",
        "作业",
        "话术",
    ]
    .iter()
    .any(|marker| text.contains(marker));
    !has_business_answer || text.chars().count() < 120
}

fn is_direct_deliverable_only_task(task: &str, summary: &str) -> bool {
    let task = task.trim();
    if task.is_empty() || summary.trim().is_empty() {
        return false;
    }
    let has_direct_boundary = [
        "只输出",
        "只要",
        "不要整坨",
        "不要解释",
        "不要展开",
        "别解释",
        "直接给",
    ]
    .iter()
    .any(|marker| task.contains(marker));
    if !has_direct_boundary {
        return false;
    }
    ["标题", "文案", "话术", "脚本", "开头", "卖点", "清单"]
        .iter()
        .any(|marker| task.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::{RoleWorkOutput, normalize_worker_role_output};
    use crate::{
        agent::{
            forced_rich_delivery::MIN_FORCED_RICH_DELIVERY_CHARS,
            worker::{Task, runner::types::ParsedWorkerRoleOutput},
        },
        core::protocol::{AgentId, MessageContext},
    };

    #[test]
    fn chat_role_keeps_only_user_facing_summary() {
        let parsed = ParsedWorkerRoleOutput {
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "### 回答\n1. **先看真实资料**".to_string(),
                findings: vec!["不应展示的发现".to_string()],
                recommendations: vec!["不应展示的建议".to_string()],
                evidence: vec!["不应展示的依据".to_string()],
                risks: vec!["不应展示的风险".to_string()],
                open_questions: vec!["不应展示的待确认".to_string()],
            }),
            evidence: vec!["外层依据".to_string()],
            risks: vec!["外层风险".to_string()],
            open_questions: vec!["外层问题".to_string()],
            diagnostics: Vec::new(),
        };
        let task = Task {
            id: "task-1".to_string(),
            context: MessageContext::default(),
            description: "咖啡怎么卖？".to_string(),
            requester: AgentId("user-1".to_string()),
            assignment: None,
        };

        let output = normalize_worker_role_output("chat", &task, parsed);
        let role_output = output.role_output.expect("role output should exist");

        assert_eq!(output.content, "回答\n先看真实资料");
        assert_eq!(role_output.summary, "回答\n先看真实资料");
        assert!(role_output.findings.is_empty());
        assert!(role_output.recommendations.is_empty());
        assert!(role_output.evidence.is_empty());
        assert!(role_output.risks.is_empty());
        assert!(role_output.open_questions.is_empty());
        assert!(output.evidence.is_empty());
        assert!(output.risks.is_empty());
        assert!(output.open_questions.is_empty());
    }

    #[test]
    fn chat_role_expands_thin_forced_rich_followup() {
        let parsed = ParsedWorkerRoleOutput {
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "基于现有资料，可以直接做一套二手教材运营方案。".to_string(),
                ..RoleWorkOutput::default()
            }),
            evidence: vec![],
            risks: vec![],
            open_questions: vec![],
            diagnostics: Vec::new(),
        };
        let mut task = Task {
            id: "task-rich".to_string(),
            context: MessageContext::default(),
            description:
                "我不会给你提供任何其他信息，但是你必须根据我刚才所给资料和你的回答进行丰富回答，要有干货。"
                    .to_string(),
            requester: AgentId("user-1".to_string()),
            assignment: None,
        };
        task.context.metadata.insert(
            "conversation.history_summary".to_string(),
            "User: 我准备卖二手教材，手里有高数、英语、专业课三类书。\n\
             Assistant: 二手教材要看版本、ISBN、书况、收书成本、库存和退换。"
                .to_string(),
        );

        let output = normalize_worker_role_output("chat", &task, parsed);

        assert!(output.content.chars().count() >= MIN_FORCED_RICH_DELIVERY_CHARS);
        assert!(output.content.contains("当前结论"));
        assert!(output.content.contains("三类教材先这样拆"));
        assert!(output.content.contains("今天就能执行的动作"));
        assert!(output.content.contains("高数"));
        assert!(output.content.contains("英语"));
        assert!(output.content.contains("专业课"));
        assert!(output.content.contains("版本"));
        assert!(output.content.contains("ISBN"));
        assert!(output.content.contains("退换"));
        assert!(!output.content.contains("请补资料"));
        assert!(!output.content.contains("下一步目标"));
    }

    #[test]
    fn chat_role_does_not_expand_normal_short_answer() {
        let parsed = ParsedWorkerRoleOutput {
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "先看版本、ISBN、书况和成本，避免卖错教材或售后纠纷。".to_string(),
                ..RoleWorkOutput::default()
            }),
            evidence: vec![],
            risks: vec![],
            open_questions: vec![],
            diagnostics: Vec::new(),
        };
        let task = Task {
            id: "task-short".to_string(),
            context: MessageContext::default(),
            description: "我准备卖二手教材，请简短回答。".to_string(),
            requester: AgentId("user-1".to_string()),
            assignment: None,
        };

        let output = normalize_worker_role_output("chat", &task, parsed);

        assert_eq!(
            output.content,
            "先看版本、ISBN、书况和成本，避免卖错教材或售后纠纷。"
        );
        assert!(!output.content.contains("## 可直接执行表"));
    }

    #[test]
    fn non_chat_role_replaces_material_gap_summary_with_role_answer_frame() {
        let parsed = ParsedWorkerRoleOutput {
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "未收到可剖析的表格行数据：未命名表格，暂无法产出结果。".to_string(),
                findings: vec![],
                recommendations: vec![],
                evidence: vec![],
                risks: vec![
                    "资料边界：表格资料分析没有产出可引用资料；\
                     本轮没有收到可解析的表格行或表格文本。"
                        .to_string(),
                ],
                open_questions: vec![],
            }),
            evidence: vec![],
            risks: vec![],
            open_questions: vec![],
            diagnostics: Vec::new(),
        };
        let task = Task {
            id: "task-data".to_string(),
            context: MessageContext::default(),
            description: "请分析学生书包的转化率、主图点击率和标题卖点。".to_string(),
            requester: AgentId("user-1".to_string()),
            assignment: None,
        };

        let output = normalize_worker_role_output("data", &task, parsed);
        let role_output = output.role_output.expect("role output should exist");
        let combined = format!(
            "{}\n{}\n{}\n{}",
            role_output.summary,
            role_output.findings.join("\n"),
            role_output.recommendations.join("\n"),
            role_output.risks.join("\n")
        );

        assert!(role_output.summary.contains("数据分析师"));
        assert!(!role_output.summary.contains("暂无法产出结果"));
        assert!(combined.contains("指标口径"));
        assert!(combined.contains("先建一张基础表"));
        assert!(combined.contains("资料边界"));
        assert!(!combined.contains("skill:"));
        assert!(!combined.contains("material_table_profiler"));
    }

    #[test]
    fn direct_title_deliverable_does_not_append_default_role_policy_lines() {
        let parsed = ParsedWorkerRoleOutput {
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "轻便耐用学生书包，减轻肩部负担。\n多功能学生书包，分区清楚。\\
                          n防水学生书包，保护课本。"
                    .to_string(),
                findings: vec![],
                recommendations: vec![],
                evidence: vec![],
                risks: vec![],
                open_questions: vec![],
            }),
            evidence: vec![],
            risks: vec![],
            open_questions: vec![],
            diagnostics: Vec::new(),
        };
        let task = Task {
            id: "task-creative".to_string(),
            context: MessageContext::default(),
            description: "只输出5个学生书包标题，每个标题后写一句为什么能提高点击，不要整坨文字。"
                .to_string(),
            requester: AgentId("user-1".to_string()),
            assignment: None,
        };

        let output = normalize_worker_role_output("creative", &task, parsed);
        let role_output = output.role_output.expect("role output should exist");
        let combined = format!(
            "{}\n{}\n{}",
            role_output.summary,
            role_output.findings.join("\n"),
            role_output.recommendations.join("\n")
        );

        assert!(combined.contains("轻便耐用学生书包"));
        assert!(!combined.contains("创意岗位本轮负责"));
        assert!(!combined.contains("A/B 测试"));
        assert!(!combined.contains("点击率、停留、加购或付款"));
    }

    #[test]
    fn ops_experiment_review_output_policy_adds_review_completion_rules() {
        let parsed = ParsedWorkerRoleOutput {
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "MVP实验复盘报告已生成".to_string(),
                findings: vec!["G1 当前表现较好。".to_string()],
                recommendations: vec!["建议继续观察。".to_string()],
                evidence: vec!["experiment_review_report".to_string()],
                risks: vec![],
                open_questions: vec![],
            }),
            evidence: vec![],
            risks: vec![],
            open_questions: vec![],
            diagnostics: Vec::new(),
        };
        let task = Task {
            id: "task-review".to_string(),
            context: MessageContext::default(),
            description: "低糖冷萃咖啡实验复盘，判断哪组效果好、是否放量还是停止".to_string(),
            requester: AgentId("user-1".to_string()),
            assignment: None,
        };

        let output = normalize_worker_role_output("ops", &task, parsed);
        let role_output = output.role_output.expect("role output should exist");

        assert!(
            role_output
                .findings
                .iter()
                .any(|value| value.contains("放量、调整重测、暂停停止或样本不足"))
        );
        assert!(
            role_output
                .findings
                .iter()
                .any(|value| value.contains("赢家组合"))
        );
        assert!(
            role_output
                .recommendations
                .iter()
                .any(|value| value.contains("预算护栏"))
        );
        assert!(
            role_output
                .recommendations
                .iter()
                .any(|value| value.contains("获客成本"))
        );
        assert!(
            role_output
                .risks
                .iter()
                .any(|value| value.contains("不能编造曝光、点击、成交、GMV、ROAS 或风险率"))
        );
        assert!(
            role_output
                .risks
                .iter()
                .any(|value| value.contains("退款、差评、履约和售后"))
        );
    }

    #[test]
    fn ops_non_review_output_policy_does_not_add_experiment_review_rules() {
        let parsed = ParsedWorkerRoleOutput {
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "运营活动建议已生成".to_string(),
                findings: vec!["这次重点是活动节奏和客服承接。".to_string()],
                recommendations: vec![
                    "先确定活动目标、主推商品和页面承接。".to_string(),
                    "准备客服答疑和售后承诺边界。".to_string(),
                    "活动后按点击、加购和成交复盘。".to_string(),
                ],
                evidence: vec!["用户当前活动需求".to_string()],
                risks: vec!["不要承诺无法履约的发货时效。".to_string()],
                open_questions: vec![],
            }),
            evidence: vec![],
            risks: vec![],
            open_questions: vec![],
            diagnostics: Vec::new(),
        };
        let task = Task {
            id: "task-campaign".to_string(),
            context: MessageContext::default(),
            description: "帮我做一版端午节促销活动计划".to_string(),
            requester: AgentId("user-1".to_string()),
            assignment: None,
        };

        let output = normalize_worker_role_output("ops", &task, parsed);
        let role_output = output.role_output.expect("role output should exist");
        let combined = format!(
            "{}\n{}\n{}",
            role_output.findings.join("\n"),
            role_output.recommendations.join("\n"),
            role_output.risks.join("\n")
        );

        assert!(!combined.contains("放量、调整重测、暂停停止或样本不足"));
        assert!(!combined.contains("实验复盘需要明确当前相对赢家组合"));
        assert!(!combined.contains("不能编造曝光、点击、成交、GMV、ROAS 或风险率"));
    }
}
