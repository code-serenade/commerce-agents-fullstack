use super::{RoleWorkOutput, util::merge_unique as merge_unique_items, worker_policies};
use crate::agent::worker::{Task, runner::inference::context::first_selected_product};

pub(super) fn normalize_product_context_evidence(task: &Task, output: &mut RoleWorkOutput) {
    if let Some(product) = first_selected_product(task) {
        super::push_unique(
            &mut output.evidence,
            worker_policies()
                .output_normalization
                .selected_product_evidence(product.name(), &product.ref_key()),
        );
        return;
    }

    super::push_unique(
        &mut output.evidence,
        worker_policies()
            .output_normalization
            .unselected_product_evidence()
            .to_string(),
    );
}

pub(super) fn normalize_upstream_evidence(task: &Task, output: &mut RoleWorkOutput) {
    let Some(assignment) = &task.assignment else {
        return;
    };

    for input in &assignment.inputs {
        super::push_unique(
            &mut output.evidence,
            worker_policies().output_normalization.upstream_evidence(
                &input.node_id.0,
                worker_policies().output.role_display_name(&input.role),
            ),
        );
    }
}

pub(super) fn merge_unique(left: Vec<String>, right: Vec<String>) -> Vec<String> {
    merge_unique_items(left, right)
}
