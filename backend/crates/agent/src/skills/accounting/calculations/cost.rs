use serde_json::{Value, json};

use super::super::support::{currency, num, pct, percent, round2};

pub(in crate::skills::accounting) fn cost_calc(input: &Value) -> Value {
    let purchase = num(input, "purchase_cost", 0.0);
    let shipping = num(input, "shipping_cost", 5.0);
    let packaging = num(input, "packaging_cost", 2.0);
    let fee_rate = num(input, "platform_fee_rate", 5.0);
    let ad_cost = num(input, "ad_cost_per_unit", 3.0);
    let price = num(input, "selling_price", 100.0);
    let platform_fee = round2(price * fee_rate / 100.0);
    let total_cost = round2(purchase + shipping + packaging + platform_fee + ad_cost);
    let profit = round2(price - total_cost);
    let margin = pct(profit, price);
    let uses_defaults = input.get("shipping_cost").is_none()
        || input.get("packaging_cost").is_none()
        || input.get("platform_fee_rate").is_none()
        || input.get("ad_cost_per_unit").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("成本核算完成，综合成本{}，单品利润{}，利润率{}", currency(total_cost), currency(profit), percent(margin, 2)),
        "售价": price,
        "成本明细": {
            "采购成本": purchase,
            "物流成本": shipping,
            "包装成本": packaging,
            "平台佣金": platform_fee,
            "推广成本": ad_cost,
        },
        "综合成本": total_cost,
        "单品利润": profit,
        "利润率": percent(margin, 2),
        "成本占比": {
            "采购": percent(pct(purchase, total_cost), 1),
            "物流": percent(pct(shipping, total_cost), 1),
            "包装": percent(pct(packaging, total_cost), 1),
            "平台": percent(pct(platform_fee, total_cost), 1),
            "推广": percent(pct(ad_cost, total_cost), 1),
        },
        "建议": if margin < 15.0 { "利润率偏低，建议优化采购或提价" } else { "利润率健康" },
    })
}
