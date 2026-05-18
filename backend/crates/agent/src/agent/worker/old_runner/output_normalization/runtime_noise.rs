use super::*;
use super::normalization_policy::{
    contains_any, contains_any_case_insensitive_ascii, policy as normalization_policy,
};

pub(super) fn sanitize_category_level_runtime_artifacts(output: &mut RoleWorkOutput) {
    output
        .findings
        .retain(|item| !is_category_runtime_noise(item));
    output
        .recommendations
        .retain(|item| !is_category_runtime_noise(item));
    output.risks.retain(|item| !is_category_runtime_noise(item));
    output
        .open_questions
        .retain(|item| !is_category_runtime_noise(item));
    output.evidence.retain(|item| {
        item == CATEGORY_LEVEL_BOUNDARY_MARKER
            || item.contains("worker.assignment.input")
            || is_category_boundary_explanation(item)
            || !is_category_runtime_noise(item)
    });
    if is_category_runtime_noise(&output.summary) || contains_refusal_summary(&output.summary) {
        output.summary = normalization_policy()
            .category_runtime_summary_replacement()
            .to_string();
    }
}

pub(super) fn is_category_runtime_noise(value: &str) -> bool {
    let text = value.trim();
    if text.is_empty() {
        return false;
    }
    asks_for_specific_product_identity(text)
        || contains_raw_skill_runtime_text(text)
        || contains_refusal_summary(text)
}

fn is_category_boundary_explanation(value: &str) -> bool {
    contains_any(
        value,
        normalization_policy().category_boundary_explanation_keywords(),
    )
}

fn contains_raw_skill_runtime_text(value: &str) -> bool {
    contains_any_case_insensitive_ascii(value, normalization_policy().runtime_noise_keywords())
}

pub(super) fn contains_refusal_summary(value: &str) -> bool {
    contains_any(value, normalization_policy().refusal_summary_keywords())
}

#[cfg(test)]
fn normalize_worker_role_output_for_test(
    role: &str,
    task_description: &str,
    parsed: ParsedWorkerRoleOutput,
) -> ParsedWorkerRoleOutput {
    normalize_worker_role_output_for_test_with_metadata(role, task_description, &[], parsed)
}

#[cfg(test)]
fn normalize_worker_role_output_for_test_with_metadata(
    role: &str,
    task_description: &str,
    metadata: &[(&str, &str)],
    parsed: ParsedWorkerRoleOutput,
) -> ParsedWorkerRoleOutput {
    let mut context = crate::core::protocol::MessageContext::default();
    for (key, value) in metadata {
        context
            .metadata
            .insert((*key).to_string(), (*value).to_string());
    }
    normalize_worker_role_output(
        role,
        &Task {
            id: "task-test".to_string(),
            context,
            description: task_description.to_string(),
            requester: AgentId("agent.test".to_string()),
            assignment: None,
        },
        parsed,
    )
}
