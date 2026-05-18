fn infer_audience(text: &str) -> &'static str {
    if contains_any(text, &["student", "学生"]) {
        "学生用户"
    } else if contains_any(text, &["office", "commute", "通勤", "白领", "上班"]) {
        "通勤白领"
    } else if contains_any(text, &["mom", "妈妈", "宝妈", "家庭"]) {
        "家庭消费人群"
    } else if contains_any(text, &["young", "年轻", "女生", "女性"]) {
        "年轻女性"
    } else {
        "目标消费者"
    }
}

fn infer_category(text: &str) -> &'static str {
    policy_match_label(text, "category").unwrap_or("电商消费品")
}

fn infer_product_name(text: &str) -> String {
    compact_task_text(text)
}

fn infer_campaign_name(text: &str) -> String {
    if contains_any(text, &["618"]) {
        "618大促".to_string()
    } else if contains_any(text, &["双11", "双十一"]) {
        "双11大促".to_string()
    } else if contains_any(text, &["五一"]) {
        "五一活动".to_string()
    } else {
        compact_task_text(text)
    }
}

fn infer_campaign_type(text: &str) -> &'static str {
    policy_match_label(text, "campaign_type").unwrap_or("general")
}

fn extract_labeled_token(text: &str, labels: &[&str]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    for label in labels {
        let label_lower = label.to_ascii_lowercase();
        let Some(index) = lower.find(&label_lower) else {
            continue;
        };
        let start = index + label_lower.len();
        if let Some(token) = parse_token_after_label(&text[start ..]) {
            return Some(token);
        }
    }
    None
}

fn parse_token_after_label(segment: &str) -> Option<String> {
    let mut chars = segment.char_indices().peekable();
    let mut token_start = None;
    while let Some((index, ch)) = chars.peek().copied() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            token_start = Some(index);
            break;
        }
        if matches!(ch, ':' | '：' | '=' | '＝' | ' ' | '\t' | '#' | '号') {
            chars.next();
            continue;
        }
        if index > 12 {
            return None;
        }
        chars.next();
    }

    let start = token_start?;
    let mut end = start;
    for (index, ch) in segment[start ..].char_indices() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            end = start + index + ch.len_utf8();
            continue;
        }
        break;
    }
    let token = segment[start .. end].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}
