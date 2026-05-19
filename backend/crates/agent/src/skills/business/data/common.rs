use serde_json::{Value, json};

pub(super) use super::dashboard::empty_string_as_null;
use crate::skills::SkillError;

pub(super) fn value_string(input: &Value, key: &str, default: &str) -> String {
    input
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_owned()
}

pub(super) fn value_pointer_string<'a>(input: &'a Value, path: &str, default: &'a str) -> &'a str {
    input
        .pointer(path)
        .and_then(Value::as_str)
        .unwrap_or(default)
}

pub(super) fn value_pointer_number(input: &Value, path: &str) -> Option<f64> {
    input.pointer(path).and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_i64().map(|number| number as f64))
    })
}

pub(super) fn value_to_short_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => value.to_string(),
    }
}

pub(super) fn number_from_any(input: &Value, keys: &[&str], default: f64) -> (f64, bool) {
    for key in keys {
        if let Some(value) = input.get(*key).and_then(Value::as_f64) {
            return (value.max(0.0), true);
        }
        if let Some(value) = input.get(*key).and_then(Value::as_i64) {
            return ((value as f64).max(0.0), true);
        }
    }
    (default.max(0.0), false)
}

#[derive(Debug, Clone)]
pub(super) struct AttributionChannel {
    pub(super) name: String,
    pub(super) gmv: f64,
    pub(super) uv: f64,
    pub(super) conversion_rate: f64,
    pub(super) ad_spend: f64,
}

pub(super) fn normalized_rate(value: f64) -> f64 {
    if value > 1.0 {
        value / 100.0
    } else {
        value.max(0.0)
    }
}

pub(super) fn normalize(values: Vec<f64>) -> Vec<f64> {
    let total = values.iter().sum::<f64>();
    if total.abs() < 1e-9 {
        let len = values.len().max(1);
        vec![1.0 / len as f64; len]
    } else {
        values.into_iter().map(|value| value / total).collect()
    }
}

pub(super) fn one_hot(len: usize, active_idx: usize) -> Vec<f64> {
    (0 .. len)
        .map(|idx| if idx == active_idx { 1.0 } else { 0.0 })
        .collect()
}

pub(super) fn position_based_weights(len: usize) -> Vec<f64> {
    match len {
        0 => Vec::new(),
        1 => vec![1.0],
        2 => vec![0.5, 0.5],
        3 => vec![0.4, 0.2, 0.4],
        _ => {
            let middle = 0.20 / (len - 2) as f64;
            let mut weights = Vec::with_capacity(len);
            weights.push(0.4);
            weights.extend(std::iter::repeat_n(middle, len - 2));
            weights.push(0.4);
            normalize(weights)
        }
    }
}

pub(super) fn attribution_models_to_show(model: &str) -> Vec<&'static str> {
    match model {
        "last_touch" => vec!["last_touch"],
        "linear" => vec!["linear"],
        "time_decay" => vec!["time_decay"],
        "position" => vec!["position"],
        _ => vec!["last_touch", "linear", "time_decay", "position"],
    }
}

pub(super) fn pct_delta(current: f64, previous: f64) -> f64 {
    pct(current - previous, previous.max(1.0))
}

pub(super) fn metric_card(label: &str, value: f64, unit: &str, delta: f64, note: &str) -> Value {
    json!({
        "指标": label,
        "数值": value,
        "单位": unit,
        "变化": signed_percent(delta, 2),
        "状态": if delta >= 0.0 { "改善" } else if delta > -5.0 { "轻微下滑" } else { "需关注" },
        "说明": note
    })
}

pub(super) fn dashboard_health_score(
    gmv_growth: f64,
    conversion_rate: f64,
    ad_roi: f64,
    refund_rate: f64,
    gross_margin: f64,
    target_achievement: f64,
) -> i64 {
    let score = 50.0
        + gmv_growth.clamp(-20.0, 20.0) * 0.6
        + (conversion_rate - 3.0).clamp(-3.0, 8.0) * 2.0
        + (ad_roi - 2.0).clamp(-2.0, 6.0) * 3.0
        + (gross_margin - 25.0).clamp(-20.0, 30.0) * 0.5
        + (target_achievement - 80.0).clamp(-40.0, 40.0) * 0.2
        - (refund_rate - 4.0).clamp(0.0, 12.0) * 2.0;
    score.round().clamp(0.0, 100.0) as i64
}

pub(super) fn growth_summary(
    gmv_growth: f64,
    order_growth: f64,
    visitor_growth: f64,
) -> &'static str {
    if gmv_growth >= 5.0 && order_growth >= 3.0 {
        "收入和成交同步增长，可继续放大有效渠道"
    } else if visitor_growth >= 5.0 && order_growth < 0.0 {
        "流量增长但成交未跟上，重点排查转化承接"
    } else if gmv_growth < -5.0 {
        "收入动能走弱，优先定位流量、订单和客单价变化"
    } else {
        "增长基本平稳，建议持续观察渠道结构"
    }
}

pub(super) fn efficiency_summary(conversion_rate: f64, ad_roi: f64, aov: f64) -> String {
    if conversion_rate < 2.0 {
        "转化效率偏弱，先优化详情页和首屏卖点".to_string()
    } else if ad_roi < 2.0 {
        "投放效率偏弱，先收缩低ROI计划".to_string()
    } else {
        format!("效率稳定，当前客单价约{}", round2(aov))
    }
}

pub(super) fn risk_summary(refund_rate: f64, gross_margin: f64) -> &'static str {
    if refund_rate > 6.0 {
        "售后风险偏高，优先拆解退款原因"
    } else if gross_margin < 20.0 {
        "利润安全垫不足，需复核成本和优惠"
    } else {
        "退款和毛利暂未触发高风险阈值"
    }
}

pub(super) fn dashboard_actions(
    gmv_growth: f64,
    conversion_rate: f64,
    ad_roi: f64,
    refund_rate: f64,
    gross_margin: f64,
) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if gmv_growth < -5.0 {
        actions.push("拆分GMV下滑来源：流量、转化、客单价、退款");
    }
    if conversion_rate < 2.0 {
        actions.push("复核详情页首屏、价格锚点、评价和客服响应");
    }
    if ad_roi < 2.0 {
        actions.push("暂停低ROI计划，保留高转化素材继续A/B测试");
    }
    if refund_rate > 6.0 {
        actions.push("按退款原因做帕累托拆解，优先处理累计80%问题项");
    }
    if gross_margin < 20.0 {
        actions.push("复核进货成本、平台扣点、优惠券和包邮策略");
    }
    if actions.is_empty() {
        actions.push("保持当前节奏，下一轮重点观察渠道结构和复购");
    }
    actions
}

pub(super) fn json_ok(value: Value) -> Result<Value, SkillError> {
    Ok(value)
}

pub(super) fn required_string(input: &Value, key: &str) -> Result<String, SkillError> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| SkillError::missing_parameter(key))
}

pub(super) fn required_i(input: &Value, key: &str) -> Result<i64, SkillError> {
    input
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| SkillError::missing_parameter(key))
}

pub(super) fn s(input: &Value, key: &str, default: &str) -> String {
    input
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_owned()
}

pub(super) fn i(input: &Value, key: &str, default: i64) -> i64 {
    input.get(key).and_then(Value::as_i64).unwrap_or(default)
}

pub(super) fn n(input: &Value, key: &str, default: f64) -> f64 {
    input.get(key).and_then(Value::as_f64).unwrap_or(default)
}

pub(super) fn nums(input: &Value, key: &str) -> Vec<f64> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

pub(super) fn has_any_input(input: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        input.get(*key).is_some_and(|value| match value {
            Value::Null => false,
            Value::String(value) => !value.trim().is_empty(),
            Value::Array(values) => !values.is_empty(),
            Value::Object(values) => !values.is_empty(),
            _ => true,
        })
    })
}

pub(super) fn input_array_missing_or_empty(input: &Value, key: &str) -> bool {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::is_empty)
        .unwrap_or(true)
}

pub(super) fn ints(input: &Value, key: &str, default: Vec<i64>) -> Vec<i64> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_i64).collect())
        .filter(|values: &Vec<i64>| !values.is_empty())
        .unwrap_or(default)
}

pub(super) fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

pub(super) fn stddev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = average(values);
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt()
}

pub(super) fn linear_fit(values: &[f64]) -> (f64, f64) {
    let len = values.len() as f64;
    let x_mean = (len - 1.0) / 2.0;
    let y_mean = average(values);
    let numerator = values
        .iter()
        .enumerate()
        .map(|(idx, value)| (idx as f64 - x_mean) * (value - y_mean))
        .sum::<f64>();
    let denominator = (0 .. values.len())
        .map(|idx| (idx as f64 - x_mean).powi(2))
        .sum::<f64>();
    let slope = if denominator.abs() > 1e-9 {
        numerator / denominator
    } else {
        0.0
    };
    (slope, y_mean - slope * x_mean)
}

pub(super) fn moving_average(values: &[f64], window: usize) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    (0 .. values.len())
        .map(|idx| {
            let start = (idx + 1).saturating_sub(window);
            round2(average(&values[start ..= idx]))
        })
        .collect()
}

pub(super) fn avg_growth(values: &[f64]) -> f64 {
    let rates = values
        .windows(2)
        .filter_map(|pair| {
            (pair[0].abs() > 1e-9).then_some((pair[1] - pair[0]) / pair[0].abs() * 100.0)
        })
        .collect::<Vec<_>>();
    average(&rates)
}

pub(super) fn ses_level(values: &[f64], alpha: f64) -> f64 {
    let mut level = values[0];
    for value in &values[1 ..] {
        level = alpha * *value + (1.0 - alpha) * level;
    }
    level
}

pub(super) fn ses_fitted(values: &[f64], alpha: f64) -> Vec<f64> {
    let mut level = values[0];
    let mut fitted = Vec::with_capacity(values.len().saturating_sub(1));
    for value in &values[1 ..] {
        fitted.push(level);
        level = alpha * *value + (1.0 - alpha) * level;
    }
    fitted
}

pub(super) fn mape(actual: &[f64], predicted: &[f64]) -> f64 {
    let errors = actual
        .iter()
        .zip(predicted)
        .filter_map(|(actual, predicted)| {
            (actual.abs() > 1e-9).then_some(((actual - predicted) / actual).abs() * 100.0)
        })
        .collect::<Vec<_>>();
    average(&errors)
}

pub(super) fn normal_confidence(z_abs: f64) -> f64 {
    if z_abs >= 2.576 {
        0.99
    } else if z_abs >= 1.96 {
        0.95
    } else if z_abs >= 1.645 {
        0.90
    } else {
        (0.5 + z_abs / 3.29).min(0.89)
    }
}

pub(super) fn metric_display(metric: &str) -> &str {
    match metric {
        "gmv" => "GMV",
        "orders" => "订单数",
        "uv" => "访客数(UV)",
        "conversion_rate" => "转化率",
        "ad_spend" => "广告花费",
        _ => metric,
    }
}

pub(super) fn pct(num: f64, den: f64) -> f64 {
    if den.abs() < 1e-9 {
        0.0
    } else {
        num / den * 100.0
    }
}

pub(super) fn percent(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}%")
}

pub(super) fn signed_percent(value: f64, decimals: usize) -> String {
    format!("{value:+.decimals$}%")
}

pub(super) fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub(super) fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}
