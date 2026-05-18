use super::config::{AutosuggestRule, AutosuggestRuleMode, WorkerAutosuggestPolicy};
use crate::agent::worker::{
    Task,
    runner::{
        inference::{context, metrics, numbers},
        types::ParsedSkillRequest,
    },
};

pub(super) fn configured_rule_requests(
    policy: &WorkerAutosuggestPolicy,
    role_runtime: &str,
    text: &str,
    task: &Task,
    route_mode: &str,
) -> Vec<ParsedSkillRequest> {
    let mut requests = Vec::new();
    for rule in &policy.rules {
        if !rule_matches(rule, role_runtime, text) {
            continue;
        }

        requests.push(ParsedSkillRequest {
            skill_id: rule.skill_id.clone(),
            input: render_template_value(&rule.input, text, task),
            reason: rule
                .reason
                .clone()
                .or_else(|| Some(format!("autosuggest:{}", rule.id))),
        });

        if route_mode != "requests" || rule.mode == AutosuggestRuleMode::First {
            break;
        }
    }
    requests
}

pub(super) fn contains_any(text: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| contains_keyword(text, needle))
}

fn rule_matches(rule: &AutosuggestRule, role_runtime: &str, text: &str) -> bool {
    (rule.roles.is_empty() || rule.roles.iter().any(|role| role == role_runtime))
        && (rule.r#match.any_keywords.is_empty() || contains_any(text, &rule.r#match.any_keywords))
        && rule
            .r#match
            .all_keywords
            .iter()
            .all(|keyword| contains_keyword(text, keyword))
        && !contains_any(text, &rule.r#match.none_keywords)
}

fn render_template_value(value: &serde_json::Value, text: &str, task: &Task) -> serde_json::Value {
    match value {
        serde_json::Value::String(template) => {
            serde_json::Value::String(render_template_string(template, text, task))
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(|item| render_template_value(item, text, task))
                .collect(),
        ),
        serde_json::Value::Object(object) => serde_json::Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), render_template_value(value, text, task)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn render_template_string(template: &str, text: &str, task: &Task) -> String {
    template
        .replace("{{task_text}}", text.trim())
        .replace("{{task_description}}", task.description.trim())
        .replace("{{task_id}}", task.id.as_str())
        .replace("{{platform}}", &metrics::infer_platform(text))
        .replace(
            "{{store_metrics_platform}}",
            &metrics::infer_store_metrics_platform(text),
        )
        .replace("{{stage}}", &metrics::infer_stage(text))
        .replace("{{metric}}", metrics::infer_metric(text))
        .replace(
            "{{period_days}}",
            &numbers::infer_period_days(text).to_string(),
        )
        .replace(
            "{{selected_product_ref}}",
            context::selected_product_ref(task).as_deref().unwrap_or(""),
        )
        .replace(
            "{{selected_product_name}}",
            &context::selected_product_name_or_task(task),
        )
}

fn contains_keyword(text: &str, needle: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
}
