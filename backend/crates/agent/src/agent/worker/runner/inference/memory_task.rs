use super::text::compact_task_text;
use crate::agent::worker::Task;

pub(in crate::agent::worker::runner) fn infer_memory_key(text: &str) -> String {
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

pub(in crate::agent::worker::runner) fn infer_memory_type(text: &str) -> &'static str {
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

pub(in crate::agent::worker::runner) fn infer_memory_content(text: &str, task: &Task) -> String {
    let compact = compact_task_text(text);
    if compact.chars().count() >= 24 {
        return compact.chars().take(500).collect();
    }
    compact_task_text(&task.description)
        .chars()
        .take(500)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_task_title(text: &str, task: &Task) -> String {
    extract_title_candidate(text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| compact_task_text(&task.description))
}

pub(in crate::agent::worker::runner) fn infer_task_owner_role(text: &str) -> &'static str {
    if contains_any(text, &["数据", "data", "分析"]) {
        "data"
    } else if contains_any(text, &["客服", "售后", "service"]) {
        "service"
    } else if contains_any(text, &["设计", "主图", "design"]) {
        "design"
    } else if contains_any(text, &["财务", "roi", "accounting"]) {
        "accounting"
    } else if contains_any(text, &["技术", "工程", "engineering", "开发"]) {
        "engineering"
    } else if contains_any(text, &["seo", "店铺", "web"]) {
        "web"
    } else if contains_any(text, &["创意", "视频", "creative", "文案"]) {
        "creative"
    } else {
        "ops"
    }
}

pub(in crate::agent::worker::runner) fn infer_task_priority(text: &str) -> i64 {
    if contains_any(text, &["紧急", "立刻", "马上", "高优", "urgent", "asap"]) {
        8
    } else if contains_any(text, &["重要", "优先", "priority"]) {
        6
    } else {
        3
    }
}

pub(in crate::agent::worker::runner) fn infer_task_acceptance(text: &str) -> String {
    if let Some(index) = text.find("验收") {
        return compact_task_text(&text[index ..]);
    }
    if let Some(index) = text.find("目标") {
        return compact_task_text(&text[index ..]);
    }
    "产出明确结果并可在工作区复用".to_string()
}

fn extract_title_candidate(text: &str) -> Option<String> {
    for quote in ['"', '\'', '“', '”', '「', '」'] {
        let mut parts = text.split(quote);
        let _ = parts.next();
        if let Some(candidate) = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(compact_task_text(candidate));
        }
    }
    None
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}
