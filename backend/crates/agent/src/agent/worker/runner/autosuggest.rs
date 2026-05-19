pub(super) mod config;
mod rules;

use config::WorkerAutosuggestPolicy;
pub(super) use config::load_autosuggest_policy;
use rules::{configured_rule_requests, contains_any, render_template_value};

use super::{inference::inference_text, policy::worker_policies, types::ParsedSkillRequest};
use crate::agent::worker::Task;

pub(super) fn autosuggest_skill_requests(
    role_runtime: &str,
    task: &Task,
    existing_requests: &[ParsedSkillRequest],
) -> Vec<ParsedSkillRequest> {
    let policy = &worker_policies().autosuggest;
    let text = inference_text(task);
    let Some(route) = policy
        .role_routes
        .iter()
        .find(|route| route.role == role_runtime)
    else {
        return Vec::new();
    };

    let mut requests = Vec::new();
    if route.learning_first
        && let Some(request) = learning_request(policy, &text, task)
    {
        requests.push(request);
        if route.mode != "requests" {
            return dedupe_new_requests(requests, existing_requests);
        }
    }

    match route.handler.as_str() {
        "chat" => extend_optional(&mut requests, chat_request(policy, &text, task)),
        "search" => requests.extend(configured_rule_requests(
            policy,
            role_runtime,
            &text,
            task,
            &route.mode,
        )),
        _ => requests.extend(configured_rule_requests(
            policy,
            role_runtime,
            &text,
            task,
            &route.mode,
        )),
    }

    if route.handler != "search" && requests.is_empty() {
        let fallback_mode = if route.mode == "requests" {
            "requests"
        } else {
            "single"
        };
        requests.extend(configured_rule_requests(
            policy,
            "search",
            &text,
            task,
            fallback_mode,
        ));
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
            input: render_template_value(&policy.chat.save_memory_input, text, task),
            reason: Some("autosuggest:chat.save_memory".to_string()),
        });
    }
    if contains_any(text, &policy.chat.status_query_keywords) {
        return Some(ParsedSkillRequest {
            skill_id: policy.chat.status_query_skill_id.clone(),
            input: render_template_value(&policy.chat.status_query_input, text, task),
            reason: Some("autosuggest:chat.status_query".to_string()),
        });
    }
    learning_request(policy, text, task)
}

fn learning_request(
    policy: &WorkerAutosuggestPolicy,
    text: &str,
    task: &Task,
) -> Option<ParsedSkillRequest> {
    let has_explicit_keyword = contains_any(text, &policy.learning.explicit_keywords);
    let has_question_with_explanation = contains_any(text, &policy.learning.question_keywords)
        && contains_any(text, &policy.learning.explanation_keywords);
    if !has_explicit_keyword && !has_question_with_explanation {
        return None;
    }

    Some(ParsedSkillRequest {
        skill_id: policy.learning.skill_id.clone(),
        input: render_learning_input(policy, text, task),
        reason: Some("autosuggest:learning".to_string()),
    })
}

fn render_learning_input(
    policy: &WorkerAutosuggestPolicy,
    text: &str,
    task: &Task,
) -> serde_json::Value {
    let mut input = render_template_value(&policy.learning.input, text, task);
    if let Some(object) = input.as_object_mut() {
        object.insert(
            "include_self_check".to_string(),
            serde_json::json!(!contains_any(
                text,
                &policy.learning.self_check_disabled_keywords
            )),
        );
    }
    input
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
