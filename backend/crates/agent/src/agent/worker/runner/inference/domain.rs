use super::{metrics::infer_platform, numbers::numbers_in_text, text::compact_task_text};
use crate::agent::worker::runner::policy::worker_policies;

pub(in crate::agent::worker::runner) fn infer_return_reason(text: &str) -> String {
    match_label(text, "service.return_reason", "不喜欢")
}

pub(in crate::agent::worker::runner) fn infer_service_issue_type(text: &str) -> String {
    match_label(text, "service.issue_type", "售后咨询")
}

pub(in crate::agent::worker::runner) fn infer_service_emotion(text: &str) -> String {
    match_label(text, "service.emotion", "普通")
}

pub(in crate::agent::worker::runner) fn infer_design_usage(text: &str) -> String {
    match_label(text, "design.usage", "主图套组")
}

pub(in crate::agent::worker::runner) fn infer_design_style(text: &str) -> String {
    match_label(text, "design.style", "简约现代")
}

pub(in crate::agent::worker::runner) fn infer_page_type(text: &str) -> String {
    match_label(text, "web.page_type", "首页")
}

pub(in crate::agent::worker::runner) fn infer_discount_hint(text: &str) -> String {
    match_label(text, "promotion.discount_hint", "限时优惠")
}

pub(in crate::agent::worker::runner) fn infer_video_type(text: &str) -> String {
    match_label(text, "creative.video_type", "种草")
}

pub(in crate::agent::worker::runner) fn infer_video_duration_seconds(text: &str) -> i64 {
    let numbers = numbers_in_text(text);
    if numbers.is_empty() {
        return 30;
    }
    if contains_any(
        text,
        &["秒", "sec", "second", "seconds", "时长", "duration"],
    ) {
        return numbers[0].round().clamp(10.0, 180.0) as i64;
    }
    if contains_any(text, &["分钟", "minute", "minutes"]) {
        return (numbers[0] * 60.0).round().clamp(10.0, 180.0) as i64;
    }
    30
}

pub(in crate::agent::worker::runner) fn infer_audience(text: &str) -> String {
    match_label(text, "audience", "目标消费者")
}

pub(in crate::agent::worker::runner) fn infer_category(text: &str) -> String {
    match_label(text, "category", "电商消费品")
}

pub(in crate::agent::worker::runner) fn infer_product_name(text: &str) -> String {
    compact_task_text(text)
}

pub(in crate::agent::worker::runner) fn infer_campaign_name(text: &str) -> String {
    match_label(text, "campaign.name", &compact_task_text(text))
}

pub(in crate::agent::worker::runner) fn infer_campaign_type(text: &str) -> String {
    match_label(text, "campaign_type", "general")
}

pub(in crate::agent::worker::runner) fn extract_labeled_token(
    text: &str,
    labels: &[&str],
) -> Option<String> {
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

pub(in crate::agent::worker::runner) fn infer_system_type(text: &str) -> String {
    match_label(text, "engineering.system_type", "电商平台")
}

pub(in crate::agent::worker::runner) fn infer_bug_type(text: &str) -> String {
    match_label(text, "engineering.bug_type", "数据异常")
}

pub(in crate::agent::worker::runner) fn infer_bug_frequency(text: &str) -> String {
    match_label(text, "engineering.bug_frequency", "偶发")
}

pub(in crate::agent::worker::runner) fn infer_perf_bottleneck(text: &str) -> String {
    match_label(text, "engineering.perf_bottleneck", "接口")
}

pub(in crate::agent::worker::runner) fn infer_stack_hint(text: &str) -> String {
    let mut parts = Vec::new();
    for stack in [
        "Rust", "Python", "FastAPI", "React", "Postgres", "MySQL", "Redis", "Docker", "Nginx",
    ] {
        if text.contains(stack)
            || text
                .to_ascii_lowercase()
                .contains(&stack.to_ascii_lowercase())
        {
            parts.push(stack.to_string());
        }
    }
    if parts.is_empty() {
        "未明确技术栈".to_string()
    } else {
        parts.join("/")
    }
}

pub(in crate::agent::worker::runner) fn infer_tech_domain(text: &str) -> String {
    match_label(text, "engineering.tech_domain", "Web框架")
}

pub(in crate::agent::worker::runner) fn infer_status_query_type(text: &str) -> String {
    match_label(text, "status.query_type", "system_health")
}

pub(in crate::agent::worker::runner) fn infer_platform_health_check_platform(text: &str) -> String {
    if !has_platform_health_check_intent(text) {
        return String::new();
    }
    let platform = infer_platform(text);
    if platform == "电商" {
        String::new()
    } else {
        platform
    }
}

pub(in crate::agent::worker::runner) fn infer_learning_topic(text: &str) -> String {
    match_label(text, "learning.topic", &compact_task_text(text))
        .trim_start_matches("请")
        .trim_start_matches("帮我")
        .trim_start_matches("给我")
        .trim()
        .to_string()
}

pub(in crate::agent::worker::runner) fn infer_learning_level(text: &str) -> String {
    match_label(text, "learning.level", "practitioner")
}

pub(in crate::agent::worker::runner) fn infer_learning_scenario(text: &str) -> String {
    match_label(text, "learning.scenario", "电商经营学习")
}

fn has_platform_health_check_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "淘宝",
            "天猫",
            "抖音",
            "千川",
            "京东",
            "拼多多",
            "pdd",
            "jd",
            "taobao",
            "tmall",
            "douyin",
            "tiktok",
            "api",
            "API",
            "平台",
        ],
    ) && contains_any(
        text,
        &[
            "接口健康",
            "健康检查",
            "连接状态",
            "连接配置",
            "连通性",
            "连通",
            "测试连接",
            "API健康",
            "api健康",
            "同步状态",
            "凭证",
            "密钥",
            "授权",
        ],
    )
}

fn match_label(text: &str, rule_set: &str, default: &str) -> String {
    worker_policies()
        .inference
        .match_label(text, rule_set)
        .unwrap_or(default)
        .to_string()
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

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}
