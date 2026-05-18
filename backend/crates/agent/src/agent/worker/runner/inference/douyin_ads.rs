use super::{
    context,
    metrics::{extract_metric_value, infer_stage},
    numbers::infer_explicit_period_days,
    text::compact_task_text,
};
use crate::agent::worker::{Task, runner::policy::worker_policies};

pub(in crate::agent::worker::runner) fn infer_goal(text: &str, task: &Task) -> String {
    for rule in config_array("/goal/rules") {
        let Some(label) = rule.get("label").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let keywords = string_list_from_value(rule.get("keywords"));
        if contains_any(text, &keywords) {
            return label.to_string();
        }
    }
    compact_task_text(&task.description)
}

pub(in crate::agent::worker::runner) fn infer_days(text: &str) -> i64 {
    infer_explicit_period_days(text).unwrap_or_else(|| config_i64("/days/default", 7))
}

pub(in crate::agent::worker::runner) fn infer_product_name(text: &str, task: &Task) -> String {
    if context::first_selected_product(task).is_some()
        || contains_any(
            text,
            &config_string_list(
                "/product_name/intent_keywords",
                &["商品", "产品", "咖啡", "SKU", "sku"],
            ),
        )
    {
        context::selected_product_name_or_task(task)
    } else {
        config_string("/product_name/fallback_when_no_intent", "")
    }
}

pub(in crate::agent::worker::runner) fn infer_budget(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &metric_aliases(
            "budget",
            &[
                "投放预算",
                "广告预算",
                "千川预算",
                "总预算",
                "人群预算",
                "预算",
                "budget",
            ],
        ),
    )
}

pub(in crate::agent::worker::runner) fn infer_target_roas(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &metric_aliases(
            "target_roas",
            &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
        ),
    )
}

pub(in crate::agent::worker::runner) fn infer_spend(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &metric_aliases(
            "spend",
            &[
                "已消耗",
                "消耗",
                "花费",
                "广告费",
                "广告花费",
                "spend",
                "ad_spend",
            ],
        ),
    )
}

pub(in crate::agent::worker::runner) fn infer_gmv(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &metric_aliases("gmv", &["GMV", "gmv", "成交额", "销售额", "营收"]),
    )
}

pub(in crate::agent::worker::runner) fn infer_current_roas(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &config_string_list(
            "/metrics/current_roas/primary_aliases",
            &[
                "当前ROAS",
                "当前roas",
                "实际ROAS",
                "实际roas",
                "当前投产",
                "实际投产",
                "当前ROI",
                "当前roi",
                "实际ROI",
                "实际roi",
            ],
        ),
    )
    .or_else(|| {
        if contains_any(
            text,
            &config_string_list(
                "/metrics/current_roas/blocked_when_target_alias_present",
                &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
            ),
        ) {
            None
        } else {
            extract_metric_value(
                text,
                &config_string_list(
                    "/metrics/current_roas/fallback_aliases",
                    &["ROAS", "roas", "投产", "ROI", "roi"],
                ),
            )
        }
    })
}

pub(in crate::agent::worker::runner) fn infer_live_metric(text: &str, key: &str) -> Option<f64> {
    match key {
        "ad_spend" => infer_spend(text),
        "gmv" => infer_gmv(text),
        _ => {
            let aliases = config_string_list(&format!("/live_metrics/{key}/aliases"), &[]);
            (!aliases.is_empty())
                .then(|| extract_metric_value(text, &aliases))
                .flatten()
        }
    }
}

pub(in crate::agent::worker::runner) fn infer_stage_for_ads(text: &str) -> String {
    infer_stage(text)
}

fn contains_any<T: AsRef<str>>(text: &str, needles: &[T]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles.iter().any(|needle| {
        let needle = needle.as_ref();
        text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
    })
}

fn config_array(pointer: &str) -> Vec<serde_json::Value> {
    worker_policies()
        .inference
        .section("douyin_ads")
        .and_then(|value| value.pointer(pointer))
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn config_i64(pointer: &str, fallback: i64) -> i64 {
    worker_policies()
        .inference
        .i64_or("douyin_ads", pointer, fallback)
}

fn config_string(pointer: &str, fallback: &str) -> String {
    worker_policies()
        .inference
        .string_or("douyin_ads", pointer, fallback)
}

fn config_string_list(pointer: &str, fallback: &[&str]) -> Vec<String> {
    worker_policies()
        .inference
        .string_list_or("douyin_ads", pointer, fallback)
}

fn metric_aliases(key: &str, fallback: &[&str]) -> Vec<String> {
    config_string_list(&format!("/metrics/{key}/aliases"), fallback)
}

fn string_list_from_value(value: Option<&serde_json::Value>) -> Vec<&str> {
    value
        .and_then(serde_json::Value::as_array)
        .map(|items| items.iter().filter_map(serde_json::Value::as_str).collect())
        .unwrap_or_default()
}
