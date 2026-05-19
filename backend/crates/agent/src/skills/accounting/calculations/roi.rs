use serde_json::{Value, json};

use super::super::support::{currency, deferred_accounting, num, pct, percent, round2, string};

pub(in crate::skills::accounting) fn roi_calc(input: &Value) -> Value {
    let period = string(input, "period", "最近30天");
    let mut investment = num(input, "investment", 0.0);
    let revenue = num(input, "revenue_generated", 0.0);
    if investment <= 0.0 && revenue <= 0.0 {
        return deferred_accounting(
            "accounting_roi_calc",
            "纯计算版本需要手动传入 investment 和 revenue_generated。",
            input,
        );
    }
    if investment <= 0.0 {
        investment = 1.0;
    }
    let cogs = if input.get("cost_of_goods").is_some() {
        num(input, "cost_of_goods", 0.0)
    } else {
        round2(revenue * 0.40)
    };
    let aov = num(input, "avg_order_value", 150.0).max(0.01);
    let roas = round2(revenue / investment);
    let profit_roi = round2((revenue - cogs - investment) / investment);
    let acos = pct(investment, revenue);
    let cpa = if revenue > 0.0 {
        round2(investment / (revenue / aov))
    } else {
        0.0
    };
    let net_profit = round2(revenue - cogs - investment);
    let uses_defaults =
        input.get("cost_of_goods").is_none() || input.get("avg_order_value").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("ROI计算完成，ROAS {}，净利润{}，利润ROI {}", roas, currency(net_profit), profit_roi),
        "统计周期": period,
        "数据来源": "用户提供",
        "投入": investment,
        "产出营收": revenue,
        "ROI指标": {
            "ROAS(广告回报率)": roas,
            "真实广告ROI": roas,
            "利润ROI": profit_roi,
            "ACOS(广告成本占比)": percent(acos, 2),
            "CPA(获客成本)": cpa,
        },
        "盈亏分析": {
            "毛利润": round2(revenue - cogs),
            "净利润": net_profit,
            "是否盈利": if net_profit > 0.0 { "是" } else { "否" },
        },
        "评价": {
            "ROAS": if roas > 5.0 { "优秀" } else if roas > 3.0 { "良好" } else if roas > 1.0 { "及格" } else { "亏损" },
            "建议": if roas > 5.0 { "加大投放" } else if roas > 3.0 { "维持投放" } else if roas > 1.0 { "优化素材" } else { "暂停投放排查" },
        }
    })
}
