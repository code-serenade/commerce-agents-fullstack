use super::config::{AutosuggestRule, AutosuggestRuleMode, WorkerAutosuggestPolicy};
use crate::agent::worker::{
    Task,
    runner::{
        inference::{commerce, context, domain, memory_task, metrics, numbers},
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
        serde_json::Value::String(template) => render_template_scalar(template, text, task),
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

fn render_template_scalar(template: &str, text: &str, task: &Task) -> serde_json::Value {
    let rendered = render_template_string(template, text, task);
    if template.trim().starts_with("{{")
        && template.trim().ends_with("}}")
        && let Ok(value) = rendered.parse::<f64>()
        && value.is_finite()
    {
        return serde_json::json!(value);
    }
    serde_json::Value::String(rendered)
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
            "{{integer}}",
            &numbers::infer_integer_or_default(text, 1).to_string(),
        )
        .replace(
            "{{percent}}",
            &numbers::infer_percent_number(text, 0.0).to_string(),
        )
        .replace(
            "{{second_percent}}",
            &numbers::infer_second_percent_number(text, 0.0).to_string(),
        )
        .replace("{{cohort_unit}}", metrics::infer_cohort_unit(text))
        .replace(
            "{{seasonal_period}}",
            &metrics::infer_seasonal_period(text).to_string(),
        )
        .replace("{{memory_key}}", &memory_task::infer_memory_key(text))
        .replace("{{memory_type}}", memory_task::infer_memory_type(text))
        .replace(
            "{{memory_content}}",
            &memory_task::infer_memory_content(text, task),
        )
        .replace("{{task_title}}", &memory_task::infer_task_title(text, task))
        .replace(
            "{{task_owner_role}}",
            memory_task::infer_task_owner_role(text),
        )
        .replace(
            "{{task_priority}}",
            &memory_task::infer_task_priority(text).to_string(),
        )
        .replace(
            "{{task_acceptance}}",
            &memory_task::infer_task_acceptance(text),
        )
        .replace("{{return_reason}}", &domain::infer_return_reason(text))
        .replace(
            "{{service_issue_type}}",
            &domain::infer_service_issue_type(text),
        )
        .replace("{{service_emotion}}", &domain::infer_service_emotion(text))
        .replace("{{design_usage}}", &domain::infer_design_usage(text))
        .replace("{{design_style}}", &domain::infer_design_style(text))
        .replace("{{page_type}}", &domain::infer_page_type(text))
        .replace("{{discount_hint}}", &domain::infer_discount_hint(text))
        .replace("{{video_type}}", &domain::infer_video_type(text))
        .replace(
            "{{video_duration_seconds}}",
            &domain::infer_video_duration_seconds(text).to_string(),
        )
        .replace("{{audience}}", &domain::infer_audience(text))
        .replace("{{category}}", &domain::infer_category(text))
        .replace("{{product_name}}", &domain::infer_product_name(text))
        .replace("{{campaign_name}}", &domain::infer_campaign_name(text))
        .replace("{{campaign_type}}", &domain::infer_campaign_type(text))
        .replace("{{cost}}", &commerce::infer_cost(text, 1.0).to_string())
        .replace(
            "{{current_price}}",
            &commerce::infer_current_price(text, 9.9).to_string(),
        )
        .replace(
            "{{budget}}",
            &commerce::infer_budget(text, &["预算", "budget"], 10_000.0).to_string(),
        )
        .replace("{{system_type}}", &domain::infer_system_type(text))
        .replace("{{bug_type}}", &domain::infer_bug_type(text))
        .replace("{{bug_frequency}}", &domain::infer_bug_frequency(text))
        .replace("{{perf_bottleneck}}", &domain::infer_perf_bottleneck(text))
        .replace("{{stack_hint}}", &domain::infer_stack_hint(text))
        .replace("{{tech_domain}}", &domain::infer_tech_domain(text))
        .replace(
            "{{status_query_type}}",
            &domain::infer_status_query_type(text),
        )
        .replace(
            "{{platform_health_check_platform}}",
            &domain::infer_platform_health_check_platform(text),
        )
        .replace("{{learning_topic}}", &domain::infer_learning_topic(text))
        .replace("{{learning_level}}", &domain::infer_learning_level(text))
        .replace(
            "{{learning_scenario}}",
            &domain::infer_learning_scenario(text),
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
