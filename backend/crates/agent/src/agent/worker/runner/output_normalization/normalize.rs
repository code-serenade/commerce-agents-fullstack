use super::*;
use super::normalization_policy::policy as normalization_policy;

pub(super) fn normalize_worker_role_output(
    role: &str,
    task: &Task,
    mut parsed: ParsedWorkerRoleOutput,
) -> ParsedWorkerRoleOutput {
    let output = parsed
        .role_output
        .get_or_insert_with(crate::core::protocol::RoleWorkOutput::default);

    let task_line = compact_task_text(&task.description);
    if output.summary.trim().is_empty() {
        output.summary = format!(
            "{}已基于本轮任务给出岗位判断：{}",
            role_display_name(role),
            task_line
        );
    }

    if output.findings.is_empty() {
        output.findings = default_role_findings(role, &task_line);
    }
    let required_findings = required_findings_for_role(role);
    while output.findings.len() < required_findings {
        output.findings.push(default_role_finding(
            role,
            output.findings.len(),
            &task_line,
        ));
    }
    normalize_role_specific_findings(role, &mut output.findings, &task_line);

    if output.recommendations.is_empty() {
        output.recommendations = default_role_recommendations(role, &task_line);
    }
    let required_recommendations = required_recommendations_for_role(role);
    while output.recommendations.len() < required_recommendations {
        output.recommendations.push(default_role_recommendation(
            role,
            output.recommendations.len(),
            &task_line,
        ));
    }
    normalize_role_specific_recommendations(role, &mut output.recommendations, &task_line);

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
    let removed_unsupported_benchmark_claims = sanitize_unsupported_benchmark_claims(output);
    if removed_unsupported_benchmark_claims {
        push_unique(
            &mut output.risks,
            normalization_policy()
                .unsupported_benchmark()
                .risk()
                .to_string(),
        );
        while output.findings.len() < required_findings {
            output.findings.push(default_role_finding(
                role,
                output.findings.len(),
                &task_line,
            ));
        }
        while output.recommendations.len() < required_recommendations {
            output.recommendations.push(default_role_recommendation(
                role,
                output.recommendations.len(),
                &task_line,
            ));
        }
    }
    if output.evidence.is_empty() {
        output
            .evidence
            .push("本轮用户任务和会话/工作区上下文".to_string());
    }
    if output.risks.is_empty() && role_requires_risk(role) {
        output.risks = default_role_risks(role);
    }
    normalize_role_specific_risks(role, &mut output.risks);

    parsed.evidence = merge_unique(parsed.evidence, output.evidence.clone());
    parsed.risks = merge_unique(parsed.risks, output.risks.clone());
    parsed.open_questions = merge_unique(parsed.open_questions, output.open_questions.clone());
    parsed.content = format_role_work_output_content(output);
    parsed
}
