use serde_json::{Value, json};

use crate::skills::data::{
    common::{has_any_input, i, input_array_missing_or_empty, nums, s},
    output_contract::data_summary,
};

pub(super) fn comprehensive_deferred(reason: &str, input: &Value) -> Value {
    json!({
        "status": "deferred",
        "skill": "data_comprehensive_diagnosis",
        "has_data": false,
        "adapter_status": "user_input_required",
        "data_readiness": "missing_user_input",
        "deferred_reason": reason,
        "summary": data_summary("data_comprehensive_diagnosis"),
        "received_params": input,
        "expected_inputs": [
            "经营指标：gmv/orders/visitors/clicks/ad_spend/refund_amount/gross_profit",
            "漏斗数据：impressions/clicks/add_to_cart/orders/payments",
            "时序数据：values",
            "渠道数据：channels",
            "退款明细：items",
            "客户或LTV指标：total_customers/cac/avg_order_value/purchase_frequency"
        ],
        "side_effects": "none"
    })
}

pub(super) fn product_ref(input: &Value) -> String {
    s(input, "product_ref", "")
}

pub(super) fn days(input: &Value) -> i64 {
    i(input, "days", 30).clamp(1, 365)
}

pub(super) fn has_comprehensive_diagnosis_input(input: &Value) -> bool {
    has_any_input(
        input,
        &[
            "gmv",
            "revenue",
            "previous_gmv",
            "orders",
            "order_count",
            "previous_orders",
            "visitors",
            "uv",
            "previous_visitors",
            "clicks",
            "add_to_cart",
            "payments",
            "impressions",
            "ad_spend",
            "ad_cost",
            "refund_amount",
            "refunds",
            "gross_profit",
            "profit",
            "target_gmv",
            "goal_gmv",
            "total_customers",
            "avg_order_value",
            "avg_orders_per_customer",
            "recency_days",
            "cac",
            "purchase_frequency",
            "gross_margin",
            "monthly_retention",
        ],
    ) || nums(input, "values").len() >= 3
        || !input_array_missing_or_empty(input, "channels")
        || !input_array_missing_or_empty(input, "items")
}

pub(super) fn normalize_focus(focus: &str) -> String {
    match focus {
        "growth" | "profit" | "retention" | "risk" | "all" => focus.to_owned(),
        _ => "all".to_owned(),
    }
}

pub(super) fn focus(input: &Value) -> String {
    normalize_focus(&s(input, "focus", "all"))
}

pub(super) fn focus_instruction(focus: &str) -> &'static str {
    match focus {
        "growth" => "重点聚焦增长机会、流量效率和转化改善",
        "profit" => "重点聚焦利润结构、投放效率和成本优化",
        "retention" => "重点聚焦客户留存、复购和LTV健康度",
        "risk" => "重点聚焦异常、退款、毛利和经营风险预警",
        _ => "全面均衡诊断，兼顾增长、利润、留存和风险",
    }
}
