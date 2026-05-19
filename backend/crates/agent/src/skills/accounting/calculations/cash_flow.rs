use serde_json::{Value, json};

use super::super::support::{currency, num, nums, round2, signed_percent};

pub(in crate::skills::accounting) fn cash_flow_forecast(input: &Value) -> Value {
    let days = num(input, "days", 30.0).max(1.0);
    let base_gmv = num(input, "base_gmv", 500_000.0).max(0.0);
    let base_ad = num(input, "base_ad_spend", round2(base_gmv * 0.10));
    let capex = num(input, "capex", 0.0);
    let loan_repayment = num(input, "loan_repayment", 0.0);
    let opening_cash = num(input, "opening_cash", 0.0);
    let mut growth = nums(input, "growth_assumptions");
    if growth.len() < 3 {
        growth.extend([5.0, 8.0, 12.0]);
        growth.truncate(3);
    }
    let factor = 30.0 / days;
    let monthly_gmv = base_gmv * factor;
    let monthly_ad = base_ad * factor;
    let labels = ["第1个月", "第2个月", "第3个月"];
    let mut running_cash = opening_cash;
    let mut forecast = Vec::new();
    let mut total_net_cf = 0.0;
    for (idx, growth_pct) in growth.iter().take(3).enumerate() {
        let multiplier = (1.0 + growth_pct / 100.0).powi((idx + 1) as i32);
        let gmv = round2(monthly_gmv * multiplier);
        let ad = round2(monthly_ad * multiplier);
        let refund = round2(gmv * 0.03);
        let cash_in = round2(gmv - refund);
        let cogs = round2(gmv * 0.40);
        let logistics = round2(gmv * 0.08);
        let platform = round2(gmv * 0.05);
        let opex = round2(cash_in * 0.10);
        let tax = round2(cash_in * 0.03);
        let operating_cf = round2(cash_in - cogs - logistics - platform - ad - opex - tax);
        let investing_cf = round2(-capex);
        let financing_cf = round2(-loan_repayment);
        let net_cf = round2(operating_cf + investing_cf + financing_cf);
        running_cash = round2(running_cash + net_cf);
        total_net_cf += net_cf;
        forecast.push(json!({
            "月份": labels[idx],
            "GMV预测": currency(gmv),
            "增长假设": signed_percent(*growth_pct, 1),
            "经营现金流入": currency(cash_in),
            "经营现金流出明细": {
                "商品成本": currency(cogs),
                "物流费": currency(logistics),
                "平台佣金": currency(platform),
                "广告投放": currency(ad),
                "运营费用": currency(opex),
                "税费": currency(tax),
            },
            "经营现金流净额": currency(operating_cf),
            "投资现金流": currency(investing_cf),
            "筹资现金流": currency(financing_cf),
            "月净现金流": currency(net_cf),
            "期末现金余额": currency(running_cash),
            "经营现金流状态": if operating_cf > 0.0 { "正常" } else { "预警" },
        }));
    }
    let risk_tips: Vec<String> = forecast
        .iter()
        .filter(|m| m["经营现金流状态"] == "预警")
        .map(|m| {
            format!(
                "{}经营现金流为负，需关注资金缺口",
                m["月份"].as_str().unwrap_or("")
            )
        })
        .collect();
    let uses_defaults = input.get("base_gmv").is_none()
        || input.get("base_ad_spend").is_none()
        || input.get("growth_assumptions").is_none()
        || input.get("capex").is_none()
        || input.get("loan_repayment").is_none()
        || input.get("opening_cash").is_none();
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("3个月现金流预测完成，累计净现金流{}，期末现金余额{}", currency(total_net_cf), currency(running_cash)),
        "数据来源": if input.get("base_gmv").is_some() { "用户提供基期数据 + 增长假设" } else { "默认基期数据 + 增长假设" },
        "基期月GMV": currency(monthly_gmv),
        "期初现金余额": currency(opening_cash),
        "3个月现金流预测": forecast,
        "3个月累计净现金流": currency(total_net_cf),
        "预测说明": "基于固定成本率和增长假设的确定性滚动预测；未调用外部数据或LLM。",
        "风险提示": if risk_tips.is_empty() { vec!["未发现明显资金缺口".to_owned()] } else { risk_tips },
    })
}
