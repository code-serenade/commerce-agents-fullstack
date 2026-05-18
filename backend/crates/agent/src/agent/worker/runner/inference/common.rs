fn autosuggest_request(
    skill_id: impl Into<String>,
    input: serde_json::Value,
    reason: impl Into<String>,
) -> ParsedSkillRequest {
    ParsedSkillRequest {
        skill_id: skill_id.into(),
        input: compact_skill_input(input),
        reason: Some(reason.into()),
    }
}

fn compact_skill_input(input: serde_json::Value) -> serde_json::Value {
    match input {
        serde_json::Value::Object(values) => serde_json::Value::Object(
            values
                .into_iter()
                .filter_map(|(key, value)| {
                    compact_skill_input_field(value).map(|value| (key, value))
                })
                .collect(),
        ),
        value => value,
    }
}

fn compact_skill_input_field(value: serde_json::Value) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Array(values) => {
            let values = values
                .into_iter()
                .filter_map(compact_skill_input_field)
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(serde_json::Value::Array(values))
            }
        }
        serde_json::Value::Object(values) => {
            let values = values
                .into_iter()
                .filter_map(|(key, value)| {
                    compact_skill_input_field(value).map(|value| (key, value))
                })
                .collect();
            Some(serde_json::Value::Object(values))
        }
        value => Some(value),
    }
}

fn autosuggest_text(task: &Task, output: &str) -> String {
    let mut parts = vec![task.description.clone(), output.to_string()];
    for key in [
        "context.summary",
        "context.source_titles",
        "rag.summary",
        "rag.source_titles",
    ] {
        if let Some(value) = task.context.metadata.get(key) {
            parts.push(value.clone());
        }
    }
    if let Some(assignment) = &task.assignment {
        parts.push(assignment.objective.clone());
        parts.extend(assignment.inputs.iter().map(|input| input.content.clone()));
    }
    parts.join("\n")
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}

fn numbers_in_text(text: &str) -> Vec<f64> {
    let mut values = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
            continue;
        }
        push_parsed_number(&mut values, &mut current);
    }
    push_parsed_number(&mut values, &mut current);
    values
}

