fn infer_task_title(text: &str, task: &Task) -> String {
    extract_title_candidate(text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| compact_task_text(&task.description))
}

fn infer_task_owner_role(text: &str) -> &'static str {
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

fn infer_task_priority(text: &str) -> i64 {
    if contains_any(text, &["紧急", "立刻", "马上", "高优", "urgent", "asap"]) {
        8
    } else if contains_any(text, &["重要", "优先", "priority"]) {
        6
    } else {
        3
    }
}

fn infer_task_acceptance(text: &str) -> String {
    if let Some(index) = text.find("验收") {
        return compact_task_text(&text[index ..]);
    }
    if let Some(index) = text.find("目标") {
        return compact_task_text(&text[index ..]);
    }
    "产出明确结果并可在工作区复用".to_string()
}

