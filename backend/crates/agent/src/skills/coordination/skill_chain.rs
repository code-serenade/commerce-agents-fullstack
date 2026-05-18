use std::collections::BTreeSet;

use async_trait::async_trait;
use serde_json::{Map, Value, json};

use super::specs::coordination_skill_chain_planner_spec;
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params,
    optional_string_param, optional_string_vec_param, string_param,
};

pub struct CoordinationSkillChainPlanner {
    spec: SkillSpec,
}

impl CoordinationSkillChainPlanner {
    pub fn new() -> Self {
        Self {
            spec: coordination_skill_chain_planner_spec(),
        }
    }
}

impl Default for CoordinationSkillChainPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationSkillChainPlanner {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let goal = string_param(&params, "goal")?;
        let role = normalize_chain_role(
            optional_string_param(&params, "role")?.unwrap_or_else(|| "ops".to_owned()),
        );
        let action = normalize_chain_action(
            optional_string_param(&params, "action")?.unwrap_or_else(|| "analyze".to_owned()),
        );
        let available = normalize_available_skills(
            optional_string_vec_param(&params, "skills_available")?.unwrap_or_default(),
        );
        let chain = skill_chain_template(&goal, &role, &action);
        let detailed_steps = chain
            .iter()
            .enumerate()
            .map(|(index, (skill, reason))| skill_chain_step(index + 1, skill, reason, &available))
            .collect::<Vec<_>>();
        let prerequisite_notes = skill_chain_prerequisite_notes(&chain);
        let prerequisite_gaps = skill_chain_prerequisite_gaps(&chain);
        let unavailable = detailed_steps
            .iter()
            .filter(|step| step.get("可用").and_then(Value::as_bool) == Some(false))
            .filter_map(|step| {
                step.get("技能")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();
        let approval_steps = detailed_steps
            .iter()
            .filter(|step| step.get("执行模式").and_then(Value::as_str) == Some("pending_approval"))
            .filter_map(|step| {
                step.get("技能")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();
        let adapter_steps = detailed_steps
            .iter()
            .filter(|step| step.get("执行模式").and_then(Value::as_str) == Some("adapter_required"))
            .filter_map(|step| {
                step.get("技能")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();
        let unavailable_count = unavailable.len();
        let has_adapter_steps = !adapter_steps.is_empty();
        let requires_approval = !approval_steps.is_empty();
        let summary = format!(
            "技能链规划已生成：{}步，{}个adapter提示，{}个审批提示",
            detailed_steps.len(),
            adapter_steps.len(),
            approval_steps.len()
        );
        Ok(SkillOutcome::new(json!({
            "status": "success",
            "任务目标": goal,
            "角色": role,
            "动作": action,
            "推荐执行链": detailed_steps,
            "前置依赖说明": prerequisite_notes,
            "前置依赖缺口": prerequisite_gaps,
            "不可用技能": unavailable,
            "审批提示": approval_steps,
            "adapter提示": adapter_steps,
            "可用技能过滤": !available.is_empty(),
            "执行建议": "先加载真实数据或明确缺数据状态，再计算分析，最后生成内容、沉淀任务或进入审批；不得把adapter_required步骤展示成已读取平台实时数据。",
            "总步骤数": chain.len(),
            "规划来源": "Rust deterministic commerce template"
        }))
        .with_summary(summary)
        .with_metadata("skill_chain_step_count", json!(chain.len()))
        .with_metadata("skill_chain_has_adapter_steps", json!(has_adapter_steps))
        .with_metadata("skill_chain_requires_approval", json!(requires_approval))
        .with_metadata("skill_chain_unavailable_count", json!(unavailable_count)))
    }
}

fn agent_matches(task: &str) -> Vec<(String, String, i64)> {
    const AGENTS: &[(&str, &str, &[&str])] = &[
        (
            "ops",
            "运营专家",
            &[
                "运营", "促销", "活动", "推广", "投放", "选品", "定价", "库存",
            ],
        ),
        (
            "data",
            "数据分析师",
            &[
                "数据",
                "分析",
                "报表",
                "漏斗",
                "转化率",
                "趋势",
                "异常",
                "指标",
            ],
        ),
        (
            "service",
            "客服专家",
            &[
                "客服", "工单", "投诉", "退款", "退货", "售后", "DSR", "评价",
            ],
        ),
        (
            "design",
            "设计师",
            &["设计", "主图", "详情页", "配色", "海报", "视觉", "素材"],
        ),
        (
            "accounting",
            "财务分析师",
            &["财务", "成本", "利润", "预算", "ROI", "税务", "合规"],
        ),
        (
            "engineering",
            "技术工程师",
            &["技术", "架构", "bug", "性能", "部署", "开发", "接口"],
        ),
        (
            "web",
            "SEO专家",
            &["SEO", "网站", "店铺装修", "关键词", "标题", "转化", "建站"],
        ),
        (
            "creative",
            "内容创作者",
            &["视频", "文案", "种草", "直播", "IP", "内容", "创意"],
        ),
    ];

    AGENTS
        .iter()
        .copied()
        .map(|(agent, name, keywords)| {
            let score = keywords
                .iter()
                .filter(|keyword| task.contains(**keyword))
                .count() as i64;
            (agent.to_owned(), name.to_owned(), score)
        })
        .filter(|(_, _, score)| *score > 0)
        .collect()
}

fn orchestration_template(goal: &str) -> Vec<Value> {
    let launch = goal.contains("新品") || goal.contains("上新");
    let promo = goal.contains("大促") || goal.contains("活动") || goal.contains("618");
    let rows = if launch {
        vec![
            ("竞品分析与市场调研", "data", "2天", "无", "竞品报告"),
            (
                "定价策略与成本核算",
                "accounting",
                "1天",
                "任务1",
                "定价方案",
            ),
            ("主图与详情页设计", "design", "3天", "任务1", "设计稿"),
            ("商品文案与SEO标题", "web", "1天", "任务3", "优化文案"),
            ("种草内容与脚本", "creative", "2天", "任务3", "内容素材"),
            ("客服话术准备", "service", "1天", "任务4", "话术库"),
            ("上线推广与监控", "ops", "持续", "任务4,5,6", "上线运营"),
        ]
    } else if promo {
        vec![
            ("历史数据分析与目标设定", "data", "1天", "无", "数据报告"),
            ("活动策划与预算分配", "ops", "2天", "任务1", "活动方案"),
            (
                "财务预算与盈亏测算",
                "accounting",
                "1天",
                "任务2",
                "预算报告",
            ),
            ("活动海报与素材设计", "design", "2天", "任务2", "视觉素材"),
            ("直播脚本与种草内容", "creative", "1天", "任务2", "内容脚本"),
            ("客服应急话术与预案", "service", "1天", "任务2", "应急手册"),
            (
                "技术压测与系统保障",
                "engineering",
                "1天",
                "任务2",
                "压测报告",
            ),
        ]
    } else {
        vec![
            ("现状分析与数据调研", "data", "1天", "无", "分析报告"),
            ("策略与方案制定", "ops", "1天", "任务1", "执行方案"),
            ("财务可行性评估", "accounting", "1天", "任务2", "财务测算"),
            ("视觉设计", "design", "2天", "任务2", "设计稿"),
            ("内容与文案制作", "creative", "2天", "任务2", "内容素材"),
            ("执行落地与效果监控", "ops", "持续", "任务3,4,5", "执行报告"),
        ]
    };
    rows.into_iter()
        .enumerate()
        .map(|(idx, (task, agent, duration, dep, output))| {
            json!({
                "序号": idx + 1,
                "任务": task,
                "Agent": agent,
                "耗时": duration,
                "依赖": dep,
                "目标产出": output
            })
        })
        .collect()
}

fn normalize_chain_role(value: String) -> String {
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

fn normalize_chain_action(value: String) -> String {
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

fn normalize_available_skills(values: Vec<String>) -> BTreeSet<String> {
    values
        .into_iter()
        .map(|value| normalize_skill_id(&value).to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn normalize_skill_id(skill: &str) -> &str {
    match skill.trim() {
        "data_query_store_metrics" => "query_store_metrics",
        "data_query_product_metrics" => "query_product_metrics",
        "data_query_campaign_metrics" => "query_campaign_metrics",
        "service_response_templates" => "service_faq_playbook",
        value => value,
    }
}

fn skill_chain_step(
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

fn skill_chain_prerequisite_notes(chain: &[(&'static str, &'static str)]) -> Value {
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

fn skill_chain_prerequisite_gaps(chain: &[(&'static str, &'static str)]) -> Vec<Value> {
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

fn skill_chain_template(goal: &str, role: &str, action: &str) -> Vec<(&'static str, &'static str)> {
    let goal_lower = goal.to_lowercase();
    if goal_lower.contains("nps驱动")
        || goal.contains("投诉根因")
        || goal.contains("满意度根因")
        || goal.contains("贬低者")
    {
        return vec![
            ("query_store_metrics", "加载退款、评价和客户满意度相关数据"),
            ("service_nps_driver_analysis", "量化投诉类别对NPS的拖累"),
            ("service_sentiment_analyzer", "识别高危负面评价和情绪强度"),
            ("service_dsr_improvement", "生成DSR与售后改善动作"),
        ];
    }
    if goal_lower.contains("nps") || goal.contains("满意度") {
        return vec![
            ("query_store_metrics", "加载服务、退款和评价基线"),
            ("service_nps_analyzer", "计算NPS/CSAT基线"),
            ("service_nps_driver_analysis", "识别满意度驱动因素"),
            ("service_dsr_improvement", "生成服务改善动作"),
        ];
    }
    if goal_lower.contains("sla")
        || goal_lower.contains("p99")
        || goal_lower.contains("p90")
        || goal.contains("错误预算")
    {
        return vec![
            ("engineering_sla_monitor", "计算延迟分位数和错误预算"),
            ("engineering_perf_optimize", "基于SLA问题制定性能优化方案"),
            (
                "engineering_system_check",
                "检查平台/接口状态并进入审批确认",
            ),
        ];
    }
    if goal_lower.contains("ctr")
        || goal.contains("点击率")
        || goal.contains("标题吸引力")
        || goal.contains("情感触发")
    {
        return vec![
            ("creative_title_ctr_scorer", "评分标题吸引力和情感触发词"),
            ("creative_seeding_copy", "基于评分结果优化种草文案"),
            ("web_title_seo_scorer", "同步检查标题SEO质量"),
        ];
    }
    if goal_lower.contains("abc")
        || goal_lower.contains("xyz")
        || goal.contains("SKU分类")
        || goal.contains("库存矩阵")
    {
        return vec![
            ("query_store_metrics", "加载SKU销量、库存和波动数据"),
            ("ops_abc_xyz_classifier", "生成ABC-XYZ双维分类矩阵"),
            (
                "ops_inventory_optimizer",
                "按分类结果制定安全库存和补货策略",
            ),
        ];
    }
    if goal.contains("广告疲劳")
        || goal.contains("投放疲劳")
        || goal.contains("素材更新")
        || goal.contains("创意刷新")
    {
        return vec![
            ("query_store_metrics", "加载广告花费、点击率、ROI和UV数据"),
            ("ops_ad_fatigue_detector", "识别疲劳渠道和CTR衰减"),
            ("data_channel_roi", "对比渠道ROI，确认加码或收缩方向"),
            ("coordination_create_task", "创建素材更新或预算调整任务"),
        ];
    }
    if goal_lower.contains("rfm") || goal.contains("客群分层") || goal.contains("客户分层")
    {
        return vec![
            ("query_store_metrics", "加载客户交易和复购数据"),
            ("data_customer_segmentation", "生成RFM分层和高价值客群标签"),
            ("data_ltv_calculator", "计算不同客群LTV/CAC健康度"),
            (
                "coordination_save_memory",
                "保存高价值客群特征供后续对话复用",
            ),
        ];
    }
    if goal_lower.contains("seo") || goal.contains("关键词") || role == "web" {
        return vec![
            ("query_store_metrics", "加载当前搜索流量和转化基线"),
            ("web_keyword_research", "建立关键词矩阵"),
            ("web_title_seo_scorer", "评分当前标题"),
            ("web_seo_optimize", "输出优化方案"),
        ];
    }
    if goal.contains("设计") || goal.contains("主图") || role == "design" {
        return vec![
            ("design_material_spec", "确定素材规格"),
            ("design_main_image", "生成主图方案"),
            ("design_detail_page", "规划详情页"),
        ];
    }
    if goal.contains("技术") || goal_lower.contains("bug") || role == "engineering" {
        return vec![
            ("engineering_bug_analysis", "定位技术问题"),
            ("engineering_perf_optimize", "给出优化方案"),
            (
                "engineering_system_check",
                "检查平台/接口状态并进入审批确认",
            ),
        ];
    }
    if goal.contains("现金流") || goal.contains("预算差异") || goal.contains("预实差异")
    {
        return vec![
            ("query_store_metrics", "加载收入、支出和预算执行数据"),
            ("accounting_cash_flow_forecast", "预测未来现金流压力"),
            ("accounting_budget_vs_actual", "定位预算与实际差异"),
            ("accounting_financial_narrative", "生成财务健康诊断"),
        ];
    }
    if goal.contains("财务")
        || goal.contains("利润")
        || goal.contains("成本")
        || goal.contains("亏损")
    {
        return vec![
            ("query_store_metrics", "加载GMV、费用和利润相关数据"),
            ("accounting_pl_statement", "计算P&L利润表"),
            ("accounting_financial_narrative", "生成财务健康度诊断"),
        ];
    }
    if goal.contains("预测")
        || goal.contains("未来")
        || goal.contains("下月")
        || goal.contains("下季")
    {
        return vec![
            ("query_store_metrics", "加载历史时序数据"),
            ("data_demand_forecast", "预测订单或GMV需求"),
            ("data_seasonal_decompose", "分离季节性噪音和真实趋势"),
            ("data_cohort_analysis", "检查留存队列变化"),
        ];
    }
    if goal.contains("库存")
        || goal.contains("补货")
        || goal.contains("滞销")
        || goal.contains("积压")
    {
        return vec![
            ("query_store_metrics", "加载销量、库存和周转数据"),
            ("ops_inventory_optimizer", "计算补货、安全库存和滞销风险"),
            ("data_demand_forecast", "预测后续需求"),
            ("ops_smart_pricing", "如库存积压，制定清仓或利润定价方案"),
        ];
    }
    if goal.contains("促销")
        || goal.contains("大促")
        || goal.contains("618")
        || goal.contains("双11")
    {
        return vec![
            ("query_store_metrics", "加载当前GMV基线和广告ROI"),
            ("ops_promo_planning", "制定促销活动方案"),
            ("creative_seeding_copy", "生成活动配套种草文案"),
            ("coordination_create_campaign", "将活动沉淀为营销活动草稿"),
        ];
    }
    if goal.contains("新品")
        || goal.contains("上新")
        || goal.contains("选品")
        || goal.contains("商品")
    {
        return vec![
            ("ops_listing_copy", "生成商品标题、卖点和上架文案"),
            ("design_main_image", "生成主图设计方案"),
            ("creative_video_script", "生成首批种草视频脚本"),
            ("web_seo_optimize", "补齐搜索关键词和详情页SEO方向"),
        ];
    }
    if role == "data" || goal.contains("数据") || goal.contains("指标") || goal.contains("异常")
    {
        return vec![
            ("query_store_metrics", "加载多维度经营指标"),
            ("data_anomaly_diagnosis", "检测异常指标并定位根因"),
            ("data_multi_period_trend", "比较多周期趋势和拐点"),
        ];
    }
    if role == "accounting" {
        return vec![
            ("query_store_metrics", "加载财务分析所需经营指标"),
            ("accounting_pl_statement", "计算P&L利润表"),
            ("accounting_financial_narrative", "输出财务健康诊断"),
        ];
    }
    if role == "service" {
        return vec![
            ("query_store_metrics", "加载退款率、评价和客服相关指标"),
            ("service_sentiment_analyzer", "分析评价和投诉情绪强度"),
            ("service_faq_playbook", "生成标准化FAQ和应答口径"),
            ("service_dsr_improvement", "输出DSR改善方案"),
        ];
    }
    if role == "creative" {
        return vec![
            ("query_store_metrics", "了解商品和平台表现基线"),
            ("creative_video_script", "生成短视频脚本"),
            ("creative_seeding_copy", "生成配套种草文案"),
        ];
    }
    if action == "create" {
        return vec![
            ("query_store_metrics", "加载真实业务数据作为输入"),
            ("creative_video_script", "生成内容资产"),
            ("coordination_create_task", "创建后续工作区任务"),
        ];
    }
    if action == "plan" {
        return vec![
            ("query_store_metrics", "了解当前业绩基准"),
            ("ops_assortment_planning", "制定品类和选品规划"),
            ("ops_execution_plan", "生成30天可落地执行计划"),
            ("coordination_create_task", "把关键动作保存为工作区任务"),
        ];
    }
    vec![
        ("query_store_metrics", "加载真实业务数据作为分析基础"),
        ("ops_execution_plan", "生成可落地执行方案"),
        ("coordination_create_task", "创建后续工作区任务"),
    ]
}
