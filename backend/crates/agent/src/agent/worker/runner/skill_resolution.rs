fn skill_not_allowed_result(
    request_id: String,
    skill_id: String,
    runtime_role: &str,
) -> SkillExecutionResult {
    SkillExecutionResult {
        request_id,
        skill_id: SkillId::new(skill_id.clone()),
        status: SkillExecutionStatus::ValidationError,
        output: None,
        summary: None,
        metadata: serde_json::Map::new(),
        error: Some(format!(
            "skill '{skill_id}' is not allowed for role '{runtime_role}'"
        )),
    }
}

fn resolve_skill_requests(
    role: &RoleProfile,
    task: &Task,
    output: &str,
) -> Vec<ParsedSkillRequest> {
    let requests = parse_skill_requests(output);
    if !requests.is_empty() {
        return normalize_explicit_skill_requests_for_task(requests, task);
    }
    autosuggest_skill_requests(role, task, output)
}

fn normalize_explicit_skill_requests_for_task(
    requests: Vec<ParsedSkillRequest>,
    task: &Task,
) -> Vec<ParsedSkillRequest> {
    requests
        .into_iter()
        .filter_map(|mut request| {
            if should_defer_category_level_skill_request(&request, task) {
                return None;
            }
            normalize_parsed_skill_request(&mut request);
            Some(request)
        })
        .collect()
}

fn should_defer_category_level_skill_request(request: &ParsedSkillRequest, task: &Task) -> bool {
    first_selected_product(task).is_none()
        && is_category_level_task(&task.description)
        && is_category_level_deferred_skill(request.skill_id.as_str())
}

fn is_category_level_deferred_skill(skill: &str) -> bool {
    matches!(
        skill,
        "coordination_create_task"
            | "coordination_create_campaign"
            | "platform_update_price"
            | "platform_update_inventory"
            | "platform_sync_products"
            | "coordination_external_agent_call"
            | "engineering_system_check"
    )
}

fn normalize_parsed_skill_request(request: &mut ParsedSkillRequest) {
    if request.skill_id == "coordination_create_task" {
        normalize_coordination_create_task_input(&mut request.input);
    }
}

fn normalize_coordination_create_task_input(input: &mut serde_json::Value) {
    let Some(object) = input.as_object_mut() else {
        return;
    };

    if let Some(priority) = object.get("priority").cloned() {
        let normalized = match priority {
            serde_json::Value::Number(number) => number.as_i64().map(|value| value.clamp(0, 2)),
            serde_json::Value::String(text) => normalize_task_priority_text(&text),
            _ => None,
        };
        if let Some(value) = normalized {
            object.insert("priority".to_string(), serde_json::json!(value));
        } else {
            object.remove("priority");
        }
    }
}

fn normalize_task_priority_text(value: &str) -> Option<i64> {
    let text = value.trim().to_ascii_lowercase();
    match text.as_str() {
        "0" | "normal" | "low" | "普通" | "低" => Some(0),
        "1" | "medium" | "important" | "重要" | "中" => Some(1),
        "2" | "high" | "urgent" | "紧急" | "高" => Some(2),
        _ => value
            .trim()
            .parse::<i64>()
            .ok()
            .map(|value| value.clamp(0, 2)),
    }
}

