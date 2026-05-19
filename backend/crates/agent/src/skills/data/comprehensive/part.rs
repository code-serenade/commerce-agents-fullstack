use serde_json::{Value, json};

use crate::skills::data::{
    common::{
        round2, value_pointer_number, value_pointer_string, value_string, value_to_short_string,
    },
    output_contract::{data_deferred_reason, data_summary, normalize_data_output},
};

pub(super) fn diagnosis_part(module: &str, skill: &str, mut output: Value, input: &Value) -> Value {
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

pub(super) fn diagnosis_status(part: &Value) -> String {
    value_string(part, "status", "success")
}

pub(super) fn diagnosis_readiness(part: &Value) -> String {
    value_string(part, "data_readiness", "user_provided")
}

pub(super) fn diagnosis_is_user_success(part: &Value) -> bool {
    diagnosis_status(part) == "success"
        && part
            .get("has_data")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        && diagnosis_readiness(part) == "user_provided"
}

pub(super) fn diagnosis_conclusion(skill: &str, output: &Value) -> String {
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

pub(super) fn diagnosis_evidence(skill: &str, output: &Value) -> Value {
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

pub(super) fn data_deferred_reason_from_output(output: &Value) -> String {
    output
        .as_object()
        .map(data_deferred_reason)
        .unwrap_or_else(|| "需要补充用户输入数据".to_owned())
}

pub(super) fn ltv_cac_display(output: &Value) -> String {
    output
        .get("LTV/CAC")
        .map(value_to_short_string)
        .unwrap_or_else(|| "待评估".to_owned())
}
