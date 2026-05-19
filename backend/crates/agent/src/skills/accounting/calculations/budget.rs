use serde_json::{Value, json};

use super::super::support::{
    currency, deferred_accounting, num, pct, percent, round2, signed_percent, string, variance,
};

pub(in crate::skills::accounting) fn budget_plan(input: &Value) -> Value {
    let month = string(input, "month", "下月");
    let growth_rate = num(input, "growth_rate", 20.0);
    let target_gmv = num(input, "target_gmv", 500_000.0).max(0.0);
    let target_margin = num(input, "target_margin", 15.0).max(0.0);
    let target_profit = round2(target_gmv * target_margin / 100.0);
    let items = [
        ("采购成本", 0.40),
        ("物流费用", 0.08),
        ("平台佣金", 0.05),
        ("广告投放", 0.12),
        ("人力成本", 0.10),
        ("包装耗材", 0.03),
        ("其他费用", 0.02),
    ];
    let budget_items: Vec<_> = items
        .iter()
        .map(|(name, ratio)| {
            json!({
                "科目": name,
                "预算金额": currency(target_gmv * ratio),
                "占GMV": percent(ratio * 100.0, 1),
            })
        })
        .collect();
    let total_cost_rate: f64 = items.iter().map(|(_, ratio)| ratio).sum();
    let total_budget = round2(target_gmv * total_cost_rate);
    let planned_profit = round2(target_gmv - total_budget);
    let uses_defaults = input.get("target_gmv").is_none()
        || input.get("target_margin").is_none()
        || input.get("growth_rate").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("预算编制完成，目标GMV{}，计划利润{}", currency(target_gmv), currency(planned_profit)),
        "数据来源": if input.get("target_gmv").is_some() { "用户指定目标" } else { "默认目标（纯计算版本未读取历史数据）" },
        "预算月份": month,
        "目标增长率": signed_percent(growth_rate, 1),
        "目标GMV": currency(target_gmv),
        "目标利润": currency(target_profit),
        "预算明细": budget_items,
        "预算汇总": {
            "总预算成本": currency(total_budget),
            "计划利润": currency(planned_profit),
            "计划利润率": percent(pct(planned_profit, target_gmv), 1),
            "目标达成": if planned_profit >= target_profit { "满足目标利润率" } else { "低于目标利润率，需压缩费用或提高毛利" },
        },
    })
}

pub(in crate::skills::accounting) fn budget_vs_actual(input: &Value) -> Value {
    let actual_gmv = num(input, "actual_gmv", 0.0);
    if actual_gmv <= 0.0 {
        return deferred_accounting(
            "accounting_budget_vs_actual",
            "纯计算版本需要手动传入 actual_gmv；budget_gmv 不传时会按 actual_gmv×120%生成。",
            input,
        );
    }
    let budget_gmv = num(input, "budget_gmv", round2(actual_gmv * 1.20));
    let actual_ad = num(input, "actual_ad_spend", 0.0);
    let budget_ad = num(input, "budget_ad_spend", round2(budget_gmv * 0.12));
    let budget_cogs_rate = num(input, "budget_cogs_rate", 40.0) / 100.0;
    let actual_cogs_rate = num(input, "actual_cogs_rate", 40.0) / 100.0;
    let budget_opex_rate = num(input, "budget_opex_rate", 10.0) / 100.0;
    let actual_opex_rate = num(input, "actual_opex_rate", 10.0) / 100.0;
    let actual_refund_rate = num(input, "actual_refund_rate", 3.0) / 100.0;

    let budget_net_rev = round2(budget_gmv * 0.97);
    let budget_cogs = round2(budget_gmv * budget_cogs_rate);
    let budget_opex = round2(budget_gmv * budget_opex_rate);
    let budget_gross_profit = round2(budget_net_rev - budget_cogs);
    let budget_op_profit = round2(budget_gross_profit - budget_opex - budget_ad);
    let budget_margin = pct(budget_op_profit, budget_gmv);

    let actual_net_rev = round2(actual_gmv * (1.0 - actual_refund_rate));
    let actual_cogs = round2(actual_net_rev * actual_cogs_rate);
    let actual_opex = round2(actual_gmv * actual_opex_rate);
    let actual_gross_profit = round2(actual_net_rev - actual_cogs);
    let actual_op_profit = round2(actual_gross_profit - actual_opex - actual_ad);
    let actual_margin = pct(actual_op_profit, actual_gmv);

    let variance_table = json!({
        "GMV": variance(budget_gmv, actual_gmv, true),
        "净营收": variance(budget_net_rev, actual_net_rev, true),
        "商品成本": variance(budget_cogs, actual_cogs, false),
        "广告投入": variance(budget_ad, actual_ad, false),
        "运营费用": variance(budget_opex, actual_opex, false),
        "毛利润": variance(budget_gross_profit, actual_gross_profit, true),
        "营业利润": variance(budget_op_profit, actual_op_profit, true),
        "利润率": {
            "预算": percent(budget_margin, 2),
            "实际": percent(actual_margin, 2),
            "差异": format!("{:+.1}ppt", actual_margin - budget_margin),
            "状态": if actual_margin >= budget_margin { "有利" } else { "不利" },
        },
    });

    let gmv_ach = actual_gmv / budget_gmv.max(1.0);
    let margin_ach = actual_margin / budget_margin.max(0.1);
    let ad_eff = if actual_ad > 0.0 && budget_ad > 0.0 {
        budget_ad / actual_ad
    } else {
        1.0
    };
    let exec_score =
        ((gmv_ach * 0.50 + margin_ach * 0.35 + ad_eff.min(1.5) * 0.15) * 100.0).min(100.0);
    let mut risks = Vec::new();
    if actual_gmv < budget_gmv * 0.90 {
        risks.push(format!(
            "GMV未达标（完成率{:.0}%，差{}元）",
            gmv_ach * 100.0,
            round2(budget_gmv - actual_gmv)
        ));
    }
    if budget_ad > 0.0 && actual_ad > budget_ad * 1.15 {
        risks.push(format!(
            "广告超支（超出{}元）",
            round2(actual_ad - budget_ad)
        ));
    }
    if actual_margin < budget_margin * 0.85 {
        risks.push(format!(
            "利润率低于预算（实{} vs 预算{}）",
            percent(actual_margin, 2),
            percent(budget_margin, 2)
        ));
    }

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if input.get("budget_gmv").is_none()
            || input.get("budget_ad_spend").is_none()
            || input.get("budget_cogs_rate").is_none()
            || input.get("actual_cogs_rate").is_none()
            || input.get("budget_opex_rate").is_none()
            || input.get("actual_opex_rate").is_none()
            || input.get("actual_refund_rate").is_none() { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("预算vs实际分析完成，GMV完成率{}，执行质量评分{:.1}/100", percent(gmv_ach * 100.0, 1), exec_score),
        "数据说明": if input.get("budget_gmv").is_some() { "用户提供预算" } else { "预算自动生成：actual_gmv×120%" },
        "执行质量评分": format!("{:.1}/100", exec_score),
        "GMV完成率": percent(gmv_ach * 100.0, 1),
        "预算vs实际对比": variance_table,
        "风险项": if risks.is_empty() { vec!["各科目执行符合预算，无重大风险".to_owned()] } else { risks },
    })
}
