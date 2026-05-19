use serde_json::{Value, json};

use super::super::helpers::{
    currency, deferred, empty_string_as_null, funnel_actions, funnel_bottleneck, has_any,
    missing_funnel_fields, optional_num, percent, rate, round0, round2, string,
};

pub(super) fn live_funnel_diagnosis(input: &Value) -> Value {
    let Some(impressions) = optional_num(input, "impressions") else {
        return deferred(
            "douyin_ads_live_funnel_diagnosis",
            "直播间投放漏斗诊断至少需要 impressions 和 clicks。",
            &[
                "impressions",
                "clicks",
                "live_room_entries",
                "product_clicks",
                "orders",
                "payments",
                "ad_spend",
                "gmv",
            ],
        );
    };
    let Some(clicks) = optional_num(input, "clicks") else {
        return deferred(
            "douyin_ads_live_funnel_diagnosis",
            "直播间投放漏斗诊断至少需要 impressions 和 clicks。",
            &[
                "impressions",
                "clicks",
                "live_room_entries",
                "product_clicks",
                "orders",
                "payments",
                "ad_spend",
                "gmv",
            ],
        );
    };

    let live_room_entries = optional_num(input, "live_room_entries");
    let product_clicks = optional_num(input, "product_clicks");
    let orders = optional_num(input, "orders");
    let payments = optional_num(input, "payments");
    let ad_spend = optional_num(input, "ad_spend");
    let gmv = optional_num(input, "gmv");
    let product_ref = string(input, "product_ref", "");

    let ctr = rate(clicks, impressions);
    let entry_rate = live_room_entries.map(|value| rate(value, clicks));
    let product_click_rate = match (product_clicks, live_room_entries) {
        (Some(product_clicks), Some(entries)) => Some(rate(product_clicks, entries)),
        _ => None,
    };
    let order_rate = match (orders, product_clicks) {
        (Some(orders), Some(product_clicks)) => Some(rate(orders, product_clicks)),
        _ => None,
    };
    let pay_rate = match (payments, orders) {
        (Some(payments), Some(orders)) => Some(rate(payments, orders)),
        _ => None,
    };
    let roas = match (gmv, ad_spend) {
        (Some(gmv), Some(ad_spend)) if ad_spend > 0.0 => Some(gmv / ad_spend),
        _ => None,
    };
    let bottleneck = funnel_bottleneck(ctr, entry_rate, product_click_rate, order_rate, pay_rate);

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": if has_any(input, &["live_room_entries", "product_clicks", "orders", "payments", "ad_spend", "gmv"]) { "user_provided" } else { "partial_user_input" },
        "summary": "直播间投放漏斗诊断已生成",
        "数据来源": "用户显式输入 + 新产品 Rust Agent 千川投放 deterministic skill；未读取抖音直播间或千川后台实时漏斗数据",
        "商品引用": empty_string_as_null(&product_ref),
        "漏斗指标": {
            "曝光": round0(impressions),
            "点击": round0(clicks),
            "点击率": percent(ctr, 2),
            "进房数": live_room_entries.map(round0).map(Value::from).unwrap_or(Value::Null),
            "点击到进房率": entry_rate.map(|value| json!(percent(value, 2))).unwrap_or(Value::Null),
            "商品点击数": product_clicks.map(round0).map(Value::from).unwrap_or(Value::Null),
            "进房到商品点击率": product_click_rate.map(|value| json!(percent(value, 2))).unwrap_or(Value::Null),
            "下单数": orders.map(round0).map(Value::from).unwrap_or(Value::Null),
            "商品点击到下单率": order_rate.map(|value| json!(percent(value, 2))).unwrap_or(Value::Null),
            "付款数": payments.map(round0).map(Value::from).unwrap_or(Value::Null),
            "下单到付款率": pay_rate.map(|value| json!(percent(value, 2))).unwrap_or(Value::Null),
            "广告消耗": ad_spend.map(currency).map(Value::from).unwrap_or(Value::Null),
            "GMV": gmv.map(currency).map(Value::from).unwrap_or(Value::Null),
            "ROAS": roas.map(round2).map(Value::from).unwrap_or(Value::Null)
        },
        "主要瓶颈": bottleneck,
        "动作建议": funnel_actions(&bottleneck),
        "待补字段": missing_funnel_fields(input),
        "风险提示": [
            "只有曝光和点击时只能判断前链路点击效率，不能直接判断直播成交承接",
            "GMV和消耗口径未扣除退款、佣金和履约成本时，不能当作利润ROI"
        ]
    })
}
