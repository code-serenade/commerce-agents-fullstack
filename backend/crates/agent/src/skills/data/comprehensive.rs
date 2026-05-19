use serde_json::{Value, json};

use super::{
    attribution::{attribution_analysis, ltv_calculator},
    common::{
        empty_string_as_null, has_any_input, i, input_array_missing_or_empty, nums, round2, s,
        value_pointer_number, value_pointer_string, value_string, value_to_short_string,
    },
    dashboard::data_dashboard,
    forecast::{demand_forecast, refund_decomposition},
    funnel_forecast::{anomaly_diagnosis, funnel_analysis},
    market::customer_segmentation,
    output_contract::{data_deferred_reason, data_summary, normalize_data_output},
};

pub(super) fn comprehensive_diagnosis(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let days = i(input, "days", 30).clamp(1, 365);
    let focus = normalize_focus(&s(input, "focus", "all"));
    if !has_comprehensive_diagnosis_input(input) {
        return comprehensive_deferred(
            "需要至少提供经营指标、漏斗数据、时序数据、渠道数据、退款明细、\
             客户指标或LTV输入中的一类",
            input,
        );
    }

    let parts = vec![
        diagnosis_part("经营看板", "data_dashboard", data_dashboard(input), input),
        diagnosis_part(
            "漏斗转化",
            "data_funnel_analysis",
            funnel_analysis(input),
            input,
        ),
        diagnosis_part(
            "异常检测",
            "data_anomaly_diagnosis",
            anomaly_diagnosis(input),
            input,
        ),
        diagnosis_part(
            "客户分层",
            "data_customer_segmentation",
            customer_segmentation(input),
            input,
        ),
        diagnosis_part(
            "退款根因",
            "data_refund_decomposition",
            refund_decomposition(input),
            input,
        ),
        diagnosis_part(
            "需求预测",
            "data_demand_forecast",
            demand_forecast(input),
            input,
        ),
        diagnosis_part(
            "渠道归因",
            "data_attribution_analysis",
            attribution_analysis(input),
            input,
        ),
        diagnosis_part(
            "LTV健康度",
            "data_ltv_calculator",
            ltv_calculator(input),
            input,
        ),
    ];

    let user_success_count = parts
        .iter()
        .filter(|part| diagnosis_is_user_success(part))
        .count();
    if user_success_count == 0 {
        return comprehensive_deferred(
            "已有输入不足以形成真实综合诊断；请补充至少一类可计算业务数据",
            input,
        );
    }

    let success_count = parts
        .iter()
        .filter(|part| diagnosis_status(part) == "success")
        .count();
    let deferred_count = parts
        .iter()
        .filter(|part| diagnosis_status(part) == "deferred")
        .count();
    let default_count = parts
        .iter()
        .filter(|part| {
            matches!(
                diagnosis_readiness(part).as_str(),
                "calculated_from_defaults" | "sample_only"
            )
        })
        .count();
    let has_partial_data = deferred_count > 0 || default_count > 0;
    let health_score = comprehensive_health_score(&parts);
    let grade = diagnosis_grade(health_score);
    let findings = comprehensive_findings(&parts);
    let risks = comprehensive_risks(&parts);
    let opportunities = comprehensive_opportunities(&parts, &focus);
    let action_plan = comprehensive_action_plan(&parts, &focus);
    let missing_data = comprehensive_missing_data(&parts);

    json!({
        "status": "success",
        "has_data": true,
        "has_partial_data": has_partial_data,
        "adapter_status": if has_partial_data { "partial_manual_input" } else { "manual_input" },
        "data_readiness": if has_partial_data { "partial_user_input" } else { "user_provided" },
        "summary": format!("综合业务诊断完成，健康评分{}分，等级{}", health_score, grade),
        "商品引用": empty_string_as_null(product_ref),
        "分析天数": days,
        "重点方向": focus,
        "分析口径": focus_instruction(&focus),
        "数据来源": "基于本次用户显式输入和新产品 Rust Agent 数据 skill 编排；未直连淘宝、抖音、千川或数仓实时数据",
        "诊断覆盖": {
            "成功分项": success_count,
            "真实输入成功分项": user_success_count,
            "缺数据分项": deferred_count,
            "默认估算分项": default_count
        },
        "整体健康评分": {
            "总分": health_score,
            "等级": grade,
            "评分口径": "优先采用经营看板评分；无完整看板时按真实分项覆盖、风险数量和缺失数据进行规则评分"
        },
        "综合诊断报告": build_diagnosis_report(DiagnosisReportDraft {
            days,
            focus: &focus,
            score: health_score,
            grade,
            findings: &findings,
            risks: &risks,
            opportunities: &opportunities,
            actions: &action_plan,
        }),
        "分项诊断": parts,
        "核心发现": findings,
        "风险预警": risks,
        "增长机会": opportunities,
        "7天行动计划": action_plan,
        "缺失数据": missing_data,
        "治理说明": "缺失分项以 deferred 明示，默认估算分项以 data_readiness 标记；前端不能展示为已读取平台实时数据。"
    })
}

fn comprehensive_deferred(reason: &str, input: &Value) -> Value {
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

fn diagnosis_part(module: &str, skill: &str, mut output: Value, input: &Value) -> Value {
    normalize_data_output(skill, input, &mut output);
    let status = value_string(&output, "status", "success");
    let data_readiness = value_string(&output, "data_readiness", "user_provided");
    let adapter_status = value_string(&output, "adapter_status", "manual_input");
    let has_data = output
        .get("has_data")
        .and_then(Value::as_bool)
        .unwrap_or(status == "success");
    let conclusion = diagnosis_conclusion(skill, &output);
    let evidence = diagnosis_evidence(skill, &output);

    json!({
        "模块": module,
        "skill": skill,
        "status": status,
        "has_data": has_data,
        "data_readiness": data_readiness,
        "adapter_status": adapter_status,
        "结论": conclusion,
        "关键证据": evidence,
        "output": output
    })
}

fn has_comprehensive_diagnosis_input(input: &Value) -> bool {
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

fn normalize_focus(focus: &str) -> String {
    match focus {
        "growth" | "profit" | "retention" | "risk" | "all" => focus.to_owned(),
        _ => "all".to_owned(),
    }
}

fn focus_instruction(focus: &str) -> &'static str {
    match focus {
        "growth" => "重点聚焦增长机会、流量效率和转化改善",
        "profit" => "重点聚焦利润结构、投放效率和成本优化",
        "retention" => "重点聚焦客户留存、复购和LTV健康度",
        "risk" => "重点聚焦异常、退款、毛利和经营风险预警",
        _ => "全面均衡诊断，兼顾增长、利润、留存和风险",
    }
}

fn diagnosis_status(part: &Value) -> String {
    value_string(part, "status", "success")
}

fn diagnosis_readiness(part: &Value) -> String {
    value_string(part, "data_readiness", "user_provided")
}

fn diagnosis_is_user_success(part: &Value) -> bool {
    diagnosis_status(part) == "success"
        && part
            .get("has_data")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        && diagnosis_readiness(part) == "user_provided"
}

fn diagnosis_conclusion(skill: &str, output: &Value) -> String {
    let status = value_string(output, "status", "success");
    if status == "deferred" {
        return format!("缺数据：{}", data_deferred_reason_from_output(output));
    }
    if let Some(summary) = output.get("summary").and_then(Value::as_str) {
        return summary.to_owned();
    }
    match skill {
        "data_dashboard" => {
            let score = value_pointer_number(output, "/经营健康度/总分")
                .map(|value| format!("{}分", round2(value)))
                .unwrap_or_else(|| "未评分".to_owned());
            let grade = value_pointer_string(output, "/经营健康度/等级", "未知");
            format!("经营健康度{score}，等级{grade}")
        }
        "data_anomaly_diagnosis" => {
            let count = output
                .get("异常点数")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            format!("发现{count}个异常点")
        }
        "data_customer_segmentation" => "客户分层已生成".to_owned(),
        "data_ltv_calculator" => {
            format!("LTV健康度{}", value_string(output, "健康度", "待评估"))
        }
        _ => data_summary(skill).to_owned(),
    }
}

fn diagnosis_evidence(skill: &str, output: &Value) -> Value {
    match skill {
        "data_dashboard" => json!({
            "健康度": output.get("经营健康度").cloned().unwrap_or(Value::Null),
            "预警数": output.get("预警").and_then(Value::as_array).map(Vec::len).unwrap_or_default()
        }),
        "data_funnel_analysis" => json!({
            "瓶颈环节": output.get("瓶颈环节").cloned().unwrap_or(Value::Null),
            "整体转化率": output.get("整体转化率").cloned().unwrap_or(Value::Null)
        }),
        "data_anomaly_diagnosis" => json!({
            "异常点数": output.get("异常点数").cloned().unwrap_or(Value::Null),
            "样本数": output.get("样本数").cloned().unwrap_or(Value::Null)
        }),
        "data_refund_decomposition" => json!({
            "总退款估算": output.get("总退款估算").cloned().unwrap_or(Value::Null),
            "帕累托分析": output.get("帕累托分析").cloned().unwrap_or(Value::Null)
        }),
        "data_demand_forecast" => json!({
            "预测结果": output.get("预测结果").cloned().unwrap_or(Value::Null),
            "备货建议": output.get("备货建议").cloned().unwrap_or(Value::Null)
        }),
        "data_attribution_analysis" => json!({
            "渠道数": output.get("渠道数").cloned().unwrap_or(Value::Null),
            "预算再分配建议": output.get("预算再分配建议").cloned().unwrap_or(Value::Null)
        }),
        "data_ltv_calculator" => json!({
            "LTV": output.get("LTV").cloned().unwrap_or(Value::Null),
            "LTV/CAC": output.get("LTV/CAC").cloned().unwrap_or(Value::Null),
            "健康度": output.get("健康度").cloned().unwrap_or(Value::Null)
        }),
        _ => json!({"summary": output.get("summary").cloned().unwrap_or(Value::Null)}),
    }
}

fn comprehensive_health_score(parts: &[Value]) -> i64 {
    if let Some(score) = parts
        .iter()
        .find(|part| {
            part.get("skill").and_then(Value::as_str) == Some("data_dashboard")
                && diagnosis_is_user_success(part)
        })
        .and_then(|part| part.get("output"))
        .and_then(|output| value_pointer_number(output, "/经营健康度/总分"))
    {
        return score.round().clamp(0.0, 100.0) as i64;
    }

    let user_success = parts
        .iter()
        .filter(|part| diagnosis_is_user_success(part))
        .count() as i64;
    let deferred = parts
        .iter()
        .filter(|part| diagnosis_status(part) == "deferred")
        .count() as i64;
    let risk_count = comprehensive_risks(parts).len() as i64;
    (62 + user_success * 5 - deferred * 3 - risk_count * 2).clamp(35, 92)
}

fn diagnosis_grade(score: i64) -> &'static str {
    if score >= 80 {
        "健康"
    } else if score >= 60 {
        "需关注"
    } else {
        "风险"
    }
}

fn comprehensive_findings(parts: &[Value]) -> Vec<String> {
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

fn comprehensive_risks(parts: &[Value]) -> Vec<String> {
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

fn comprehensive_opportunities(parts: &[Value], focus: &str) -> Vec<String> {
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
                    output
                        .get("LTV/CAC")
                        .map(value_to_short_string)
                        .unwrap_or_else(|| "待评估".to_owned())
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

fn comprehensive_action_plan(parts: &[Value], focus: &str) -> Vec<Value> {
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

fn comprehensive_missing_data(parts: &[Value]) -> Vec<String> {
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

struct DiagnosisReportDraft<'a> {
    days: i64,
    focus: &'a str,
    score: i64,
    grade: &'a str,
    findings: &'a [String],
    risks: &'a [String],
    opportunities: &'a [String],
    actions: &'a [Value],
}

fn build_diagnosis_report(draft: DiagnosisReportDraft<'_>) -> String {
    let mut report = format!(
        "【诊断基于{}天数据】\n整体健康评分：{}分（{}）。\n分析重点：{}。\n\n",
        draft.days,
        draft.score,
        draft.grade,
        focus_instruction(draft.focus)
    );
    report.push_str("核心发现：\n");
    for finding in draft.findings.iter().take(5) {
        report.push_str(&format!("- {finding}\n"));
    }
    report.push_str("\n风险预警：\n");
    for risk in draft.risks.iter().take(3) {
        report.push_str(&format!("- {risk}\n"));
    }
    report.push_str("\n增长机会：\n");
    for opportunity in draft.opportunities.iter().take(3) {
        report.push_str(&format!("- {opportunity}\n"));
    }
    report.push_str("\n7天行动计划：\n");
    for action in draft.actions.iter().take(5) {
        report.push_str(&format!(
            "- {}：{}\n",
            value_string(action, "优先级", "P?"),
            value_string(action, "动作", "待补充动作")
        ));
    }
    report
}

fn data_deferred_reason_from_output(output: &Value) -> String {
    output
        .as_object()
        .map(data_deferred_reason)
        .unwrap_or_else(|| "需要补充用户输入数据".to_owned())
}
