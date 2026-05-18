use super::{
    RoleWorkOutput,
    runtime_noise::asks_for_specific_product_identity,
    util::{CATEGORY_LEVEL_BOUNDARY_MARKER, compact_task_text, contains_any},
    worker_policies,
};
use crate::agent::worker::{Task, runner::inference::context::first_selected_product};

pub(in crate::agent::worker::runner) fn is_category_level_task(task: &str) -> bool {
    contains_any(
        task,
        worker_policies()
            .output_normalization
            .category_level_task_keywords(),
    )
}

pub(super) fn normalize_category_level_product_requests(
    role: &str,
    task: &Task,
    output: &mut RoleWorkOutput,
) -> bool {
    if first_selected_product(task).is_some() || !is_category_level_task(&task.description) {
        return false;
    }

    if asks_for_specific_product_identity(&output.summary) {
        output.summary = worker_policies()
            .output_normalization
            .category_product_summary(
                worker_policies().output.role_display_name(role),
                &compact_task_text(&task.description),
            );
    }

    filter_product_identity_requests(&mut output.findings);
    let removed_open_questions = filter_product_identity_requests(&mut output.open_questions);
    let removed_recommendations = filter_product_identity_requests(&mut output.recommendations);
    let removed_risks = filter_product_identity_requests(&mut output.risks);
    let removed_product_specific_request =
        removed_open_questions || removed_recommendations || removed_risks;

    let output_policy = &worker_policies().output;
    let task_line = compact_task_text(&task.description);
    while output.findings.len() < output_policy.required_findings_for_role(role) {
        output.findings.push(output_policy.default_role_finding(
            role,
            output.findings.len(),
            &task_line,
        ));
    }
    if output.recommendations.is_empty() {
        output.recommendations = output_policy.default_role_recommendations(role, &task_line);
    }
    output_policy.normalize_role_specific_findings(role, &mut output.findings, &task_line);
    output_policy.normalize_role_specific_recommendations(
        role,
        &mut output.recommendations,
        &task_line,
    );
    while output.recommendations.len() < output_policy.required_recommendations_for_role(role) {
        output
            .recommendations
            .push(output_policy.default_role_recommendation(
                role,
                output.recommendations.len(),
                &task_line,
            ));
    }
    apply_category_team_output_shape(role, output);
    apply_category_boundary_output(output, removed_product_specific_request);
    super::push_unique(
        &mut output.evidence,
        CATEGORY_LEVEL_BOUNDARY_MARKER.to_string(),
    );
    true
}

fn filter_product_identity_requests(items: &mut Vec<String>) -> bool {
    let original_len = items.len();
    items.retain(|item| !asks_for_specific_product_identity(item));
    original_len != items.len()
}

fn apply_category_team_output_shape(role: &str, output: &mut RoleWorkOutput) {
    let Some(role_policy) = worker_policies().output.role(role) else {
        return;
    };
    for finding in role_policy.category_findings() {
        super::push_unique(&mut output.findings, finding.clone());
    }
    for recommendation in role_policy.category_recommendations() {
        super::push_unique(&mut output.recommendations, recommendation.clone());
    }
}

fn apply_category_boundary_output(
    output: &mut RoleWorkOutput,
    removed_product_specific_request: bool,
) {
    let category_boundary = worker_policies().output.category_boundary();
    for evidence in category_boundary.evidence() {
        super::push_unique(&mut output.evidence, evidence.clone());
    }
    for risk in category_boundary.risks() {
        super::push_unique(&mut output.risks, risk.clone());
    }
    if !removed_product_specific_request {
        return;
    }
    for evidence in category_boundary.removed_product_identity_evidence() {
        super::push_unique(&mut output.evidence, evidence.clone());
    }
    for recommendation in category_boundary.removed_product_identity_recommendations() {
        super::push_unique(&mut output.recommendations, recommendation.clone());
    }
    for risk in category_boundary.removed_product_identity_risks() {
        super::push_unique(&mut output.risks, risk.clone());
    }
}
