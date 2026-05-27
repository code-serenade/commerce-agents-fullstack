use serde_json::{Value, json};

use crate::skills::{
    SkillCategory, SkillError, SkillInputField, SkillOutcome, SkillPriority, SkillSpec,
};

pub(super) fn spec(
    name: &str,
    display_name: &str,
    description: &str,
    input_fields: Vec<SkillInputField>,
) -> SkillSpec {
    let mut spec = SkillSpec::new(name, display_name, description)
        .with_category(SkillCategory::Custom("ops".to_owned()))
        .with_priority(SkillPriority::Critical)
        .with_tag("deterministic")
        .with_tag("p0")
        .with_version("0.1.0");
    for input in input_fields {
        spec = spec.with_input(input);
    }
    spec
}

pub(super) fn field(
    name: &str,
    field_type: &str,
    description: &str,
    required: bool,
) -> SkillInputField {
    SkillInputField::new(name, description, required, field_schema(field_type))
}

pub(super) fn outcome(value: Value) -> SkillOutcome {
    SkillOutcome::new(value)
}

pub(super) fn invalid_input(message: String) -> SkillError {
    SkillError::execution(message)
}

fn field_schema(field_type: &str) -> Value {
    match field_type {
        "string" => json!({"type": "string"}),
        "number" => json!({"type": "number"}),
        "integer" => json!({"type": "integer"}),
        "object" => json!({"type": "object"}),
        "array<number>" => json!({"type": "array", "items": {"type": "number"}}),
        "array<string>" => json!({"type": "array", "items": {"type": "string"}}),
        "array<object>" => json!({"type": "array", "items": {"type": "object"}}),
        _ => json!({"type": "string"}),
    }
}

pub(super) fn num(input: &Value, key: &str, default: f64) -> f64 {
    val_num(input, key, default)
}

pub(super) fn val_num(value: &Value, key: &str, default: f64) -> f64 {
    value[key]
        .as_f64()
        .or_else(|| value[key].as_i64().map(|v| v as f64))
        .or_else(|| value[key].as_u64().map(|v| v as f64))
        .unwrap_or(default)
}

pub(super) fn nums(input: &Value, key: &str) -> Vec<f64> {
    value_nums(&input[key])
}

pub(super) fn strings(input: &Value, key: &str) -> Vec<String> {
    input[key]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn value_nums(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn string(input: &Value, key: &str, default: &str) -> String {
    input[key].as_str().unwrap_or(default).to_owned()
}

pub(super) fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

pub(super) fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub(super) fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub(super) fn pct(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() > f64::EPSILON {
        numerator / denominator * 100.0
    } else {
        0.0
    }
}

pub(super) fn percent(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}%")
}

pub(super) fn signed_percent(value: f64, decimals: usize) -> String {
    format!("{value:+.decimals$}%")
}

pub(super) fn currency(value: f64) -> String {
    format!("¥{:.2}", round2(value))
}

pub(super) fn signed_currency(value: f64) -> String {
    if value >= 0.0 {
        format!("+{}", currency(value))
    } else {
        format!("-{}", currency(value.abs()))
    }
}

pub(super) fn z_score(service_level: f64) -> f64 {
    if (service_level - 0.90).abs() < 0.001 {
        1.28
    } else if (service_level - 0.99).abs() < 0.001 {
        2.33
    } else {
        1.65
    }
}

pub(super) fn promo_phase(
    name: &str,
    days: i64,
    budget: f64,
    ratio: f64,
    target: &str,
    actions: &[&str],
) -> Value {
    json!({
        "阶段": name,
        "天数": days,
        "预算": currency(round2(budget * ratio)),
        "预算占比": percent(ratio * 100.0, 0),
        "目标": target,
        "动作": actions,
    })
}

pub(super) fn listing_keywords(name: &str, features: &[String], platform: &str) -> Value {
    let feature_words = features.iter().take(5).cloned().collect::<Vec<_>>();
    json!({
        "核心词": [name, platform],
        "卖点词": feature_words,
        "场景词": ["家用", "送礼", "日常", "新品"],
        "转化词": ["现货", "官方", "高性价比", "售后保障"],
    })
}

pub(super) fn commission_rate(platform: &str) -> f64 {
    match platform {
        "京东" => 0.08,
        "拼多多" => 0.03,
        "淘宝" | "抖音" | "快手" => 0.05,
        _ => 0.05,
    }
}

pub(super) fn price_position(price: f64, avg_comp: f64) -> &'static str {
    if price < avg_comp * 0.95 {
        "低价"
    } else if price > avg_comp * 1.05 {
        "高价"
    } else {
        "中价"
    }
}

pub(super) fn elasticity_label(elasticity: f64) -> &'static str {
    if elasticity <= -2.5 {
        "高弹性（价格敏感型买家，降价显著拉量）"
    } else if elasticity <= -1.5 {
        "中弹性（均衡定价，适度调整有效）"
    } else if elasticity <= -0.8 {
        "低弹性（品牌溢价空间，可适度提价）"
    } else {
        "超低弹性（刚需产品，价格影响有限）"
    }
}

pub(super) fn cv_xyz(history: &[f64]) -> (f64, &'static str) {
    if history.len() < 2 {
        return (0.0, "Z");
    }
    let mean = average(history);
    if mean == 0.0 {
        return (0.0, "Z");
    }
    let variance = history.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / history.len() as f64;
    let cv = round2((variance.sqrt() / mean) * 100.0) / 100.0;
    let xyz = if cv < 0.20 {
        "X"
    } else if cv < 0.50 {
        "Y"
    } else {
        "Z"
    };
    (cv, xyz)
}

pub(super) fn matrix_cells() -> std::collections::BTreeMap<String, (usize, f64)> {
    ["AX", "AY", "AZ", "BX", "BY", "BZ", "CX", "CY", "CZ"]
        .into_iter()
        .map(|cell| (cell.to_owned(), (0, 0.0)))
        .collect()
}

pub(super) fn matrix_strategy(cell: &str) -> (&'static str, f64) {
    match cell {
        "AX" => ("持续补货，自动化补货触发，维持最低安全库存", 1.2),
        "AY" => ("提前预测补货，安全库存1.5倍需求，定期评估", 1.5),
        "AZ" => ("柔性供应链，小批量高频补货，紧密监控", 2.0),
        "BX" => ("标准补货流程，定期审查", 1.3),
        "BY" => ("适度库存缓冲，季节性调整", 1.5),
        "BZ" => ("按需补货，避免积压", 1.8),
        "CX" => ("批量采购降成本，低频补货", 1.0),
        "CY" => ("清理尾货，考虑停售低利润款", 1.0),
        _ => ("建议停售或大促清仓", 0.5),
    }
}
