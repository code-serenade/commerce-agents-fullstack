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
    output_policy.normalize_role_specific_risks(role, &mut output.risks);
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

        assert_eq!(output.content, "回答\n- 先看真实资料");
        assert_eq!(role_output.summary, "回答\n- 先看真实资料");
        assert!(role_output.findings.is_empty());
        assert!(role_output.recommendations.is_empty());
        assert!(role_output.evidence.is_empty());
        assert!(role_output.risks.is_empty());
        assert!(role_output.open_questions.is_empty());
        assert!(output.evidence.is_empty());
        assert!(output.risks.is_empty());
        assert!(output.open_questions.is_empty());
    }
}
