use serde_json::{Value, json};

use super::super::support::{currency, deferred_accounting, num, pct, percent, round2};

pub(in crate::skills::accounting) fn profit_analysis(input: &Value) -> Value {
    let days = num(input, "days", 30.0).round();
    let revenue = num(input, "revenue", 0.0);
    if revenue <= 0.0 {
        return deferred_accounting(
            "accounting_profit_analysis",
            "纯计算版本需要手动传入 revenue 参数。",
            input,
        );
    }
    let refund = num(input, "refund_amount", 0.0);
    let cogs = if input.get("cogs").is_some() {
        num(input, "cogs", 0.0)
    } else {
        round2(revenue * 0.40)
    };
    let opex = if input.get("operating_expenses").is_some() {
        num(input, "operating_expenses", 0.0)
    } else {
        round2(revenue * 0.10)
    };
    let ad = num(input, "ad_spend", 0.0);
    let net_revenue = round2(revenue - refund);
    let gross_profit = round2(net_revenue - cogs);
    let gross_margin = pct(gross_profit, net_revenue);
    let operating_profit = round2(gross_profit - opex - ad);
    let operating_margin = pct(operating_profit, net_revenue);
    let uses_defaults = input.get("cogs").is_none() || input.get("operating_expenses").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("利润分析完成，净营收{}，营业利润{}，营业利润率{}", currency(net_revenue), currency(operating_profit), percent(operating_margin, 2)),
        "统计周期": format!("最近{}天", days),
        "数据来源": "用户提供",
        "营收分析": {
            "总营收": revenue,
            "退款": refund,
            "净营收": net_revenue,
            "退款率": percent(pct(refund, revenue), 2),
        },
        "利润分析": {
            "毛利润": gross_profit,
            "毛利率": percent(gross_margin, 2),
            "营业利润": operating_profit,
            "营业利润率": percent(operating_margin, 2),
        },
        "费用结构": {
            "商品成本": cogs,
            "运营费用": opex,
            "广告支出": ad,
            "费用率": percent(pct(cogs + opex + ad, net_revenue), 2),
        },
        "健康度": {
            "毛利率": if gross_margin > 40.0 { "健康" } else if gross_margin > 20.0 { "偏低" } else { "亏损风险" },
            "营业利润率": if operating_margin > 10.0 { "健康" } else if operating_margin > 0.0 { "偏低" } else { "亏损" },
            "广告占比": if pct(ad, net_revenue) < 15.0 { "合理" } else { "偏高" },
        }
    })
}

pub(in crate::skills::accounting) fn break_even_calc(input: &Value) -> Value {
    let price = num(input, "selling_price", 100.0);
    let unit_var = num(input, "unit_variable_cost", round2(price * 0.55));
    let fixed = num(input, "fixed_costs", round2(price * 200.0 * 0.13));
    let contribution_per_unit = price - unit_var;
    if contribution_per_unit <= 0.0 {
        return deferred_accounting(
            "accounting_break_even_calc",
            "单品变动成本高于售价，无法达到盈亏平衡；请降低成本或提高售价后重新计算。",
            input,
        );
    }

    let bep_units = (fixed / contribution_per_unit).round();
    let bep_revenue = round2(bep_units * price);
    let margin_of_safety_pct = if price > unit_var {
        (1.0 - 1.0 / (price / unit_var)) * 100.0
    } else {
        0.0
    };
    let price_variants = [0.8, 0.9, 1.0, 1.1, 1.2].map(|r| round2(price * r));
    let sales_targets = [0.5, 0.75, 1.0, 1.5, 2.0].map(|r| (bep_units * r).round());
    let sensitivity: Vec<_> = price_variants
        .iter()
        .map(|pv| {
            let mut row = serde_json::Map::new();
            row.insert("售价".to_owned(), json!(currency(*pv)));
            for sv in sales_targets {
                let profit = round2((pv - unit_var) * sv - fixed);
                row.insert(
                    format!("{}单", sv as i64),
                    json!(format!(
                        "{}{}",
                        currency(profit),
                        if profit >= 0.0 { "✓" } else { "✗" }
                    )),
                );
            }
            Value::Object(row)
        })
        .collect();
    let uses_defaults = input.get("selling_price").is_none()
        || input.get("unit_variable_cost").is_none()
        || input.get("fixed_costs").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("盈亏平衡分析完成，需销售{}单，需实现营收{}，贡献利润率{}", bep_units as i64, currency(bep_revenue), percent(pct(contribution_per_unit, price), 1)),
        "数据来源": "用户提供/默认参数",
        "定价": price,
        "单品变动成本": unit_var,
        "月固定成本": fixed,
        "单位贡献利润": round2(contribution_per_unit),
        "贡献利润率": percent(pct(contribution_per_unit, price), 1),
        "盈亏平衡点": {
            "需销售订单数": bep_units as i64,
            "需实现营收": currency(bep_revenue),
            "安全边际": percent(margin_of_safety_pct, 1),
        },
        "敏感性分析(利润矩阵)": sensitivity,
        "说明": "✓=盈利 ✗=亏损",
    })
}
