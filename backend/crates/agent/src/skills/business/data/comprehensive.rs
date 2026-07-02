mod input;
mod part;
mod report;
mod scoring;
mod synthesis;

use serde_json::{Value, json};

use self::{
    input::{
        comprehensive_deferred, days, focus, focus_instruction, has_comprehensive_diagnosis_input,
        product_ref,
    },
    part::{diagnosis_is_user_success, diagnosis_part, diagnosis_readiness, diagnosis_status},
    report::{DiagnosisReportDraft, build_diagnosis_report},
    scoring::{comprehensive_health_score, diagnosis_grade},
    synthesis::{
        comprehensive_action_plan, comprehensive_findings, comprehensive_missing_data,
        comprehensive_opportunities, comprehensive_risks,
    },
};
use super::{
    attribution::{attribution_analysis, ltv_calculator},
    common::empty_string_as_null,
    dashboard::data_dashboard,
    forecast::{demand_forecast, refund_decomposition},
    funnel_forecast::{anomaly_diagnosis, funnel_analysis},
    market::customer_segmentation,
};

pub(super) fn comprehensive_diagnosis(input: &Value) -> Value {
    let product_ref = product_ref(input);
    let days = days(input);
    let focus = focus(input);
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
        "分析方法": focus_instruction(&focus),
        "数据来源": "基于本次用户显式输入和新产品 Rust Agent 数据 skill 编排；未直连淘宝、抖音或数仓实时数据",
        "诊断覆盖": {
            "成功分项": success_count,
            "真实输入成功分项": user_success_count,
            "缺数据分项": deferred_count,
            "默认估算分项": default_count
        },
        "整体健康评分": {
            "总分": health_score,
            "等级": grade,
            "评分方式": "优先采用经营看板评分；无完整看板时按真实分项覆盖、风险数量和缺失数据进行规则评分"
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
