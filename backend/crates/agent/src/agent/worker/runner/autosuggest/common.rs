fn autosuggest_skill_requests(
    role: &RoleProfile,
    task: &Task,
    output: &str,
) -> Vec<ParsedSkillRequest> {
    let text = autosuggest_text(task, output);
    if let Some(requests) = configured_role_requests(role.runtime_role.as_str(), &text, task) {
        return requests;
    }
    let Some(request) = autosuggest_for_role(role.runtime_role.as_str(), &text, task) else {
        return Vec::new();
    };
    vec![request]
}

fn autosuggest_for_role(runtime_role: &str, text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    match runtime_role {
        "chat" => autosuggest_chat(text, task),
        "ops" => autosuggest_ops(text, task),
        "data" => autosuggest_data(text, task),
        "accounting" => autosuggest_accounting(text, task),
        "service" => autosuggest_service(text, task),
        "creative" => autosuggest_creative(text, task),
        "web" => autosuggest_web(text, task),
        "design" => autosuggest_design(text, task),
        "douyin_ads" => autosuggest_douyin_ads(text, task),
        "engineering" => autosuggest_engineering(text, task),
        _ => None,
    }
}

fn autosuggest_learning(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if !has_learning_teaching_intent(text) {
        return None;
    }

    let mut input = serde_json::Map::new();
    input.insert(
        "topic".to_string(),
        serde_json::json!(infer_learning_topic(text)),
    );
    input.insert(
        "learning_level".to_string(),
        serde_json::json!(infer_learning_level_from_text(text)),
    );
    input.insert(
        "scenario".to_string(),
        serde_json::json!(infer_learning_scenario(text)),
    );
    input.insert(
        "include_self_check".to_string(),
        serde_json::json!(!is_learning_self_check_disabled(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
        if let Some(product) = first_selected_product(task) {
            input.insert("product_name".to_string(), serde_json::json!(product.name));
            if let Some(platform) = product.platform {
                input.insert("platform".to_string(), serde_json::json!(platform));
            }
        }
    }

    Some(autosuggest_request(
        learning_skill_id_from_policy(),
        serde_json::Value::Object(input),
        "根据教学/解释/概念学习意图自动补偿教学解释 skill",
    ))
}

fn has_learning_teaching_intent(text: &str) -> bool {
    has_learning_teaching_intent_from_policy(text)
}
