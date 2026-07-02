use serde_json::{Value, json};

use crate::skills::{SkillCategory, SkillInputField, SkillPriority, SkillSpec};

pub(super) fn deferred(skill: &str, reason: &str, required_fields: &[&str]) -> Value {
    json!({
        "status": "deferred",
        "has_data": false,
        "adapter_status": "user_input_required",
        "data_readiness": "missing_user_input",
        "summary": reason,
        "skill": skill,
        "required_fields": required_fields,
        "说明": "当前不会伪造平台、直播间或广告后台实时数据；请补充本轮显式指标，或后续接入平台 adapter。"
    })
}

pub(super) fn budget_action(action: &str) -> Vec<&'static str> {
    match action {
        "放量" => vec![
            "保留原计划学习状态，单次加预算不超过30%",
            "复制优胜素材和人群包做相邻出价测试",
            "同步检查直播间库存、客服和优惠承接",
        ],
        "稳投观察" => vec![
            "维持预算，观察至少2个转化窗口",
            "优先优化低点击素材和低进房时段",
            "不要频繁重启计划，避免学习期反复重置",
        ],
        "收缩优化" => vec![
            "降低低效计划预算20%-30%",
            "保留高点击计划，替换转化弱素材或落地承接",
            "拆分人群包，排除高频无互动人群",
        ],
        _ => vec![
            "暂停低ROAS计划，保留少量复盘预算",
            "优先排查商品点击、下单和付款承接",
            "重新定义素材、人群和直播间话术假设后再测试",
        ],
    }
}

pub(super) fn funnel_bottleneck(
    ctr: f64,
    entry_rate: Option<f64>,
    product_click_rate: Option<f64>,
    order_rate: Option<f64>,
    pay_rate: Option<f64>,
) -> String {
    if ctr < 1.0 {
        return "广告点击弱，优先排查素材钩子、定向匹配和出价竞争力".to_string();
    }
    if entry_rate.is_some_and(|value| value < 45.0) {
        return "点击到进房弱，优先排查直播间封面、标题、承诺一致性和加载承接".to_string();
    }
    if product_click_rate.is_some_and(|value| value < 12.0) {
        return "进房到商品点击弱，优先排查主播讲解、货品摆放、权益露出和商品卡点击引导"
            .to_string();
    }
    if order_rate.is_some_and(|value| value < 8.0) {
        return "商品点击到下单弱，优先排查价格锚点、优惠门槛、信任背书和库存规格".to_string();
    }
    if pay_rate.is_some_and(|value| value < 70.0) {
        return "下单到付款弱，优先排查支付激励、客服答疑、优惠锁单和犹豫点".to_string();
    }
    "漏斗未见单一极端瓶颈，建议按素材、人群包和直播间节奏继续分层A/B测试".to_string()
}

pub(super) fn funnel_actions(bottleneck: &str) -> Vec<&'static str> {
    if bottleneck.contains("广告点击弱") {
        vec![
            "重做前3秒钩子",
            "按卖点拆素材赛马",
            "缩窄人群假设并提高素材相关性",
        ]
    } else if bottleneck.contains("点击到进房弱") {
        vec![
            "统一广告承诺和直播间首屏承接",
            "优化直播封面与标题",
            "检查投放时段和主播在线节奏",
        ]
    } else if bottleneck.contains("商品点击弱") {
        vec![
            "商品卡前置",
            "每15分钟重复核心卖点和权益",
            "用引流款承接进房用户",
        ]
    } else if bottleneck.contains("下单弱") {
        vec!["强化价格锚点", "缩短优惠路径", "补充真实评价和信任背书"]
    } else if bottleneck.contains("付款弱") {
        vec![
            "设置限时锁单权益",
            "客服及时处理支付疑虑",
            "降低优惠使用门槛",
        ]
    } else {
        vec![
            "继续拆素材、人群、时段做A/B",
            "保留稳定计划",
            "按ROAS和退款风险做预算迁移",
        ]
    }
}

pub(super) fn missing_funnel_fields(input: &Value) -> Vec<&'static str> {
    [
        ("live_room_entries", "live_room_entries"),
        ("product_clicks", "product_clicks"),
        ("orders", "orders"),
        ("payments", "payments"),
        ("ad_spend", "ad_spend"),
        ("gmv", "gmv"),
    ]
    .into_iter()
    .filter_map(|(key, label)| (!input_has(input, key)).then_some(label))
    .collect()
}

pub(super) fn spec(
    name: &str,
    display_name: &str,
    description: &str,
    input_fields: Vec<SkillInputField>,
) -> SkillSpec {
    let mut spec = SkillSpec::new(name, display_name, description)
        .with_category(SkillCategory::Custom("douyin_ads".to_owned()))
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

pub(super) fn field_schema(field_type: &str) -> Value {
    match field_type {
        "string" => json!({"type": "string"}),
        "number" => json!({"type": "number"}),
        "integer" => json!({"type": "integer"}),
        _ => json!({"type": "string"}),
    }
}

pub(super) fn num(input: &Value, key: &str, default: f64) -> f64 {
    optional_num(input, key).unwrap_or(default)
}

pub(super) fn optional_num(input: &Value, key: &str) -> Option<f64> {
    input[key]
        .as_f64()
        .or_else(|| input[key].as_i64().map(|value| value as f64))
        .or_else(|| input[key].as_u64().map(|value| value as f64))
        .filter(|value| value.is_finite())
}

pub(super) fn int(input: &Value, key: &str, default: i64) -> i64 {
    input[key]
        .as_i64()
        .or_else(|| {
            input[key]
                .as_u64()
                .and_then(|value| i64::try_from(value).ok())
        })
        .or_else(|| optional_num(input, key).map(|value| value.round() as i64))
        .unwrap_or(default)
}

pub(super) fn string(input: &Value, key: &str, default: &str) -> String {
    input[key]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
        .to_owned()
}

pub(super) fn input_has(input: &Value, key: &str) -> bool {
    match &input[key] {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        _ => true,
    }
}

pub(super) fn has_any(input: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| input_has(input, key))
}

pub(super) fn empty_string_as_null(value: &str) -> Value {
    if value.trim().is_empty() {
        Value::Null
    } else {
        json!(value)
    }
}

pub(super) fn rate(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() > f64::EPSILON {
        numerator / denominator * 100.0
    } else {
        0.0
    }
}

pub(super) fn round0(value: f64) -> i64 {
    value.round().max(0.0) as i64
}

pub(super) fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
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
