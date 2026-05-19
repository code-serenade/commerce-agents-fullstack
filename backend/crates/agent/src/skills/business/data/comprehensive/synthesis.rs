use serde_json::{Value, json};

use super::part::{
    data_deferred_reason_from_output, diagnosis_is_user_success, diagnosis_status, ltv_cac_display,
};
use crate::skills::data::common::value_string;

pub(super) fn comprehensive_findings(parts: &[Value]) -> Vec<String> {
    let mut findings = parts
        .iter()
        .filter(|part| diagnosis_is_user_success(part))
        .filter_map(|part| {
            let module = part.get("模块").and_then(Value::as_str)?;
            let conclusion = part.get("结论").and_then(Value::as_str)?;
            Some(format!("{module}：{conclusion}"))
        })
        .take(6)
        .collect::<Vec<_>>();
    if findings.is_empty() {
        findings.push("当前输入不足，尚未形成可引用的核心发现".to_owned());
    }
    findings
}

pub(super) fn comprehensive_risks(parts: &[Value]) -> Vec<String> {
    let mut risks = Vec::new();
    for part in parts {
        if !diagnosis_is_user_success(part) {
            continue;
        }
        let skill = part
            .get("skill")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(output) = part.get("output") else {
            continue;
        };
        match skill {
            "data_dashboard" => {
                if let Some(alerts) = output.get("预警").and_then(Value::as_array) {
                    for alert in alerts.iter().take(3) {
                        let level = alert.get("level").and_then(Value::as_str).unwrap_or("");
                        if level == "warning" {
                            risks.push(value_string(alert, "message", "经营指标触发预警"));
                        }
                    }
                }
            }
            "data_anomaly_diagnosis" => {
                let count = output
                    .get("异常点数")
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                if count > 0 {
                    risks.push(format!(
                        "时序数据发现{count}个异常点，需要排查活动、投放或供给变化。"
                    ));
                }
            }
            "data_refund_decomposition" if diagnosis_status(part) == "success" => {
                if let Some(first) = output
                    .get("帕累托分析")
                    .and_then(Value::as_array)
                    .and_then(|items| items.first())
                {
                    risks.push(format!(
                        "退款集中在{}，应优先处理头部售后原因。",
                        value_string(first, "项目", "关键问题项")
                    ));
                }
            }
            _ => {}
        }
    }
    if risks.is_empty() {
        risks.push("未发现明确高优先级风险；仍需持续补充平台实时数据进行复核。".to_owned());
    }
    risks.into_iter().take(5).collect()
}

pub(super) fn comprehensive_opportunities(parts: &[Value], focus: &str) -> Vec<String> {
    let mut opportunities = Vec::new();
    for part in parts {
        if !diagnosis_is_user_success(part) {
            continue;
        }
        let skill = part
            .get("skill")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(output) = part.get("output") else {
            continue;
        };
        match skill {
            "data_funnel_analysis" if diagnosis_status(part) == "success" => {
                opportunities.push(format!(
                    "围绕{}做转化专项优化：{}。",
                    value_string(output, "瓶颈环节", "当前瓶颈"),
                    value_string(output, "优化建议", "复核流量和转化链路")
                ));
            }
            "data_attribution_analysis" if diagnosis_status(part) == "success" => {
                opportunities.push(value_string(
                    output,
                    "预算再分配建议",
                    "按归因贡献重排渠道预算",
                ));
            }
            "data_demand_forecast" if diagnosis_status(part) == "success" => {
                opportunities.push(value_string(output, "备货建议", "按预测需求安排库存和补货"));
            }
            "data_ltv_calculator" if diagnosis_status(part) == "success" => {
                opportunities.push(format!(
                    "以LTV/CAC={}为复购和获客约束，优先提升高价值客群复购。",
                    ltv_cac_display(output)
                ));
            }
            _ => {}
        }
    }
    if opportunities.is_empty() {
        opportunities.push(match focus {
            "profit" => "先补齐毛利、广告花费和退款结构，再定位利润优化机会。".to_owned(),
            "retention" => "先补齐客户、复购和LTV指标，再定位留存提升机会。".to_owned(),
            "risk" => "先补齐时序、退款和毛利指标，再定位经营风险。".to_owned(),
            _ => "先补齐流量、转化和渠道数据，再定位增长机会。".to_owned(),
        });
    }
    opportunities.into_iter().take(5).collect()
}

pub(super) fn comprehensive_action_plan(parts: &[Value], focus: &str) -> Vec<Value> {
    let mut actions = Vec::new();
    let missing = comprehensive_missing_data(parts);
    if !missing.is_empty() {
        actions.push(json!({
            "优先级": "P0",
            "动作": "补齐缺失数据并重新运行综合诊断",
            "依据": missing.iter().take(3).cloned().collect::<Vec<_>>()
        }));
    }
    for (idx, risk) in comprehensive_risks(parts).into_iter().take(2).enumerate() {
        actions.push(json!({
            "优先级": format!("P{}", idx + 1),
            "动作": format!("处理风险：{risk}"),
            "依据": "风险预警"
        }));
    }
    for (idx, opportunity) in comprehensive_opportunities(parts, focus)
        .into_iter()
        .take(2)
        .enumerate()
    {
        actions.push(json!({
            "优先级": format!("P{}", idx + 3),
            "动作": opportunity,
            "依据": "增长机会"
        }));
    }
    actions.into_iter().take(5).collect()
}

pub(super) fn comprehensive_missing_data(parts: &[Value]) -> Vec<String> {
    parts
        .iter()
        .filter(|part| diagnosis_status(part) == "deferred")
        .filter_map(|part| {
            let module = part.get("模块").and_then(Value::as_str)?;
            let output = part.get("output")?;
            Some(format!(
                "{module}：{}",
                data_deferred_reason_from_output(output)
            ))
        })
        .collect()
}
