use super::*;
use super::normalization_policy::{contains_any, policy as normalization_policy};

pub(super) fn assignment_user_task(assignment: &WorkerAssignment) -> String {
    assignment
        .objective
        .rsplit_once('：')
        .map(|(_, task)| task.trim())
        .filter(|task| !task.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| assignment.objective.clone())
}

pub(super) fn normalize_category_level_product_requests(
    role: &str,
    task: &Task,
    output: &mut crate::core::protocol::RoleWorkOutput,
) -> bool {
    if first_selected_product(task).is_some() || !is_category_level_task(&task.description) {
        return false;
    }

    if asks_for_specific_product_identity(&output.summary) {
        output.summary = format!(
            "{}先按类目级或未指定商品链路回答：{}",
            role_display_name(role),
            compact_task_text(&task.description)
        );
    }
    filter_product_identity_requests(&mut output.findings);
    let removed_open_questions = filter_product_identity_requests(&mut output.open_questions);
    let removed_recommendations = filter_product_identity_requests(&mut output.recommendations);
    let removed_risks = filter_product_identity_requests(&mut output.risks);
    let removed_product_specific_request =
        removed_open_questions || removed_recommendations || removed_risks;
    let task_line = compact_task_text(&task.description);
    while output.findings.len() < required_findings_for_role(role) {
        output.findings.push(default_role_finding(
            role,
            output.findings.len(),
            &task_line,
        ));
    }
    if output.recommendations.is_empty() {
        output.recommendations = default_role_recommendations(role, &task_line);
    }
    normalize_role_specific_findings(role, &mut output.findings, &task_line);
    normalize_role_specific_recommendations(role, &mut output.recommendations, &task_line);
    while output.recommendations.len() < required_recommendations_for_role(role) {
        output.recommendations.push(default_role_recommendation(
            role,
            output.recommendations.len(),
            &task_line,
        ));
    }
    apply_category_team_output_shape(role, output);

    apply_category_boundary_output(output, removed_product_specific_request);
    push_unique(
        &mut output.evidence,
        CATEGORY_LEVEL_BOUNDARY_MARKER.to_string(),
    );

    if removed_product_specific_request {
        push_unique(
            &mut output.evidence,
            CATEGORY_LEVEL_BOUNDARY_MARKER.to_string(),
        );
    }
    true
}

fn filter_product_identity_requests(items: &mut Vec<String>) -> bool {
    let original_len = items.len();
    items.retain(|item| !asks_for_specific_product_identity(item));
    original_len != items.len()
}

pub(super) fn is_non_blocking_category_skill_gap(result: &SkillExecutionResult, task: &Task) -> bool {
    if first_selected_product(task).is_some() || !is_category_level_task(&task.description) {
        return false;
    }
    if !matches!(
        result.status,
        SkillExecutionStatus::Deferred
            | SkillExecutionStatus::NotFound
            | SkillExecutionStatus::ValidationError
            | SkillExecutionStatus::Failed
    ) {
        return false;
    }

    if is_category_non_blocking_skill(result.skill_id.as_str()) {
        return true;
    }

    result.output.as_ref().is_some_and(|output| {
        output
            .get("data_readiness")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == "missing_user_input")
            || output
                .get("adapter_status")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| {
                    value.contains("required")
                        || value.contains("missing")
                        || value.contains("not_configured")
                })
    }) || result.error.as_deref().is_some_and(|error| {
        asks_for_specific_product_identity(error)
            || contains_runtime_data_gap(error)
            || contains_role_skill_runtime_gap(error)
    }) || result.summary.as_deref().is_some_and(|summary| {
        asks_for_specific_product_identity(summary)
            || contains_runtime_data_gap(summary)
            || contains_role_skill_runtime_gap(summary)
    })
}

fn contains_role_skill_runtime_gap(value: &str) -> bool {
    contains_any(
        value,
        normalization_policy().role_skill_runtime_gap_keywords(),
    )
}

pub(super) fn contains_runtime_data_gap(value: &str) -> bool {
    contains_any(value, normalization_policy().runtime_data_gap_keywords())
}

pub(super) fn format_category_level_skill_boundary(result: &SkillExecutionResult) -> String {
    normalization_policy()
        .category_skill_boundary_output(&category_skill_boundary_label(result.skill_id.as_str()))
}

pub(super) fn category_level_skill_boundary_output(result: &SkillExecutionResult) -> serde_json::Value {
    let mut output = result
        .output
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));
    let original_status = format!("{:?}", result.status);
    let original_message = result
        .error
        .as_deref()
        .or(result.summary.as_deref())
        .map(compact_task_text);
    if let Some(object) = output.as_object_mut() {
        object.insert(
            "boundary_marker".to_string(),
            serde_json::json!(CATEGORY_LEVEL_BOUNDARY_MARKER),
        );
        object.insert(
            "boundary_policy".to_string(),
            serde_json::json!("category_level_without_selected_product"),
        );
        object.insert(
            "boundary_label".to_string(),
            serde_json::json!(category_skill_boundary_label(result.skill_id.as_str())),
        );
        object.insert(
            "original_status".to_string(),
            serde_json::json!(original_status),
        );
        if let Some(message) = original_message {
            object.insert("original_message".to_string(), serde_json::json!(message));
        }
    } else {
        output = serde_json::json!({
            "value": output,
            "boundary_marker": CATEGORY_LEVEL_BOUNDARY_MARKER,
            "boundary_policy": "category_level_without_selected_product",
            "boundary_label": category_skill_boundary_label(result.skill_id.as_str()),
            "original_status": original_status,
            "original_message": original_message,
        });
    }
    output
}

pub(super) fn should_downgrade_category_worker_failure(task: &Task, reason: &str) -> bool {
    first_selected_product(task).is_none()
        && is_category_level_task(&task.description)
        && (asks_for_specific_product_identity(reason) || contains_runtime_data_gap(reason))
}

pub(super) fn is_category_level_task(task: &str) -> bool {
    contains_any(task, normalization_policy().category_level_task_keywords())
}

pub(super) fn asks_for_specific_product_identity(question: &str) -> bool {
    if describes_unselected_product_boundary(question) {
        return false;
    }
    contains_any(
        question,
        normalization_policy().specific_product_identity_keywords(),
    )
}

fn describes_unselected_product_boundary(value: &str) -> bool {
    contains_any(
        value,
        normalization_policy().unselected_product_boundary_keywords(),
    )
}
