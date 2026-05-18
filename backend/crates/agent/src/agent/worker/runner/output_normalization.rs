use super::{
    output::{format_role_work_output_content, merge_skill_results},
    policy::worker_policies,
    types::ParsedWorkerRoleOutput,
    util::{compact_event_text, push_unique},
};
use crate::{
    agent::worker::Task,
    core::protocol::RoleWorkOutput,
    skills::{SkillExecutionResult, SkillExecutionStatus},
};

const CATEGORY_LEVEL_BOUNDARY_MARKER: &str = "category_level_boundary";

pub(super) fn finalize_worker_role_output(
    role: &str,
    task: &Task,
    parsed: ParsedWorkerRoleOutput,
    skill_results: &[SkillExecutionResult],
) -> ParsedWorkerRoleOutput {
    let parsed = merge_skill_results(parsed, skill_results);
    normalize_worker_role_output(role, task, parsed)
}

pub(super) fn is_category_level_task(task: &str) -> bool {
    contains_any(
        task,
        worker_policies()
            .output_normalization
            .category_level_task_keywords(),
    )
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
    let task_line = compact_task_text(&task.description);

    if output.summary.trim().is_empty() {
        output.summary = format!(
            "{}已基于本轮任务给出岗位判断：{}",
            output_policy.role_display_name(role),
            task_line
        );
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
            worker_policies()
                .output_normalization
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
            .push("本轮用户任务和会话/工作区上下文".to_string());
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

fn normalize_product_context_evidence(task: &Task, output: &mut RoleWorkOutput) {
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

fn normalize_upstream_evidence(task: &Task, output: &mut RoleWorkOutput) {
    let Some(assignment) = &task.assignment else {
        return;
    };

    for input in &assignment.inputs {
        push_unique(
            &mut output.evidence,
            format!(
                "worker.assignment.input.{}：引用上游{}岗位产物，保持本轮团队信息打通。",
                input.node_id.0,
                worker_policies().output.role_display_name(&input.role)
            ),
        );
    }
}

fn normalize_category_level_product_requests(
    role: &str,
    task: &Task,
    output: &mut RoleWorkOutput,
) -> bool {
    if first_selected_product(task).is_some() || !is_category_level_task(&task.description) {
        return false;
    }

    if asks_for_specific_product_identity(&output.summary) {
        output.summary = format!(
            "{}先按类目级或未指定商品链路回答：{}",
            worker_policies().output.role_display_name(role),
            compact_task_text(&task.description)
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
    push_unique(
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
        push_unique(&mut output.findings, finding.clone());
    }
    for recommendation in role_policy.category_recommendations() {
        push_unique(&mut output.recommendations, recommendation.clone());
    }
}

fn apply_category_boundary_output(
    output: &mut RoleWorkOutput,
    removed_product_specific_request: bool,
) {
    let category_boundary = worker_policies().output.category_boundary();
    for evidence in category_boundary.evidence() {
        push_unique(&mut output.evidence, evidence.clone());
    }
    for risk in category_boundary.risks() {
        push_unique(&mut output.risks, risk.clone());
    }
    if !removed_product_specific_request {
        return;
    }
    for evidence in category_boundary.removed_product_identity_evidence() {
        push_unique(&mut output.evidence, evidence.clone());
    }
    for recommendation in category_boundary.removed_product_identity_recommendations() {
        push_unique(&mut output.recommendations, recommendation.clone());
    }
    for risk in category_boundary.removed_product_identity_risks() {
        push_unique(&mut output.risks, risk.clone());
    }
}

fn sanitize_category_level_runtime_artifacts(output: &mut RoleWorkOutput) {
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
        output.summary = worker_policies()
            .output_normalization
            .category_runtime_summary_replacement()
            .to_string();
    }
}

fn is_category_runtime_noise(value: &str) -> bool {
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
        worker_policies()
            .output_normalization
            .category_boundary_explanation_keywords(),
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

fn sanitize_unsupported_benchmark_claims(output: &mut RoleWorkOutput) -> bool {
    let mut removed = false;
    if is_unsupported_benchmark_claim(&output.summary) {
        output.summary = worker_policies()
            .output_normalization
            .unsupported_benchmark()
            .summary_replacement()
            .to_string();
        removed = true;
    }
    removed |= retain_supported_benchmark_items(&mut output.findings);
    removed |= retain_supported_benchmark_items(&mut output.recommendations);
    removed |= retain_supported_benchmark_items(&mut output.risks);
    removed |= retain_supported_benchmark_items(&mut output.open_questions);
    removed |= retain_supported_benchmark_items(&mut output.evidence);
    removed
}

fn retain_supported_benchmark_items(items: &mut Vec<String>) -> bool {
    let original_len = items.len();
    items.retain(|item| !is_unsupported_benchmark_claim(item));
    original_len != items.len()
}

fn is_unsupported_benchmark_claim(value: &str) -> bool {
    let text = value.trim();
    if text.is_empty() {
        return false;
    }
    let policy = worker_policies()
        .output_normalization
        .unsupported_benchmark();
    if contains_any(text, policy.allowed_source_markers()) {
        return false;
    }

    if contains_any(text, policy.assertive_comparison_terms()) {
        return true;
    }

    if !contains_any(text, policy.benchmark_terms()) {
        return false;
    }

    let asks_for_benchmark = contains_any(text, policy.benchmark_request_terms());
    if asks_for_benchmark && !contains_benchmark_number_claim(text) {
        return false;
    }
    contains_any(text, policy.assertive_benchmark_terms()) || contains_benchmark_number_claim(text)
}

fn contains_benchmark_number_claim(value: &str) -> bool {
    worker_policies()
        .output_normalization
        .unsupported_benchmark()
        .benchmark_number_terms()
        .iter()
        .any(|term| {
            value.find(term).is_some_and(|index| {
                let after = &value[index ..];
                let before = &value[.. index];
                segment_after_contains_percent_or_range(after)
                    || segment_before_contains_percent_or_range(before)
            })
        })
}

fn segment_after_contains_percent_or_range(value: &str) -> bool {
    let segment = value
        .split(['，', '。', '；', ';', ',', '\n'])
        .next()
        .unwrap_or(value);
    contains_percent_or_range(segment)
}

fn segment_before_contains_percent_or_range(value: &str) -> bool {
    let segment = value
        .rsplit(['，', '。', '；', ';', ',', '\n'])
        .next()
        .unwrap_or(value);
    contains_percent_or_range(segment)
}

fn contains_percent_or_range(value: &str) -> bool {
    let mut previous_was_digit = false;
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            previous_was_digit = true;
            continue;
        }
        if previous_was_digit && matches!(ch, '%' | '％' | '-' | '~' | '—' | '至') {
            return true;
        }
        if !matches!(ch, '.' | ' ' | '\t') {
            previous_was_digit = false;
        }
    }
    false
}

fn asks_for_specific_product_identity(question: &str) -> bool {
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

fn merge_unique(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    for item in right {
        if item.trim().is_empty() || left.iter().any(|value| value == &item) {
            continue;
        }
        left.push(item);
    }
    left
}

fn contains_any(value: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn compact_task_text(value: &str) -> String {
    compact_event_text(value.trim().replace('\n', " "))
}

#[derive(Debug, Clone)]
struct SelectedProductContext {
    id: String,
    name: String,
}

impl SelectedProductContext {
    fn ref_key(&self) -> String {
        if self.id.starts_with("product:") {
            self.id.clone()
        } else {
            format!("product:{}", self.id)
        }
    }
}

fn first_selected_product(task: &Task) -> Option<SelectedProductContext> {
    selected_products_from_task(task).into_iter().next()
}

fn selected_products_from_task(task: &Task) -> Vec<SelectedProductContext> {
    let Some(summary) = task
        .context
        .metadata
        .get("context.selected_summary")
        .or_else(|| {
            task.context
                .metadata
                .get("commerce.selected_product_context")
        })
        .map(String::as_str)
    else {
        return Vec::new();
    };

    summary
        .lines()
        .filter_map(parse_product_summary_line)
        .collect()
}

fn parse_product_summary_line(line: &str) -> Option<SelectedProductContext> {
    let line = line.trim();
    let rest = line.strip_prefix("Product source product:")?;
    let mut parts = rest.splitn(2, '|');
    let id = parts.next()?.trim();
    let fields = parse_pipe_fields(parts.next().unwrap_or_default());
    let name = fields.get("name")?.trim();
    if id.is_empty() || name.is_empty() {
        return None;
    }
    Some(SelectedProductContext {
        id: id.to_string(),
        name: name.to_string(),
    })
}

fn parse_pipe_fields(value: &str) -> std::collections::BTreeMap<String, String> {
    value
        .split('|')
        .filter_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}
