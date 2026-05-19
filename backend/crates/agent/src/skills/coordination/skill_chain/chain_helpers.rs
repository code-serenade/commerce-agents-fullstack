use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

pub(super) fn normalize_chain_role(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "运营" | "运营专家" | "operation" | "operations" | "ops" => "ops".to_string(),
        "数据" | "数据分析" | "数据分析师" | "analysis" | "analyst" | "data" => {
            "data".to_string()
        }
        "客服" | "客服专家" | "售后" | "service" | "customer_service" => {
            "service".to_string()
        }
        "设计" | "设计师" | "design" => "design".to_string(),
        "财务" | "会计" | "财务分析师" | "accounting" | "finance" => {
            "accounting".to_string()
        }
        "工程" | "技术" | "技术工程师" | "engineering" | "engineer" => {
            "engineering".to_string()
        }
        "网站" | "seo" | "web" => "web".to_string(),
        "创意" | "内容" | "内容创作者" | "creative" | "content" => "creative".to_string(),
        other if !other.is_empty() => other.to_string(),
        _ => "ops".to_string(),
    }
}

pub(super) fn normalize_chain_action(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "分析" | "诊断" | "复盘" | "analyze" | "diagnose" | "review" => "analyze".to_string(),
        "创建" | "生成" | "create" | "generate" => "create".to_string(),
        "优化" | "optimize" | "improve" => "optimize".to_string(),
        "规划" | "计划" | "plan" => "plan".to_string(),
        "执行" | "execute" => "execute".to_string(),
        "查询" | "query" | "status" => "query".to_string(),
        other if !other.is_empty() => other.to_string(),
        _ => "analyze".to_string(),
    }
}

pub(super) fn normalize_available_skills(values: Vec<String>) -> BTreeSet<String> {
    values
        .into_iter()
        .map(|value| normalize_skill_id(&value).to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

pub(super) fn normalize_skill_id(skill: &str) -> &str {
    match skill.trim() {
        "data_query_store_metrics" => "query_store_metrics",
        "data_query_product_metrics" => "query_product_metrics",
        "data_query_campaign_metrics" => "query_campaign_metrics",
        "service_response_templates" => "service_faq_playbook",
        value => value,
    }
}

pub(super) fn skill_chain_step(
    index: usize,
    skill: &str,
    reason: &str,
    available: &BTreeSet<String>,
) -> Value {
    let skill = normalize_skill_id(skill);
    let mode = skill_chain_execution_mode(skill);
    let prerequisites = skill_chain_prerequisites(skill);
    let available_in_runtime = available.is_empty() || available.contains(skill);
    json!({
        "步骤": index,
        "技能": skill,
        "原因": reason,
        "阶段": skill_chain_phase(skill),
        "负责角色": skill_owner_role(skill),
        "执行模式": mode,
        "风险等级": skill_chain_risk(mode),
        "前置依赖": prerequisites,
        "可用": available_in_runtime,
        "说明": skill_chain_mode_note(mode, available_in_runtime),
    })
}

pub(super) fn skill_chain_prerequisite_notes(chain: &[(&'static str, &'static str)]) -> Value {
    let mut notes = Map::new();
    for (skill, _) in chain {
        let skill = normalize_skill_id(skill);
        let prerequisites = skill_chain_prerequisites(skill);
        if !prerequisites.is_empty() {
            notes.insert(skill.to_string(), json!(prerequisites));
        }
    }
    if notes.is_empty() {
        json!("无特殊依赖")
    } else {
        Value::Object(notes)
    }
}

pub(super) fn skill_chain_prerequisite_gaps(chain: &[(&'static str, &'static str)]) -> Vec<Value> {
    let mut seen = BTreeSet::new();
    let mut gaps = Vec::new();
    for (skill, _) in chain {
        let skill = normalize_skill_id(skill);
        for prerequisite in skill_chain_prerequisites(skill) {
            if !seen.contains(prerequisite) {
                gaps.push(json!({
                    "技能": skill,
                    "缺少前置": prerequisite,
                    "处理建议": "执行该技能前先补齐前置数据或接受缺数据降级状态"
                }));
            }
        }
        seen.insert(skill.to_string());
    }
    gaps
}

fn skill_chain_prerequisites(skill: &str) -> Vec<&'static str> {
    match skill {
        "data_dashboard"
        | "data_anomaly_diagnosis"
        | "data_multi_period_trend"
        | "data_channel_roi"
        | "data_attribution_analysis"
        | "data_ltv_calculator"
        | "data_cohort_analysis"
        | "data_demand_forecast"
        | "data_seasonal_decompose"
        | "data_customer_segmentation"
        | "data_comprehensive_diagnosis"
        | "ops_execution_plan"
        | "ops_assortment_planning"
        | "ops_smart_pricing"
        | "ops_inventory_optimizer"
        | "ops_ad_fatigue_detector"
        | "ops_abc_xyz_classifier"
        | "accounting_pl_statement"
        | "accounting_financial_narrative"
        | "accounting_cash_flow_forecast"
        | "web_seo_optimize"
        | "web_page_conversion"
        | "service_dsr_improvement"
        | "service_nps_analyzer"
        | "service_nps_driver_analysis"
        | "creative_content_calendar" => vec!["query_store_metrics"],
        "ops_listing_copy"
        | "design_main_image"
        | "design_detail_page"
        | "web_product_description" => {
            vec!["query_product_metrics"]
        }
        "accounting_budget_vs_actual" => vec!["query_store_metrics", "accounting_budget_plan"],
        "ops_promo_planning" => vec!["query_campaign_metrics"],
        "coordination_create_campaign" => vec!["query_campaign_metrics", "ops_promo_planning"],
        "coordination_create_task" => vec!["ops_execution_plan"],
        "coordination_save_memory" => vec!["data_customer_segmentation"],
        _ => Vec::new(),
    }
}

fn skill_chain_phase(skill: &str) -> &'static str {
    if matches!(
        skill,
        "query_store_metrics" | "query_product_metrics" | "query_campaign_metrics"
    ) {
        "data_loading"
    } else if skill.starts_with("search_") {
        "evidence_gathering"
    } else if skill.starts_with("data_") {
        "analysis"
    } else if skill.starts_with("ops_") {
        "operations_planning"
    } else if skill.starts_with("accounting_") {
        "finance_validation"
    } else if skill.starts_with("service_") {
        "service_improvement"
    } else if skill.starts_with("creative_") {
        "content_generation"
    } else if skill.starts_with("design_") {
        "design_planning"
    } else if skill.starts_with("web_") {
        "growth_optimization"
    } else if skill.starts_with("engineering_") {
        "engineering_diagnostics"
    } else if skill.starts_with("platform_") || skill == "coordination_external_agent_call" {
        "approval_execution"
    } else if skill.starts_with("coordination_") {
        "workspace_coordination"
    } else {
        "other"
    }
}

fn skill_owner_role(skill: &str) -> &'static str {
    if matches!(
        skill,
        "query_store_metrics" | "query_product_metrics" | "query_campaign_metrics"
    ) || skill.starts_with("data_")
        || skill.starts_with("search_")
    {
        "data"
    } else if skill.starts_with("ops_") {
        "ops"
    } else if skill.starts_with("accounting_") {
        "accounting"
    } else if skill.starts_with("service_") {
        "service"
    } else if skill.starts_with("creative_") {
        "creative"
    } else if skill.starts_with("design_") {
        "design"
    } else if skill.starts_with("web_") {
        "web"
    } else if skill.starts_with("engineering_") {
        "engineering"
    } else {
        "coordination"
    }
}

fn skill_chain_execution_mode(skill: &str) -> &'static str {
    if skill.starts_with("platform_")
        || skill == "coordination_external_agent_call"
        || skill == "engineering_system_check"
    {
        "pending_approval"
    } else if matches!(
        skill,
        "query_store_metrics" | "query_product_metrics" | "query_campaign_metrics"
    ) || skill.starts_with("search_")
    {
        "adapter_required"
    } else {
        "native"
    }
}

fn skill_chain_risk(mode: &str) -> &'static str {
    match mode {
        "pending_approval" => "high",
        "adapter_required" => "medium",
        _ => "low",
    }
}

fn skill_chain_mode_note(mode: &str, available: bool) -> &'static str {
    if !available {
        return "当前可用技能列表未包含该技能；前端应展示为不可直接执行";
    }
    match mode {
        "pending_approval" => "涉及平台写操作、外部调用或健康检查，必须进入审批或人工确认链路",
        "adapter_required" => "需要用户输入、数据库或外部数据adapter；不可伪造成已读取平台实时数据",
        _ => "可在Rust Agent native skill内确定性执行，实际结果仍以skill_result为准",
    }
}
