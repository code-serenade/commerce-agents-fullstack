fn infer_page_type(text: &str) -> &'static str {
    if text.contains("详情") || text.to_ascii_lowercase().contains("detail") {
        "商品详情"
    } else if text.contains("购物车") || text.to_ascii_lowercase().contains("cart") {
        "购物车"
    } else if text.contains("结算") || text.to_ascii_lowercase().contains("checkout") {
        "结算"
    } else if text.contains("列表") || text.to_ascii_lowercase().contains("listing") {
        "列表页"
    } else {
        "首页"
    }
}

fn infer_design_usage(text: &str) -> &'static str {
    policy_match_label(text, "design.usage").unwrap_or("主图套组")
}

fn infer_design_style(text: &str) -> &'static str {
    policy_match_label(text, "design.style").unwrap_or("简约现代")
}

fn extract_discount_hint(text: &str) -> &'static str {
    if contains_any(text, &["满减", "满"]) {
        "满减优惠"
    } else if contains_any(text, &["折", "折扣"]) {
        "限时折扣"
    } else if contains_any(text, &["赠", "赠品"]) {
        "下单赠礼"
    } else {
        "限时优惠"
    }
}

fn infer_video_type(text: &str) -> &'static str {
    if contains_any(text, &["review", "测评"]) {
        "测评"
    } else if contains_any(text, &["tutorial", "教程", "教学", "怎么用"]) {
        "教程"
    } else if contains_any(text, &["plot", "story", "剧情"]) {
        "剧情"
    } else if contains_any(text, &["unbox", "开箱"]) {
        "开箱"
    } else {
        "种草"
    }
}

fn infer_video_duration_seconds(text: &str) -> i64 {
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
