use serde_json::{Map, Value, json};

use super::reports::{
    arch_review_output, bug_analysis_output, perf_optimize_output, tech_selection_output,
};

pub(super) fn normalize_engineering_output(skill_id: &str, input: &Value, output: &mut Value) {
    let Some(map) = output.as_object_mut() else {
        return;
    };
    if map.contains_key("status") {
        return;
    }
    map.insert("status".to_owned(), json!("success"));
    map.insert("has_data".to_owned(), json!(true));
    map.insert("adapter_status".to_owned(), json!("manual_input"));
    map.insert(
        "data_readiness".to_owned(),
        json!(engineering_data_readiness(skill_id, input)),
    );
    map.insert("summary".to_owned(), json!(engineering_summary(skill_id)));
    map.insert(
        "data_boundary".to_owned(),
        json!(
            "本阶段基于用户输入和内置工程分析规则生成，不读取真实代码仓库、日志、APM、监控系统、\
             外部搜索或实时性能基准。"
        ),
    );
}

fn engineering_data_readiness(skill_id: &str, input: &Value) -> &'static str {
    match skill_id {
        "engineering_arch_review"
            if !has_any_input(
                input,
                &[
                    "daily_orders",
                    "tech_stack",
                    "pain_points",
                    "current_metrics",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "engineering_bug_analysis"
            if !has_any_input(
                input,
                &["error_message", "frequency", "tech_stack", "context"],
            ) =>
        {
            "calculated_from_defaults"
        }
        "engineering_perf_optimize"
            if !has_any_input(
                input,
                &["current_qps", "target_qps", "tech_stack", "metrics"],
            ) =>
        {
            "calculated_from_defaults"
        }
        "engineering_tech_selection"
            if !has_any_input(
                input,
                &["requirements", "constraints", "team_size", "current_stack"],
            ) =>
        {
            "calculated_from_defaults"
        }
        "engineering_sla_monitor"
            if !has_any_input(
                input,
                &[
                    "latency_samples",
                    "p50_ms",
                    "p90_ms",
                    "p99_ms",
                    "total_requests",
                    "error_count",
                    "downtime_minutes",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        _ => "user_provided",
    }
}

fn has_any_input(input: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        input.get(*key).is_some_and(|value| match value {
            Value::Null => false,
            Value::String(value) => !value.trim().is_empty(),
            Value::Array(values) => !values.is_empty(),
            Value::Object(values) => !values.is_empty(),
            _ => true,
        })
    })
}

pub(super) fn engineering_native_output(skill_id: &str, input: &Map<String, Value>) -> Value {
    match skill_id {
        "engineering_arch_review" => arch_review_output(input),
        "engineering_bug_analysis" => bug_analysis_output(input),
        "engineering_perf_optimize" => perf_optimize_output(input),
        "engineering_tech_selection" => tech_selection_output(input),
        _ => json!({"has_data": false, "error": format!("unknown engineering skill: {skill_id}")}),
    }
}

pub(super) fn engineering_summary(skill_id: &str) -> &'static str {
    match skill_id {
        "engineering_arch_review" => "工程架构评估报告已生成",
        "engineering_bug_analysis" => "Bug排查方案已生成",
        "engineering_perf_optimize" => "性能优化方案已生成",
        "engineering_tech_selection" => "技术选型报告已生成",
        "engineering_sla_monitor" => "SLA监控分析已完成",
        _ => "工程分析报告已生成",
    }
}
