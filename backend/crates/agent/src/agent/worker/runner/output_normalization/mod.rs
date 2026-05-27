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
use crate::{agent::worker::Task, core::protocol::RoleWorkOutput, skills::SkillExecutionResult};

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

    if role == "chat" {
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

    if output.findings.is_empty() {
        output.findings = output_policy.default_role_findings(role, &task_line);
    }
    while output.findings.len() < output_policy.required_findings_for_role(role) {
        output.findings.push(output_policy.default_role_finding(
            role,
            output.findings.len(),
            &task_line,
        ));
    }
    output_policy.normalize_role_specific_findings(role, &mut output.findings, &task_line);

    if output.recommendations.is_empty() {
        output.recommendations = output_policy.default_role_recommendations(role, &task_line);
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
    output_policy.normalize_role_specific_recommendations(
        role,
        &mut output.recommendations,
        &task_line,
    );

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

#[cfg(test)]
mod tests {
    use super::{RoleWorkOutput, normalize_worker_role_output};
    use crate::{
        agent::worker::{Task, runner::types::ParsedWorkerRoleOutput},
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
