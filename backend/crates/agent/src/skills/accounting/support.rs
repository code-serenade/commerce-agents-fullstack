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
        .with_category(SkillCategory::Custom("accounting".to_owned()))
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

pub(super) fn deferred_accounting(skill: &str, reason: &str, input: &Value) -> Value {
    json!({
        "status": "deferred",
        "skill": skill,
        "deferred_reason": reason,
        "received_params": input,
        "side_effects": "none",
        "expected_outputs": ["结构化财务指标", "诊断摘要", "可执行建议"]
    })
}

pub(super) fn invalid_input(message: String) -> SkillError {
    SkillError::execution(message)
}

fn field_schema(field_type: &str) -> Value {
    match field_type {
        "string" => json!({"type": "string"}),
        "number" => json!({"type": "number"}),
        "integer" => json!({"type": "integer"}),
        "array<number>" => json!({"type": "array", "items": {"type": "number"}}),
        "array<object>" => json!({"type": "array", "items": {"type": "object"}}),
        _ => json!({"type": "string"}),
    }
}

pub(super) fn num(input: &Value, key: &str, default: f64) -> f64 {
    input[key]
        .as_f64()
        .or_else(|| input[key].as_i64().map(|v| v as f64))
        .or_else(|| input[key].as_u64().map(|v| v as f64))
        .unwrap_or(default)
}

pub(super) fn nums(input: &Value, key: &str) -> Vec<f64> {
    input[key]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|v| {
                    v.as_f64()
                        .or_else(|| v.as_i64().map(|n| n as f64))
                        .or_else(|| v.as_u64().map(|n| n as f64))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn string(input: &Value, key: &str, default: &str) -> String {
    input[key].as_str().unwrap_or(default).to_owned()
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

pub(super) fn variance(budget: f64, actual: f64, revenue_metric: bool) -> Value {
    let abs_var = round2(actual - budget);
    let pct_var = pct(abs_var, budget.abs().max(1.0));
    let favorable = if revenue_metric {
        abs_var >= 0.0
    } else {
        abs_var <= 0.0
    };
    json!({
        "预算": round2(budget),
        "实际": round2(actual),
        "差异": abs_var,
        "差异率": signed_percent(pct_var, 1),
        "状态": if favorable { "有利" } else { "不利" },
    })
}

pub(super) fn wf(step: &str, amount: f64, denominator: f64, direction: &str, _gmv: f64) -> Value {
    json!({
        "步骤": step,
        "金额": currency(amount),
        "占GMV比": if direction == "扣减" { signed_percent(pct(amount, denominator), 1) } else { percent(pct(amount, denominator), 1) },
        "方向": direction,
    })
}

pub(super) fn leak_advice(name: &str, amount: f64, gmv: f64) -> String {
    let prefix = format!(
        "当前最大成本耗损来自「{}」（{}，占GMV {}）；",
        name,
        currency(amount),
        percent(pct(amount, gmv), 1)
    );
    let advice = match name {
        "广告" => "建议优先优化广告ROI，降低单次获客成本",
        "商品成本" => "建议与供应链谈判降低采购成本或提升商品附加值",
        "物流" => "建议提升包装效率或与物流谈量价",
        "退款" => "建议排查退款根因，优化商品描述和品控",
        other => return format!("{prefix}建议精细化管控{other}支出"),
    };
    format!("{prefix}{advice}")
}

pub(super) fn calc_scenario(
    gmv: f64,
    label: String,
    base_margin: f64,
    base_ad: f64,
    base_refund: f64,
    fixed_cost: f64,
) -> Value {
    let net_revenue = gmv * (1.0 - base_refund);
    let gross_profit = net_revenue * base_margin;
    let ad_spend = gmv * base_ad;
    let platform_fee = gmv * 0.04;
    let logistics = gmv * 0.06;
    let contribution = gross_profit - ad_spend - platform_fee - logistics;
    let operating_profit = contribution - fixed_cost;
    let tax = (operating_profit * 0.25).max(0.0);
    let net_profit = operating_profit - tax;
    json!({
        "情景": label,
        "GMV": gmv.round(),
        "退款后净收入": net_revenue.round(),
        "毛利润": gross_profit.round(),
        "广告费": ad_spend.round(),
        "平台佣金": platform_fee.round(),
        "物流成本": logistics.round(),
        "贡献利润": contribution.round(),
        "固定成本": fixed_cost.round(),
        "营业利润": operating_profit.round(),
        "税后净利润": net_profit.round(),
        "净利率": percent(pct(net_profit, gmv), 1),
        "盈亏状态": if net_profit > 0.0 { "盈利" } else { "亏损" },
    })
}

pub(super) fn deterministic_risk_samples(
    base_gmv: f64,
    base_margin: f64,
    base_ad: f64,
    base_refund: f64,
    fixed_cost: f64,
) -> Vec<f64> {
    let mut samples = Vec::with_capacity(1000);
    for i in 0 .. 1000 {
        let t = i as f64;
        let gmv_factor = (1.0 + 0.15 * (t * 12.9898).sin()).max(0.1);
        let margin = (base_margin + 0.04 * (t * 78.233).cos()).max(0.01);
        let ad = (base_ad + 0.02 * (t * 37.719).sin()).max(0.0);
        let refund = (base_refund + 0.02 * (t * 19.191).cos()).max(0.0);
        let gmv = base_gmv * gmv_factor;
        let net_revenue = gmv * (1.0 - refund);
        let gross = net_revenue * margin;
        let contribution = gross - gmv * ad - gmv * 0.04 - gmv * 0.06;
        let operating_profit = contribution - fixed_cost;
        let tax = (operating_profit * 0.25).max(0.0);
        samples.push(operating_profit - tax);
    }
    samples
}

pub(super) fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

pub(super) fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p / 100.0).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}
