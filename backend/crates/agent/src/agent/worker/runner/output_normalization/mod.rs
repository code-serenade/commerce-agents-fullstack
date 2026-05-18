mod benchmark;
mod category_boundary;
mod evidence;
mod product_context;
mod runtime_noise;
mod util;

use benchmark::sanitize_unsupported_benchmark_claims;
pub(super) use category_boundary::is_category_level_task;
use category_boundary::normalize_category_level_product_requests;
use evidence::{merge_unique, normalize_product_context_evidence, normalize_upstream_evidence};
use runtime_noise::{is_category_runtime_noise, sanitize_category_level_runtime_artifacts};
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

    parsed.evidence = merge_unique(parsed.evidence, output.evidence.clone());
    parsed.risks = merge_unique(parsed.risks, output.risks.clone());
    parsed.open_questions = merge_unique(parsed.open_questions, output.open_questions.clone());
    parsed.content = format_role_work_output_content(output);
    parsed
}
