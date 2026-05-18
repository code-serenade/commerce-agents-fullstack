fn infer_memory_key(text: &str) -> String {
    let compact = compact_task_text(text);
    let mut key = compact
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    while key.contains("__") {
        key = key.replace("__", "_");
    }
    key = key.trim_matches('_').to_string();
    if key.is_empty() {
        "workspace_memory".to_string()
    } else {
        key.chars().take(80).collect()
    }
}

fn infer_memory_type(text: &str) -> &'static str {
    if contains_any(text, &["风险", "warning", "隐患"]) {
        "warning"
    } else if contains_any(text, &["决策", "决定", "decision"]) {
        "decision"
    } else if contains_any(text, &["洞察", "发现", "insight"]) {
        "insight"
    } else {
        "fact"
    }
}

fn infer_memory_content(text: &str, task: &Task) -> String {
    let compact = compact_task_text(text);
    if compact.chars().count() >= 24 {
        return compact.chars().take(500).collect();
    }
    compact_task_text(&task.description)
        .chars()
        .take(500)
        .collect()
}

