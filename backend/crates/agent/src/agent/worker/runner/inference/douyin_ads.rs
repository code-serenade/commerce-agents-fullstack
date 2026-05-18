use super::{
    context,
    metrics::{extract_metric_value, infer_stage},
    numbers::infer_explicit_period_days,
    text::compact_task_text,
};
use crate::agent::worker::Task;

pub(in crate::agent::worker::runner) fn infer_goal(text: &str, task: &Task) -> String {
    if contains_any(text, &["直播", "直播间", "进房", "场观"]) {
        "直播间成交与进房转化".to_string()
    } else if contains_any(text, &["短视频", "视频", "素材"]) {
        "短视频引流与商品转化".to_string()
    } else if contains_any(text, &["新品", "冷启动", "起量"]) {
        "新品冷启动起量".to_string()
    } else {
        compact_task_text(&task.description)
    }
}

pub(in crate::agent::worker::runner) fn infer_days(text: &str) -> i64 {
    infer_explicit_period_days(text).unwrap_or(7)
}

pub(in crate::agent::worker::runner) fn infer_product_name(text: &str, task: &Task) -> String {
    if context::first_selected_product(task).is_some()
        || contains_any(text, &["商品", "产品", "咖啡", "SKU", "sku"])
    {
        context::selected_product_name_or_task(task)
    } else {
        String::new()
    }
}

pub(in crate::agent::worker::runner) fn infer_budget(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &[
            "投放预算",
            "广告预算",
            "千川预算",
            "总预算",
            "人群预算",
            "预算",
            "budget",
        ],
    )
}

pub(in crate::agent::worker::runner) fn infer_target_roas(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
    )
}

pub(in crate::agent::worker::runner) fn infer_spend(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &[
            "已消耗",
            "消耗",
            "花费",
            "广告费",
            "广告花费",
            "spend",
            "ad_spend",
        ],
    )
}

pub(in crate::agent::worker::runner) fn infer_gmv(text: &str) -> Option<f64> {
    extract_metric_value(text, &["GMV", "gmv", "成交额", "销售额", "营收"])
}

pub(in crate::agent::worker::runner) fn infer_current_roas(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
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
    )
    .or_else(|| {
        if contains_any(
            text,
            &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
        ) {
            None
        } else {
            extract_metric_value(text, &["ROAS", "roas", "投产", "ROI", "roi"])
        }
    })
}

pub(in crate::agent::worker::runner) fn infer_live_metric(text: &str, key: &str) -> Option<f64> {
    match key {
        "impressions" => extract_metric_value(
            text,
            &["曝光量", "曝光", "展现", "展示", "impressions", "exposure"],
        ),
        "clicks" => extract_metric_value(text, &["点击数", "点击", "广告点击", "clicks", "click"]),
        "live_room_entries" => extract_metric_value(
            text,
            &["进房数", "进房", "进入直播间", "场观", "live_room_entries"],
        ),
        "product_clicks" => extract_metric_value(
            text,
            &["商品点击数", "商品点击", "点击商品", "product_clicks"],
        ),
        "orders" => extract_metric_value(text, &["下单数", "下单", "订单数", "订单", "orders"]),
        "payments" => extract_metric_value(text, &["付款数", "付款", "支付数", "支付", "payments"]),
        "ad_spend" => infer_spend(text),
        "gmv" => infer_gmv(text),
        _ => None,
    }
}

pub(in crate::agent::worker::runner) fn infer_stage_for_ads(text: &str) -> String {
    infer_stage(text)
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}
