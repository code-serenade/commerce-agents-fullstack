use serde_json::{Value, json};

use super::super::helpers::{
    budget_action, currency, deferred, empty_string_as_null, int, num, optional_num, round2,
    signed_percent, string,
};

pub(super) fn budget_pacing(input: &Value) -> Value {
    let budget = optional_num(input, "budget");
    let spend = optional_num(input, "spend");
    let gmv = optional_num(input, "gmv");
    let current_roas = optional_num(input, "current_roas").or_else(|| match (gmv, spend) {
        (Some(gmv), Some(spend)) if spend > 0.0 => Some(gmv / spend),
        _ => None,
    });

    if budget.is_none() || current_roas.is_none() {
        return deferred(
            "douyin_ads_budget_pacing",
            "千川预算节奏需要投放总预算，并提供 current_roas，或同时提供 spend 与 gmv。",
            &[
                "budget",
                "current_roas 或 spend + gmv",
                "days",
                "target_roas",
            ],
        );
    }

    let budget = budget.unwrap_or_default().max(0.0);
    let days = int(input, "days", 7).max(1);
    let target_roas = num(input, "target_roas", 2.5).max(0.1);
    let current_roas = current_roas.unwrap_or_default().max(0.0);
    let spend = spend.unwrap_or(0.0).max(0.0);
    let gmv = gmv.unwrap_or(current_roas * spend).max(0.0);
    let product_ref = string(input, "product_ref", "");
    let remaining_budget = (budget - spend).max(0.0);
    let daily_budget = budget / days as f64;
    let remaining_daily_budget = remaining_budget / days as f64;
    let stop_line = target_roas * 0.70;
    let action = if current_roas >= target_roas * 1.15 {
        "放量"
    } else if current_roas >= target_roas * 0.90 {
        "稳投观察"
    } else if current_roas >= stop_line {
        "收缩优化"
    } else {
        "止损暂停"
    };

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": "user_provided",
        "summary": "千川预算节奏已计算",
        "数据来源": "用户显式输入 + 新产品 Rust Agent 千川投放 deterministic skill；未读取巨量千川实时消耗或转化数据",
        "商品引用": empty_string_as_null(&product_ref),
        "预算": {
            "总预算": currency(budget),
            "已消耗": currency(spend),
            "剩余预算": currency(remaining_budget),
            "周期天数": days,
            "理论日预算": currency(daily_budget),
            "剩余日均预算": currency(remaining_daily_budget)
        },
        "产出": {
            "GMV": currency(gmv),
            "当前ROAS": round2(current_roas),
            "目标ROAS": round2(target_roas),
            "止损线": round2(stop_line),
            "目标差距": signed_percent((current_roas / target_roas - 1.0) * 100.0, 1)
        },
        "动作判断": action,
        "预算动作": budget_action(action),
        "观察窗口": [
            "每30分钟观察点击率、进房率、商品点击率和付款转化",
            "每2-4小时观察ROAS、消耗速度和素材疲劳",
            "跨天保留稳定计划，不在短窗口内频繁重置学习"
        ],
        "风险提示": [
            "本结果只基于本轮显式输入，不代表已读取千川后台实时数据",
            "如果GMV未剔除退款、佣金、赠品和履约成本，ROAS不能等同利润ROI"
        ]
    })
}
