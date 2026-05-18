use super::{
    policy::{AutosuggestRule, AutosuggestRuleMode, WorkerAutosuggestPolicy, worker_policies},
    types::ParsedSkillRequest,
};
use crate::agent::worker::Task;

pub(super) fn autosuggest_skill_requests(
    role_runtime: &str,
    task: &Task,
    existing_requests: &[ParsedSkillRequest],
) -> Vec<ParsedSkillRequest> {
    let policy = &worker_policies().autosuggest;
    let text = autosuggest_text(task);
    let Some(route) = policy
        .role_routes
        .iter()
        .find(|route| route.role == role_runtime)
    else {
        return Vec::new();
    };

    let mut requests = Vec::new();
    if route.learning_first
        && let Some(request) = learning_request(policy, &text)
    {
        requests.push(request);
        if route.mode != "requests" {
            return dedupe_new_requests(requests, existing_requests);
        }
    }

    match route.handler.as_str() {
        "chat" => extend_optional(&mut requests, chat_request(policy, &text, task)),
        "search" => requests.extend(search_requests(policy, &text, task)),
        _ => requests.extend(configured_rule_requests(
            policy,
            role_runtime,
            &text,
            task,
            &route.mode,
        )),
    }

    if route.handler != "search" {
        if route.mode == "requests" {
            requests.extend(search_requests(policy, &text, task));
        } else if requests.is_empty() {
            extend_optional(&mut requests, first_search_request(policy, &text, task));
        }
    }

    dedupe_new_requests(requests, existing_requests)
}

fn chat_request(
    policy: &WorkerAutosuggestPolicy,
    text: &str,
    task: &Task,
) -> Option<ParsedSkillRequest> {
    if contains_any(text, &policy.chat.save_memory_keywords) {
        return Some(ParsedSkillRequest {
            skill_id: policy.chat.save_memory_skill_id.clone(),
            input: serde_json::json!({
                "content": task.description,
                "memory_type": "user_instruction"
            }),
            reason: Some("autosuggest:chat.save_memory".to_string()),
        });
    }
    if contains_any(text, &policy.chat.status_query_keywords) {
        return Some(ParsedSkillRequest {
            skill_id: policy.chat.status_query_skill_id.clone(),
            input: serde_json::json!({
                "query_type": "session_status"
            }),
            reason: Some("autosuggest:chat.status_query".to_string()),
        });
    }
    learning_request(policy, text)
}

fn learning_request(policy: &WorkerAutosuggestPolicy, text: &str) -> Option<ParsedSkillRequest> {
    let has_explicit_keyword = contains_any(text, &policy.learning.explicit_keywords);
    let has_question_with_explanation = contains_any(text, &policy.learning.question_keywords)
        && contains_any(text, &policy.learning.explanation_keywords);
    if !has_explicit_keyword && !has_question_with_explanation {
        return None;
    }

    Some(ParsedSkillRequest {
        skill_id: policy.learning.skill_id.clone(),
        input: serde_json::json!({
            "topic": text.trim(),
            "level": "practical",
            "include_self_check": !contains_any(text, &policy.learning.self_check_disabled_keywords)
        }),
        reason: Some("autosuggest:learning".to_string()),
    })
}

fn search_requests(
    policy: &WorkerAutosuggestPolicy,
    text: &str,
    task: &Task,
) -> Vec<ParsedSkillRequest> {
    let mut requests = Vec::new();
    if contains_any(text, &policy.search.competitor_keywords) {
        requests.push(search_request(
            policy.search.competitor_skill_id.clone(),
            "competitor",
            text,
            task,
        ));
    }
    if contains_any(text, &policy.search.trend_keywords) {
        requests.push(search_request(
            policy.search.trends_skill_id.clone(),
            "trend",
            text,
            task,
        ));
    }
    if contains_any(text, &policy.search.market_info_keywords) {
        requests.push(search_request(
            policy.search.market_info_skill_id.clone(),
            "market_info",
            text,
            task,
        ));
    }
    requests
}

fn first_search_request(
    policy: &WorkerAutosuggestPolicy,
    text: &str,
    task: &Task,
) -> Option<ParsedSkillRequest> {
    search_requests(policy, text, task).into_iter().next()
}

fn search_request(
    skill_id: String,
    search_type: &str,
    text: &str,
    task: &Task,
) -> ParsedSkillRequest {
    ParsedSkillRequest {
        skill_id,
        input: serde_json::json!({
            "query": text.trim(),
            "subject": task.description.trim(),
            "search_type": search_type
        }),
        reason: Some(format!("autosuggest:search.{search_type}")),
    }
}

fn configured_rule_requests(
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
}

fn autosuggest_text(task: &Task) -> String {
    let mut parts = vec![task.description.as_str()];
    if let Some(assignment) = &task.assignment {
        parts.push(assignment.objective.as_str());
        parts.extend(assignment.inputs.iter().map(|input| input.content.as_str()));
        if let Some(instruction) = &assignment.rework_instruction {
            parts.push(instruction.as_str());
        }
    }
    parts.join("\n")
}

fn extend_optional<T>(items: &mut Vec<T>, item: Option<T>) {
    if let Some(item) = item {
        items.push(item);
    }
}

fn dedupe_new_requests(
    requests: Vec<ParsedSkillRequest>,
    existing_requests: &[ParsedSkillRequest],
) -> Vec<ParsedSkillRequest> {
    let mut seen = std::collections::HashSet::new();
    for request in existing_requests {
        seen.insert(request.skill_id.clone());
    }

    let mut unique = Vec::new();
    for request in requests {
        if seen.insert(request.skill_id.clone()) {
            unique.push(request);
        }
    }
    unique
}

fn contains_any(text: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| contains_keyword(text, needle))
}

fn contains_keyword(text: &str, needle: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
}
