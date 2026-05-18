use super::*;

pub(super) fn normalize_product_context_evidence(
    task: &Task,
    output: &mut crate::core::protocol::RoleWorkOutput,
) {
    if let Some(product) = first_selected_product(task) {
        push_unique(
            &mut output.evidence,
            format!(
                "用户本轮手动选择的预设商品资料：{}（{}，来源 context.selected_summary）。",
                product.name,
                product.ref_key()
            ),
        );
        return;
    }

    push_unique(
        &mut output.evidence,
        "本轮未检测到 context.selected_summary；不注入任何预设商品资料，只按用户文本、\
         会话历史和工作区上下文理解问题。"
            .to_string(),
    );
}

pub(super) fn normalize_upstream_evidence(task: &Task, output: &mut crate::core::protocol::RoleWorkOutput) {
    let Some(assignment) = &task.assignment else {
        return;
    };

    for input in &assignment.inputs {
        push_unique(
            &mut output.evidence,
            format!(
                "worker.assignment.input.{}：引用上游{}岗位产物，保持本轮团队信息打通。",
                input.node_id.0,
                role_display_name(&input.role)
            ),
        );
    }
}

pub(super) fn merge_unique(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    for item in right {
        if item.trim().is_empty() || left.iter().any(|value| value == &item) {
            continue;
        }
        left.push(item);
    }
    left
}

fn contains_any_in_items(items: &[String], needles: &[&str]) -> bool {
    items.iter().any(|item| contains_any(item, needles))
}

