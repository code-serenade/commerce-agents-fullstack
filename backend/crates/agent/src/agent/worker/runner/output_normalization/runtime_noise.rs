use super::{
    RoleWorkOutput,
    util::{CATEGORY_LEVEL_BOUNDARY_MARKER, contains_any},
    worker_policies,
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
            || should_keep_runtime_evidence(item)
            || is_category_boundary_explanation(item)
            || !is_category_runtime_noise(item)
    });
    if is_category_runtime_noise(&output.summary) || contains_refusal_summary(&output.summary) {
        output.summary = worker_policies()
            .output_normalization
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

pub(super) fn asks_for_specific_product_identity(question: &str) -> bool {
    if describes_unselected_product_boundary(question) {
        return false;
    }
    contains_any(
        question,
        worker_policies()
            .output_normalization
            .specific_product_identity_keywords(),
    )
}

fn describes_unselected_product_boundary(value: &str) -> bool {
    contains_any(
        value,
        worker_policies()
            .output_normalization
            .unselected_product_boundary_keywords(),
    )
}

fn is_category_boundary_explanation(value: &str) -> bool {
    contains_any(
        value,
        worker_policies()
            .output_normalization
            .category_boundary_explanation_keywords(),
    )
}

fn should_keep_runtime_evidence(value: &str) -> bool {
    contains_any(
        value,
        worker_policies()
            .output_normalization
            .runtime_evidence_keep_markers(),
    )
}

fn contains_raw_skill_runtime_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    worker_policies()
        .output_normalization
        .runtime_noise_keywords()
        .iter()
        .any(|needle| lower.contains(needle))
}

fn contains_refusal_summary(value: &str) -> bool {
    contains_any(
        value,
        worker_policies()
            .output_normalization
            .refusal_summary_keywords(),
    )
}
