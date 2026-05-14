use std::{cmp::Reverse, collections::BTreeSet};

use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillInputField, SkillOutcome, SkillPriority, SkillResult,
    SkillSpec, object_params, optional_f64_param, optional_i64_param, optional_string_param,
    optional_string_vec_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("coordination".to_owned())
}

pub fn specs() -> Vec<SkillSpec> {
    vec![
        coordination_agent_handoff_spec(),
        coordination_task_orchestration_spec(),
        coordination_status_query_spec(),
        coordination_create_campaign_spec(),
        coordination_create_task_spec(),
        coordination_save_memory_spec(),
        platform_update_price_spec(),
        platform_update_inventory_spec(),
        platform_sync_products_spec(),
        coordination_skill_chain_planner_spec(),
        coordination_external_agent_call_spec(),
    ]
}

pub fn coordination_agent_handoff_spec() -> SkillSpec {
    SkillSpec::new(
        "coordination_agent_handoff",
        "Agent交接",
        "根据任务需求，确定应交接的目标Agent及交接信息",
    )
    .with_category(category())
    .with_priority(SkillPriority::Normal)
    .with_input(SkillInputField::optional(
        "current_agent",
        "当前Agent角色",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::required(
        "task_description",
        "需交接的任务描述",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "context",
        "上下文信息",
        json!({"type": "string"}),
    ))
    .with_tag("deterministic")
}

pub fn coordination_task_orchestration_spec() -> SkillSpec {
    SkillSpec::new(
        "coordination_task_orchestration",
        "任务编排",
        "将复杂需求拆分为多Agent协作的子任务，确定执行顺序",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::required(
        "goal",
        "整体目标",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "requirements",
        "具体需求列表",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_tag("deterministic")
}

pub fn coordination_status_query_spec() -> SkillSpec {
    SkillSpec::new(
        "coordination_status_query",
        "状态查询",
        "查询当前会话中Agent、任务、工作区和上下文的可见运行状态",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::optional(
        "query_type",
        "查询类型：agent_status/task_status/system_health/workspace_status/session_status",
        json!({"type": "string"}),
    ))
    .with_tag("deterministic")
}

pub fn coordination_create_campaign_spec() -> SkillSpec {
    SkillSpec::new(
        "coordination_create_campaign",
        "创建营销活动草稿",
        "在当前工作区生成一个可沉淀的营销活动草稿，复用商品、上下文和执行规划信息。",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::required(
        "name",
        "活动名称，例如：618大促-主推款秒杀",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "description",
        "活动说明或背景",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "campaign_type",
        "活动类型：launch/promotion/ad_review/agent_draft",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "budget",
        "活动预算（元），例如：5000",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "关联产品ID（UUID，可选；不传时优先使用当前上下文商品）",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "goal",
        "本次活动目标，例如：提升周末转化率",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "target_audience",
        "目标人群，例如：18-30岁通勤咖啡用户",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "channels",
        "投放/运营渠道列表，例如：douyin/taobao",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "strategy",
        "活动策略，例如：先小预算测款再放量",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "start_date",
        "开始日期，格式 YYYY-MM-DD",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "end_date",
        "结束日期，格式 YYYY-MM-DD",
        json!({"type": "string"}),
    ))
    .with_tag("deterministic")
}

pub fn coordination_create_task_spec() -> SkillSpec {
    SkillSpec::new(
        "coordination_create_task",
        "创建工作任务",
        "在当前工作区创建一个新任务，指定负责角色、优先级和验收标准",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::required(
        "title",
        "任务标题，例如：优化主图CTR，目标提升20%",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "description",
        "任务详细描述",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "owner_role",
        "负责角色：ops/data/service/design/accounting/engineering/web/creative",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "priority",
        "优先级：0=普通, 1=重要, 2=紧急",
        json!({"type": "integer"}),
    ))
    .with_input(SkillInputField::optional(
        "acceptance_criteria",
        "验收标准，例如：CTR提升至3.5%以上",
        json!({"type": "string"}),
    ))
    .with_tag("deterministic")
}

pub fn coordination_save_memory_spec() -> SkillSpec {
    SkillSpec::new(
        "coordination_save_memory",
        "保存工作记忆",
        "将对话中发现的重要事实、决策或结论保存到工作区记忆，供后续对话使用",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::required(
        "key",
        "记忆键名，例如：pricing_strategy_2024",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::required(
        "content",
        "记忆内容，例如：本品定价区间150-180元，低于竞品均价200元",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "memory_type",
        "记忆类型：fact=事实, decision=决策, insight=洞察, warning=风险",
        json!({"type": "string"}),
    ))
    .with_tag("deterministic")
}

pub fn platform_update_price_spec() -> SkillSpec {
    pending_spec(
        "platform_update_price",
        "修改商品价格",
        "将商品价格修改同步到淘宝/京东/拼多多/抖音等真实平台。需要平台API凭证，修改前必须确认。",
        vec![
            SkillInputField::required(
                "platform",
                "平台标识: taobao/jd/pdd/douyin",
                json!({"type": "string"}),
            ),
            SkillInputField::required(
                "product_id",
                "平台商品ID（从平台后台或同步接口获取）",
                json!({"type": "string"}),
            ),
            SkillInputField::required(
                "new_price",
                "新价格（元），例如 99.9",
                json!({"type": "number"}),
            ),
            SkillInputField::optional(
                "sku_id",
                "SKU ID（可选，多规格商品时使用）",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn platform_update_inventory_spec() -> SkillSpec {
    pending_spec(
        "platform_update_inventory",
        "修改商品库存",
        "将商品库存数量修改同步到淘宝/京东/拼多多/抖音等真实平台。需要用户已配置平台API凭证。",
        vec![
            SkillInputField::required(
                "platform",
                "平台标识: taobao/jd/pdd/douyin",
                json!({"type": "string"}),
            ),
            SkillInputField::required("product_id", "平台商品ID", json!({"type": "string"})),
            SkillInputField::required("quantity", "新库存数量", json!({"type": "integer"})),
            SkillInputField::optional("sku_id", "SKU ID（可选）", json!({"type": "string"})),
        ],
    )
}

pub fn platform_sync_products_spec() -> SkillSpec {
    pending_spec(
        "platform_sync_products",
        "同步平台商品",
        "从已连接的平台实时拉取商品列表，获取真实商品ID、当前价格和库存。",
        vec![
            SkillInputField::required(
                "platform",
                "平台标识: taobao/jd/pdd/douyin",
                json!({"type": "string"}),
            ),
            SkillInputField::optional("page", "页码，默认1", json!({"type": "integer"})),
            SkillInputField::optional("page_size", "每页数量，默认20", json!({"type": "integer"})),
        ],
    )
}

pub fn coordination_skill_chain_planner_spec() -> SkillSpec {
    SkillSpec::new(
        "coordination_skill_chain_planner",
        "技能链规划",
        "分析任务目标，返回推荐的技能调用顺序和依赖关系。",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::required(
        "goal",
        "整体任务目标，例如：分析本月亏损原因",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "role",
        "当前Agent角色: ops/data/accounting/web/creative等",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "action",
        "动作类型: analyze/create/optimize/plan/execute/query",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "skills_available",
        "可用技能名列表（可选，用于精确匹配）",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_tag("deterministic")
}

pub fn coordination_external_agent_call_spec() -> SkillSpec {
    pending_spec(
        "coordination_external_agent_call",
        "外部Agent调用",
        "调用外部系统或Agent API（A2A集成）。endpoint必须在系统配置的白名单中，不能调用任意URL。",
        vec![
            SkillInputField::required(
                "endpoint",
                "外部API/Agent端点URL（必须在白名单内）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "method",
                "HTTP方法",
                json!({"type": "string", "enum": ["GET", "POST"]}),
            ),
            SkillInputField::optional("payload", "POST请求体（JSON）", json!({"type": "object"})),
            SkillInputField::required(
                "description",
                "本次调用目的（用于日志和用户展示）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "extract_key",
                "从响应JSON中提取的字段路径，支持点号分隔（如 data.price）",
                json!({"type": "string"}),
            ),
        ],
    )
}

fn deferred_spec(
    id: &str,
    name: &str,
    description: &str,
    inputs: Vec<SkillInputField>,
) -> SkillSpec {
    inputs.into_iter().fold(
        SkillSpec::new(id, name, description)
            .with_category(category())
            .with_priority(SkillPriority::Normal)
            .with_tag("deferred"),
        SkillSpec::with_input,
    )
}

fn pending_spec(
    id: &str,
    name: &str,
    description: &str,
    inputs: Vec<SkillInputField>,
) -> SkillSpec {
    inputs.into_iter().fold(
        SkillSpec::new(id, name, description)
            .with_category(category())
            .with_priority(SkillPriority::High)
            .with_tag("pending_approval")
            .with_tag("side_effect"),
        SkillSpec::with_input,
    )
}

pub struct CoordinationAgentHandoff {
    spec: SkillSpec,
}

impl CoordinationAgentHandoff {
    pub fn new() -> Self {
        Self {
            spec: coordination_agent_handoff_spec(),
        }
    }
}

impl Default for CoordinationAgentHandoff {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationAgentHandoff {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        let current = optional_string_param(&params, "current_agent")?
            .unwrap_or_else(|| "unknown".to_owned());
        let task = string_param(&params, "task_description")?;
        let context = optional_string_param(&params, "context")?.unwrap_or_default();
        let mut matches = agent_matches(&task)
            .into_iter()
            .filter(|(agent, _, _)| *agent != current)
            .collect::<Vec<_>>();
        matches.sort_by_key(|(_, _, score)| Reverse(*score));
        if matches.is_empty() {
            matches.push(("ops".to_owned(), "运营专家".to_owned(), 0));
        }
        let target = matches[0].clone();
        let alternatives = matches
            .iter()
            .skip(1)
            .take(2)
            .map(|(agent, name, score)| json!({"agent": agent, "name": name, "匹配度": score}))
            .collect::<Vec<_>>();

        Ok(SkillOutcome::new(json!({
            "当前Agent": current,
            "任务描述": task,
            "推荐交接": {
                "目标Agent": target.0,
                "Agent名称": target.1,
                "匹配度": target.2
            },
            "备选Agent": alternatives,
            "交接信息": {
                "任务摘要": task,
                "上下文": if context.is_empty() { "无额外上下文".to_owned() } else { context },
                "优先级": "中"
            }
        }))
        .with_summary("Agent交接建议已生成"))
    }
}

pub struct CoordinationTaskOrchestration {
    spec: SkillSpec,
}

impl CoordinationTaskOrchestration {
    pub fn new() -> Self {
        Self {
            spec: coordination_task_orchestration_spec(),
        }
    }
}

impl Default for CoordinationTaskOrchestration {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationTaskOrchestration {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let goal = string_param(&params, "goal")?;
        let requirements = optional_string_vec_param(&params, "requirements")?.unwrap_or_default();
        let tasks = orchestration_template(&goal);
        Ok(SkillOutcome::new(json!({
            "目标": goal,
            "需求": if requirements.is_empty() { vec!["参考标准流程".to_owned()] } else { requirements },
            "任务编排": tasks,
            "总工期估算": "约7天（含并行）",
            "关键路径": "任务1 -> 任务2 -> 并行执行 -> 收尾",
            "风险提示": ["多Agent协作需同步边界和验收标准", "设计/内容/技术任务建议设置缓冲"],
            "编排方式": "Rust deterministic template"
        }))
        .with_summary("任务编排骨架已生成"))
    }
}

macro_rules! deferred_coordination_skill {
    ($type_name:ident, $spec_fn:ident, $status:literal, required [$($required:literal),*], defaults {$($key:literal => $value:expr),* $(,)?}) => {
        pub struct $type_name {
            spec: SkillSpec,
        }

        impl $type_name {
            pub fn new() -> Self {
                Self { spec: $spec_fn() }
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }

        #[async_trait]
        impl Skill for $type_name {
            fn spec(&self) -> &SkillSpec {
                &self.spec
            }

            async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
                object_params(&params)?;
                $(let _ = string_param(&params, $required)?;)*
                let mut normalized = serde_json::Map::new();
                $(normalized.insert($key.to_owned(), json!($value(&params)?));)*
                Ok(SkillOutcome::new(json!({
                    "status": $status,
                    "approval_required": $status == "pending_approval",
                    "deferred_reason": "协调/平台动作尚未接入Rust runtime；本实现不执行数据库、网络或平台写动作",
                    "skill": self.spec.id,
                    "normalized_inputs": normalized,
                    "no_side_effects": true
                })).with_summary("协调技能骨架已返回"))
            }
        }
    };
}

pub struct CoordinationStatusQuery {
    spec: SkillSpec,
}

impl CoordinationStatusQuery {
    pub fn new() -> Self {
        Self {
            spec: coordination_status_query_spec(),
        }
    }
}

impl Default for CoordinationStatusQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationStatusQuery {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let query_type = normalize_query_type(
            optional_string_param(&params, "query_type")?
                .unwrap_or_else(|| infer_status_query_type(&context.metadata).to_owned()),
        );
        let visible_scope = visible_scope(&context);
        let session = session_status_summary(&context);
        let worker = worker_status_summary(&context);
        let task = task_status_summary(&context);
        let workspace = workspace_status_summary(&context);
        let context_status = context_readiness_summary(&context);
        let health = system_health_summary(&context);
        let blockers = coordination_blockers(&context);
        let observability = observability_limits(&context);

        let summary = status_summary_line(&query_type, &worker, &task, &workspace, &health);
        Ok(SkillOutcome::new(json!({
            "status": "success",
            "query_type": query_type,
            "scope": visible_scope,
            "session": session,
            "worker": worker,
            "task": task,
            "workspace": workspace,
            "context": context_status,
            "system_health": health,
            "blockers": blockers,
            "observability": observability,
        }))
        .with_summary(summary)
        .with_metadata("status_query_scope", json!("session_local"))
        .with_metadata("status_query_type", json!(query_type)))
    }
}

pub struct CoordinationCreateCampaign {
    spec: SkillSpec,
}

impl CoordinationCreateCampaign {
    pub fn new() -> Self {
        Self {
            spec: coordination_create_campaign_spec(),
        }
    }
}

impl Default for CoordinationCreateCampaign {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CoordinationCreateTask {
    spec: SkillSpec,
}

pub struct CoordinationSaveMemory {
    spec: SkillSpec,
}

impl CoordinationSaveMemory {
    pub fn new() -> Self {
        Self {
            spec: coordination_save_memory_spec(),
        }
    }
}

impl Default for CoordinationSaveMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl CoordinationCreateTask {
    pub fn new() -> Self {
        Self {
            spec: coordination_create_task_spec(),
        }
    }
}

impl Default for CoordinationCreateTask {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationCreateCampaign {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let name = compact_text_for_task_title(&string_param(&params, "name")?);
        let description = optional_string_param(&params, "description")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty())
            .or_else(|| metadata_string_value(&context.metadata, "task_description"));
        let campaign_type = normalize_campaign_type(
            optional_string_param(&params, "campaign_type")?
                .unwrap_or_else(|| infer_campaign_type(&name, description.as_deref())),
        );
        let budget = optional_f64_param(&params, "budget")?;
        let product_id = optional_string_param(&params, "product_id")?
            .or_else(|| infer_primary_product_id(&context.metadata));
        let goal = optional_string_param(&params, "goal")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty())
            .or_else(|| description.clone())
            .or_else(|| Some(name.clone()));
        let target_audience = optional_string_param(&params, "target_audience")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty());
        let channels = normalize_campaign_channels(
            optional_string_vec_param(&params, "channels")?.unwrap_or_default(),
        );
        let strategy = optional_string_param(&params, "strategy")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty());
        let start_date = optional_string_param(&params, "start_date")?
            .map(|value| compact_task_text_value(&value, 64))
            .filter(|value| !value.is_empty());
        let end_date = optional_string_param(&params, "end_date")?
            .map(|value| compact_task_text_value(&value, 64))
            .filter(|value| !value.is_empty());
        let workspace_id = metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id"))
            .or_else(|| metadata_string_value(&context.metadata, "request.workspace.workspace_id"));
        let role = metadata_string_value(&context.metadata, "role_runtime")
            .or_else(|| metadata_string_value(&context.metadata, "assignment_role"))
            .unwrap_or_else(|| "agent".to_string());
        let source_summary =
            metadata_string_value(&context.metadata, "request.context.selected_summary")
                .map(|value| compact_text_for_task_body(&value))
                .or_else(|| description.clone());
        let draft = build_campaign_draft_blueprint(
            &name,
            &campaign_type,
            goal.as_deref(),
            target_audience.as_deref(),
            &channels,
            strategy.as_deref(),
            budget,
        );

        let normalized = json!({
            "name": name,
            "description": description,
            "campaign_type": campaign_type,
            "budget": budget,
            "product_id": product_id,
            "goal": goal,
            "target_audience": target_audience,
            "channels": channels,
            "strategy": strategy,
            "start_date": start_date,
            "end_date": end_date,
            "workspace_id": workspace_id,
            "role": role,
            "source_summary": source_summary,
            "draft": draft,
            "source": "session_skill",
        });

        Ok(SkillOutcome::new(json!({
            "status": "success",
            "campaign_creation": {
                "should_persist": true,
                "name": normalized.get("name").cloned().unwrap_or(Value::Null),
                "description": normalized.get("description").cloned().unwrap_or(Value::Null),
                "campaign_type": normalized.get("campaign_type").cloned().unwrap_or(Value::Null),
                "budget": normalized.get("budget").cloned().unwrap_or(Value::Null),
                "product_id": normalized.get("product_id").cloned().unwrap_or(Value::Null),
                "goal": normalized.get("goal").cloned().unwrap_or(Value::Null),
                "target_audience": normalized.get("target_audience").cloned().unwrap_or(Value::Null),
                "channels": normalized.get("channels").cloned().unwrap_or_else(|| json!([])),
                "strategy": normalized.get("strategy").cloned().unwrap_or(Value::Null),
                "start_date": normalized.get("start_date").cloned().unwrap_or(Value::Null),
                "end_date": normalized.get("end_date").cloned().unwrap_or(Value::Null),
                "workspace_id": normalized.get("workspace_id").cloned().unwrap_or(Value::Null),
                "role": normalized.get("role").cloned().unwrap_or(Value::Null),
                "source_summary": normalized.get("source_summary").cloned().unwrap_or(Value::Null),
                "draft": normalized.get("draft").cloned().unwrap_or_else(|| json!({})),
                "source": "session_skill"
            },
            "normalized_inputs": normalized,
            "no_side_effects": true
        }))
        .with_summary(format!("营销活动草稿已生成：{}", name))
        .with_metadata("campaign_creation_kind", json!("campaign_draft"))
        .with_metadata("campaign_creation_should_persist", json!(true)))
    }
}

#[async_trait]
impl Skill for CoordinationCreateTask {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let title = compact_text_for_task_title(&string_param(&params, "title")?);
        let description = optional_string_param(&params, "description")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty())
            .or_else(|| metadata_string_value(&context.metadata, "task_description"));
        let owner_role = normalize_owner_role(
            optional_string_param(&params, "owner_role")?
                .or_else(|| metadata_string_value(&context.metadata, "assignment_role"))
                .or_else(|| metadata_string_value(&context.metadata, "role_runtime")),
        );
        let priority = optional_i64_param(&params, "priority")?
            .map(clamp_coordination_priority)
            .unwrap_or(3);
        let acceptance_criteria = optional_string_param(&params, "acceptance_criteria")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty());
        let workspace_id = metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id"))
            .or_else(|| metadata_string_value(&context.metadata, "request.workspace.workspace_id"));

        let normalized = json!({
            "title": title,
            "description": description,
            "owner_role": owner_role,
            "priority": priority,
            "acceptance_criteria": acceptance_criteria,
            "workspace_id": workspace_id,
            "source": "session_skill",
        });

        Ok(SkillOutcome::new(json!({
            "status": "success",
            "task_creation": {
                "should_persist": true,
                "title": normalized.get("title").cloned().unwrap_or(Value::Null),
                "description": normalized.get("description").cloned().unwrap_or(Value::Null),
                "owner_role": normalized.get("owner_role").cloned().unwrap_or(Value::Null),
                "priority": normalized.get("priority").cloned().unwrap_or(Value::Null),
                "acceptance_criteria": normalized.get("acceptance_criteria").cloned().unwrap_or(Value::Null),
                "workspace_id": normalized.get("workspace_id").cloned().unwrap_or(Value::Null),
                "source": "session_skill"
            },
            "normalized_inputs": normalized,
            "no_side_effects": true
        }))
        .with_summary("工作任务草案已生成，可落到当前工作区")
        .with_metadata("task_creation_kind", json!("workspace_task"))
        .with_metadata("task_creation_should_persist", json!(true)))
    }
}

#[async_trait]
impl Skill for CoordinationSaveMemory {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let key = normalize_memory_key(&string_param(&params, "key")?);
        let content = compact_text_for_task_body(&string_param(&params, "content")?);
        let memory_type = normalize_memory_type(
            optional_string_param(&params, "memory_type")?.unwrap_or_else(|| {
                infer_memory_type_from_text(&content, &context.metadata).to_string()
            }),
        );
        let workspace_id = metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id"))
            .or_else(|| metadata_string_value(&context.metadata, "request.workspace.workspace_id"));
        let role = metadata_string_value(&context.metadata, "role_runtime")
            .or_else(|| metadata_string_value(&context.metadata, "assignment_role"))
            .unwrap_or_else(|| "agent".to_string());

        let normalized = json!({
            "key": key,
            "content": content,
            "memory_type": memory_type,
            "workspace_id": workspace_id,
            "role": role,
            "source": "session_skill",
        });

        Ok(SkillOutcome::new(json!({
            "status": "success",
            "memory_save": {
                "should_persist": true,
                "key": normalized.get("key").cloned().unwrap_or(Value::Null),
                "content": normalized.get("content").cloned().unwrap_or(Value::Null),
                "memory_type": normalized.get("memory_type").cloned().unwrap_or(Value::Null),
                "workspace_id": normalized.get("workspace_id").cloned().unwrap_or(Value::Null),
                "role": normalized.get("role").cloned().unwrap_or(Value::Null),
                "source": "session_skill"
            },
            "normalized_inputs": normalized,
            "no_side_effects": true
        }))
        .with_summary(format!("工作记忆草案已生成：{}", key))
        .with_metadata("memory_save_kind", json!("workspace_learning_record"))
        .with_metadata("memory_save_should_persist", json!(true)))
    }
}

deferred_coordination_skill!(
    PlatformUpdatePrice,
    platform_update_price_spec,
    "pending_approval",
    required ["platform", "product_id"],
    defaults {
        "platform" => |p: &Value| string_param(p, "platform").map(Some),
        "product_id" => |p: &Value| string_param(p, "product_id").map(Some),
        "new_price" => |p: &Value| optional_f64_param(p, "new_price"),
        "sku_id" => |p: &Value| optional_string_param(p, "sku_id"),
    }
);
deferred_coordination_skill!(
    PlatformUpdateInventory,
    platform_update_inventory_spec,
    "pending_approval",
    required ["platform", "product_id"],
    defaults {
        "platform" => |p: &Value| string_param(p, "platform").map(Some),
        "product_id" => |p: &Value| string_param(p, "product_id").map(Some),
        "quantity" => |p: &Value| optional_i64_param(p, "quantity"),
        "sku_id" => |p: &Value| optional_string_param(p, "sku_id"),
    }
);
deferred_coordination_skill!(
    PlatformSyncProducts,
    platform_sync_products_spec,
    "pending_approval",
    required ["platform"],
    defaults {
        "platform" => |p: &Value| string_param(p, "platform").map(Some),
        "page" => |p: &Value| Ok(optional_i64_param(p, "page")?.unwrap_or(1)),
        "page_size" => |p: &Value| Ok(optional_i64_param(p, "page_size")?.unwrap_or(20)),
    }
);
deferred_coordination_skill!(
    CoordinationExternalAgentCall,
    coordination_external_agent_call_spec,
    "pending_approval",
    required ["endpoint", "description"],
    defaults {
        "endpoint" => |p: &Value| string_param(p, "endpoint").map(Some),
        "method" => |p: &Value| Ok(optional_string_param(p, "method")?.unwrap_or_else(|| "GET".to_owned())),
        "payload" => |p: &Value| Ok(p.get("payload").cloned().unwrap_or_else(|| json!({}))),
        "description" => |p: &Value| string_param(p, "description").map(Some),
        "extract_key" => |p: &Value| optional_string_param(p, "extract_key"),
    }
);

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
        "accounting_budget_vs_actual" => vec!["query_store_metrics", "accounting_budget_plan"],
        "coordination_create_campaign" => vec!["ops_promo_planning"],
        "coordination_create_task" => vec!["ops_execution_plan"],
        "coordination_save_memory" => vec!["data_customer_segmentation"],
        _ => Vec::new(),
    }
}

fn skill_chain_phase(skill: &str) -> &'static str {
    if skill == "query_store_metrics" {
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
    if skill == "query_store_metrics" || skill.starts_with("data_") || skill.starts_with("search_")
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
    } else if skill == "query_store_metrics" || skill.starts_with("search_") {
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

fn normalize_query_type(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "agent" | "agent_status" => "agent_status".to_string(),
        "task" | "task_status" => "task_status".to_string(),
        "workspace" | "workspace_status" => "workspace_status".to_string(),
        "session" | "session_status" => "session_status".to_string(),
        "health" | "system" | "system_health" => "system_health".to_string(),
        _ => "system_health".to_string(),
    }
}

fn infer_primary_product_id(metadata: &Map<String, Value>) -> Option<String> {
    metadata_csv_list(metadata, "context.product_ids")
        .into_iter()
        .next()
        .or_else(|| {
            metadata_csv_list(metadata, "context.refs")
                .into_iter()
                .find_map(|item| item.strip_prefix("product:").map(ToOwned::to_owned))
        })
}

fn normalize_campaign_type(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "launch" | "上新" | "冷启动" => "launch".to_string(),
        "promotion" | "promo" | "campaign" | "促销" | "大促" => "promotion".to_string(),
        "ad_review" | "review" | "复盘" => "ad_review".to_string(),
        "agent_draft" | "draft" | "草稿" => "agent_draft".to_string(),
        _ => compact_task_text_value(value.trim(), 64),
    }
}

fn infer_campaign_type(name: &str, description: Option<&str>) -> String {
    let text = format!("{} {}", name, description.unwrap_or_default()).to_ascii_lowercase();
    if text.contains("复盘") || text.contains("review") {
        "ad_review".to_string()
    } else if text.contains("上新") || text.contains("冷启动") || text.contains("launch") {
        "launch".to_string()
    } else if text.contains("促销")
        || text.contains("大促")
        || text.contains("618")
        || text.contains("双11")
        || text.contains("promotion")
    {
        "promotion".to_string()
    } else {
        "agent_draft".to_string()
    }
}

fn normalize_campaign_channels(values: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let channel = match value.trim().to_ascii_lowercase().as_str() {
            "抖音" | "douyin" => "douyin".to_string(),
            "淘宝" | "taobao" => "taobao".to_string(),
            "京东" | "jd" => "jd".to_string(),
            "拼多多" | "pdd" => "pdd".to_string(),
            other => compact_task_text_value(other, 64),
        };
        if !normalized.iter().any(|item| item == &channel) {
            normalized.push(channel);
        }
    }
    normalized
}

fn build_campaign_draft_blueprint(
    name: &str,
    campaign_type: &str,
    goal: Option<&str>,
    target_audience: Option<&str>,
    channels: &[String],
    strategy: Option<&str>,
    budget: Option<f64>,
) -> Value {
    let execution_steps = match campaign_type {
        "launch" => vec![
            "确认主推商品、定价和核心卖点".to_string(),
            "完成首批渠道内容和素材排期".to_string(),
            "先小预算冷启动测点击率和加购率".to_string(),
            "根据首轮数据放大高转化渠道".to_string(),
        ],
        "ad_review" => vec![
            "回收活动期投放、转化和客服反馈数据".to_string(),
            "定位高 ROI 渠道与低效素材".to_string(),
            "沉淀保留、暂停和重做动作".to_string(),
            "形成下一轮预算与节奏建议".to_string(),
        ],
        _ => vec![
            "锁定活动商品、价格带和优惠机制".to_string(),
            "按渠道准备内容、素材和节奏".to_string(),
            "跟踪点击、加购、成交和退款信号".to_string(),
            "按表现及时调预算和策略".to_string(),
        ],
    };
    let metrics = match campaign_type {
        "launch" => vec![
            "点击率".to_string(),
            "加购率".to_string(),
            "首单成本".to_string(),
        ],
        "ad_review" => vec![
            "ROI".to_string(),
            "转化率".to_string(),
            "退款率".to_string(),
        ],
        _ => vec!["GMV".to_string(), "ROI".to_string(), "转化率".to_string()],
    };

    json!({
        "name": name,
        "campaignType": campaign_type,
        "goal": goal,
        "targetAudience": target_audience,
        "channels": channels,
        "strategy": strategy,
        "budget": budget,
        "executionSteps": execution_steps,
        "successMetrics": metrics,
        "guardrails": [
            "平台执行动作后续仍需审批后再触发",
            "草稿阶段先沉淀方案，不直接写真实平台",
            "素材、商品和工作区来源应保持可追溯"
        ]
    })
}

fn infer_status_query_type(metadata: &Map<String, Value>) -> &'static str {
    if metadata.contains_key("workspace_id") || metadata.contains_key("message.workspace_id") {
        "workspace_status"
    } else if metadata.contains_key("graph_id")
        || metadata.contains_key("node_id")
        || metadata.contains_key("task_id")
    {
        "task_status"
    } else if metadata.contains_key("session_id") || metadata.contains_key("message.session_id") {
        "session_status"
    } else {
        "system_health"
    }
}

fn visible_scope(context: &SkillContext) -> Value {
    json!({
        "session_id": context.session_id.clone(),
        "agent_id": context.agent_id.clone(),
        "workspace_id": metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id")),
        "run_id": metadata_string_value(&context.metadata, "request.workspace.run_id"),
        "conversation_id": metadata_string_value(&context.metadata, "request.context.conversation_id"),
        "user_id": metadata_string_value(&context.metadata, "message.user_id"),
        "visibility": "current_session_only",
    })
}

fn session_status_summary(context: &SkillContext) -> Value {
    let selected_roles = metadata_csv_list(&context.metadata, "request.strategy.selected_roles");
    let support_roles = metadata_csv_list(
        &context.metadata,
        "request.role_route.support_runtime_roles",
    );
    let refs = metadata_csv_list(&context.metadata, "request.context.refs");
    json!({
        "session_id": context.session_id.clone(),
        "current_agent_id": context.agent_id.clone(),
        "selected_roles": selected_roles,
        "primary_role": metadata_string_value(&context.metadata, "request.strategy.primary_role")
            .or_else(|| metadata_string_value(&context.metadata, "request.role_route.primary_runtime_role"))
            .or_else(|| metadata_string_value(&context.metadata, "role_runtime")),
        "support_roles": support_roles,
        "response_mode": metadata_string_value(&context.metadata, "request.strategy.response_mode"),
        "collaboration_mode": metadata_string_value(&context.metadata, "request.strategy.collaboration_mode"),
        "context_ref_count": refs.len(),
    })
}

fn worker_status_summary(context: &SkillContext) -> Value {
    let worker_role = metadata_string_value(&context.metadata, "role_runtime");
    let request_reason = metadata_string_value(&context.metadata, "request_reason");
    let attempt = metadata_u64_value(&context.metadata, "attempt").unwrap_or(1);
    json!({
        "worker_agent_id": context.agent_id.clone(),
        "runtime_role": worker_role,
        "status": if request_reason.is_some() { "executing_skill" } else { "ready" },
        "attempt": attempt,
        "request_reason": request_reason,
    })
}

fn task_status_summary(context: &SkillContext) -> Value {
    let upstream_roles = metadata_csv_list(&context.metadata, "assignment_input_roles");
    let input_count = metadata_u64_value(&context.metadata, "assignment_input_count").unwrap_or(0);
    let readiness = if input_count == 0 {
        "independent"
    } else if upstream_roles.is_empty() {
        "waiting_upstream_inputs"
    } else {
        "ready_with_upstream_inputs"
    };
    json!({
        "task_id": metadata_string_value(&context.metadata, "task_id"),
        "task_description": metadata_string_value(&context.metadata, "task_description"),
        "graph_id": metadata_string_value(&context.metadata, "graph_id"),
        "node_id": metadata_string_value(&context.metadata, "node_id"),
        "assignment_role": metadata_string_value(&context.metadata, "assignment_role"),
        "attempt": metadata_u64_value(&context.metadata, "attempt").unwrap_or(1),
        "input_count": input_count,
        "upstream_roles": upstream_roles,
        "readiness": readiness,
        "rework_instruction": metadata_string_value(&context.metadata, "assignment_rework_instruction"),
    })
}

fn workspace_status_summary(context: &SkillContext) -> Value {
    json!({
        "workspace_id": metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id")),
        "run_id": metadata_string_value(&context.metadata, "request.workspace.run_id"),
        "conversation_id": metadata_string_value(&context.metadata, "request.context.conversation_id"),
        "workspace_context_readiness": metadata_string_value(&context.metadata, "request.workspaceContext.readiness"),
        "workspace_context_pack_readiness": metadata_string_value(&context.metadata, "request.workspaceContextPack.readiness"),
    })
}

fn context_readiness_summary(context: &SkillContext) -> Value {
    json!({
        "product_context_loaded": metadata_non_empty(&context.metadata, "request.context.selected_summary"),
        "workspace_context_loaded": metadata_non_empty(&context.metadata, "request.workspaceContext.summary"),
        "workspace_context_pack_loaded": metadata_non_empty(&context.metadata, "request.workspaceContextPack.summary"),
        "product_context_refs": metadata_csv_list(&context.metadata, "request.context.refs"),
        "workspace_context_refs": metadata_csv_list(&context.metadata, "request.workspaceContext.refs"),
        "workspace_pack_refs": metadata_csv_list(&context.metadata, "request.workspaceContextPack.refs"),
    })
}

fn system_health_summary(context: &SkillContext) -> Value {
    let has_session = context.session_id.is_some();
    let has_agent = context.agent_id.is_some();
    let has_task = metadata_non_empty(&context.metadata, "task_id");
    let has_context = metadata_non_empty(&context.metadata, "request.context.selected_summary")
        || metadata_non_empty(&context.metadata, "request.workspaceContext.summary")
        || metadata_non_empty(&context.metadata, "request.workspaceContextPack.summary");
    let health = if has_session && has_agent && has_task {
        "active"
    } else if has_session || has_agent {
        "partial"
    } else {
        "limited_visibility"
    };
    json!({
        "health": health,
        "has_session": has_session,
        "has_agent": has_agent,
        "has_task": has_task,
        "has_context": has_context,
        "health_basis": "skill-context-visible-runtime-state",
    })
}

fn coordination_blockers(context: &SkillContext) -> Vec<String> {
    let mut blockers = Vec::new();
    if context.session_id.is_none() {
        blockers.push("当前 skill 调用没有 session_id，无法判断完整会话进度。".to_string());
    }
    if !metadata_non_empty(&context.metadata, "task_id") {
        blockers.push("当前上下文没有 task_id，无法判断具体任务节点状态。".to_string());
    }
    if !metadata_non_empty(&context.metadata, "request.workspace.run_id") {
        blockers.push("当前上下文没有 run_id，无法关联完整工作区运行轨迹。".to_string());
    }
    if !metadata_non_empty(&context.metadata, "request.workspaceContext.summary")
        && !metadata_non_empty(&context.metadata, "request.workspaceContextPack.summary")
    {
        blockers.push("当前工作区上下文尚未完整加载，可见来源有限。".to_string());
    }
    blockers
}

fn observability_limits(context: &SkillContext) -> Value {
    json!({
        "global_cluster_visibility": false,
        "database_live_query": false,
        "platform_adapter_probe": false,
        "other_sessions_visible": false,
        "current_session_scope": true,
        "metadata_keys": context.metadata.keys().cloned().collect::<Vec<_>>(),
    })
}

fn status_summary_line(
    query_type: &str,
    worker: &Value,
    task: &Value,
    workspace: &Value,
    health: &Value,
) -> String {
    let role = worker
        .get("runtime_role")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let task_id = task
        .get("task_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let workspace_id = workspace
        .get("workspace_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let health_name = health
        .get("health")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    format!(
        "状态查询已完成：query_type={query_type}，role={role}，task={task_id}，workspace={workspace_id}，health={health_name}"
    )
}

fn normalize_owner_role(value: Option<String>) -> String {
    let normalized = value
        .unwrap_or_else(|| "ops".to_string())
        .trim()
        .to_ascii_lowercase();
    match normalized.as_str() {
        "ops" | "data" | "service" | "design" | "accounting" | "engineering" | "web"
        | "creative" | "chat" => normalized,
        _ => "ops".to_string(),
    }
}

fn clamp_coordination_priority(value: i64) -> i64 {
    value.clamp(0, 10)
}

fn compact_text_for_task_title(value: &str) -> String {
    compact_task_text_value(value, 255)
}

fn compact_text_for_task_body(value: &str) -> String {
    compact_task_text_value(value, 2_000)
}

fn compact_task_text_value(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    trimmed.chars().take(max_chars).collect()
}

fn normalize_memory_key(value: &str) -> String {
    let mut normalized = value
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else if ch.is_ascii() {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();
    while normalized.contains("__") {
        normalized = normalized.replace("__", "_");
    }
    normalized = normalized.trim_matches('_').to_string();
    if normalized.is_empty() {
        "memory".to_string()
    } else {
        compact_task_text_value(&normalized, 120)
    }
}

fn normalize_memory_type(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "fact" | "事实" => "fact".to_string(),
        "decision" | "决策" => "decision".to_string(),
        "insight" | "洞察" => "insight".to_string(),
        "warning" | "风险" => "warning".to_string(),
        _ => "fact".to_string(),
    }
}

fn infer_memory_type_from_text(value: &str, metadata: &Map<String, Value>) -> &'static str {
    let text = value.to_ascii_lowercase();
    if text.contains("风险")
        || text.contains("warning")
        || metadata_string_value(metadata, "request_reason")
            .as_deref()
            .is_some_and(|reason| reason.contains("风险"))
    {
        "warning"
    } else if text.contains("决策") || text.contains("决定") || text.contains("decision") {
        "decision"
    } else if text.contains("洞察") || text.contains("insight") || text.contains("发现") {
        "insight"
    } else {
        "fact"
    }
}

fn metadata_string_value(metadata: &Map<String, Value>, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn metadata_u64_value(metadata: &Map<String, Value>, key: &str) -> Option<u64> {
    metadata.get(key).and_then(Value::as_u64)
}

fn metadata_non_empty(metadata: &Map<String, Value>, key: &str) -> bool {
    metadata_string_value(metadata, key).is_some()
}

fn metadata_csv_list(metadata: &Map<String, Value>, key: &str) -> Vec<String> {
    metadata_string_value(metadata, key)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{
        CoordinationCreateTask, CoordinationSkillChainPlanner, CoordinationStatusQuery,
        CoordinationTaskOrchestration, agent_matches, metadata_csv_list, orchestration_template,
        skill_chain_template,
    };
    use crate::skills::{Skill, SkillContext};
    use serde_json::{Value, json};

    #[tokio::test]
    async fn coordination_status_query_returns_native_session_scope_status() {
        let skill = CoordinationStatusQuery::new();
        let context = SkillContext::new()
            .with_session_id("session-1")
            .with_agent_id("worker.engineering")
            .with_metadata("message.user_id", json!("user-1"))
            .with_metadata("message.workspace_id", json!("workspace-1"))
            .with_metadata("task_id", json!("task-1"))
            .with_metadata("task_description", json!("查看当前任务状态"))
            .with_metadata("graph_id", json!("graph-1"))
            .with_metadata("node_id", json!("node-1"))
            .with_metadata("assignment_role", json!("engineering"))
            .with_metadata("attempt", json!(2))
            .with_metadata("assignment_input_count", json!(2))
            .with_metadata("assignment_input_roles", json!("data,ops"))
            .with_metadata("role_runtime", json!("engineering"))
            .with_metadata("request.workspace.run_id", json!("run-1"))
            .with_metadata("request.strategy.primary_role", json!("engineering"))
            .with_metadata(
                "request.strategy.selected_roles",
                json!("engineering,data,ops"),
            )
            .with_metadata("request.workspaceContext.readiness", json!("ready"))
            .with_metadata(
                "request.workspaceContext.summary",
                json!("已有工作区上下文"),
            )
            .with_metadata("request.workspaceContextPack.readiness", json!("partial"))
            .with_metadata("request.context.summary", json!("商品上下文已加载"))
            .with_metadata("request.context.refs", json!("product:1,knowledge:2"));

        let result = skill
            .run(json!({"query_type": "task_status"}), context)
            .await
            .expect("status query should succeed");

        assert_eq!(result.output.get("status"), Some(&json!("success")));
        assert_eq!(result.output.get("query_type"), Some(&json!("task_status")));
        assert_eq!(
            result.output.pointer("/task/task_id"),
            Some(&json!("task-1"))
        );
        assert_eq!(
            result.output.pointer("/task/upstream_roles"),
            Some(&json!(vec!["data", "ops"]))
        );
        assert_eq!(
            result.output.pointer("/workspace/run_id"),
            Some(&json!("run-1"))
        );
        assert_eq!(
            result.output.pointer("/system_health/health"),
            Some(&json!("active"))
        );
        assert_eq!(
            result.metadata.get("status_query_scope"),
            Some(&json!("session_local"))
        );
    }

    #[test]
    fn metadata_csv_list_splits_csv_metadata() {
        let mut metadata = serde_json::Map::new();
        metadata.insert("roles".to_string(), json!("ops,data,creative"));
        assert_eq!(
            metadata_csv_list(&metadata, "roles"),
            vec![
                "ops".to_string(),
                "data".to_string(),
                "creative".to_string()
            ]
        );
    }

    #[test]
    fn coordination_helpers_keep_expected_templates() {
        assert!(!agent_matches("运营和数据一起复盘").is_empty());
        assert!(!orchestration_template("新品上新").is_empty());
        assert!(!skill_chain_template("提升NPS满意度", "service", "analyze").is_empty());
    }

    #[tokio::test]
    async fn coordination_skill_chain_planner_returns_structured_plan() {
        let skill = CoordinationSkillChainPlanner::new();
        let result = skill
            .run(
                json!({
                    "goal": "咖啡新品促销活动需要规划投放和内容",
                    "role": "ops",
                    "action": "plan",
                    "skills_available": [
                        "query_store_metrics",
                        "ops_promo_planning",
                        "creative_seeding_copy"
                    ]
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill chain planner should succeed");

        assert_eq!(result.output.get("status"), Some(&json!("success")));
        assert_eq!(result.output.get("角色"), Some(&json!("ops")));
        assert_eq!(result.output.get("动作"), Some(&json!("plan")));
        assert_eq!(
            result.output.pointer("/推荐执行链/0/技能"),
            Some(&json!("query_store_metrics"))
        );
        assert_eq!(
            result.output.pointer("/推荐执行链/0/执行模式"),
            Some(&json!("adapter_required"))
        );
        assert_eq!(
            result.output.pointer("/推荐执行链/0/阶段"),
            Some(&json!("data_loading"))
        );
        assert!(
            result
                .output
                .pointer("/adapter提示")
                .and_then(Value::as_array)
                .is_some_and(|items| items.contains(&json!("query_store_metrics")))
        );
        assert!(
            result
                .output
                .pointer("/不可用技能")
                .and_then(Value::as_array)
                .is_some_and(|items| items.contains(&json!("coordination_create_campaign")))
        );
        assert_eq!(
            result.metadata.get("skill_chain_has_adapter_steps"),
            Some(&json!(true))
        );
    }

    #[tokio::test]
    async fn coordination_skill_chain_planner_marks_approval_steps() {
        let skill = CoordinationSkillChainPlanner::new();
        let result = skill
            .run(
                json!({
                    "goal": "检查 SLA 和 P99 延迟，确认平台接口健康状态",
                    "role": "engineering",
                    "action": "analyze"
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill chain planner should succeed");

        assert!(
            result
                .output
                .pointer("/审批提示")
                .and_then(Value::as_array)
                .is_some_and(|items| items.contains(&json!("engineering_system_check")))
        );
        assert_eq!(
            result.metadata.get("skill_chain_requires_approval"),
            Some(&json!(true))
        );
    }

    #[tokio::test]
    async fn coordination_create_task_returns_native_task_creation_payload() {
        let skill = CoordinationCreateTask::new();
        let context = SkillContext::new()
            .with_session_id("session-1")
            .with_agent_id("worker.ops")
            .with_metadata("message.workspace_id", json!("workspace-1"))
            .with_metadata("role_runtime", json!("ops"));

        let result = skill
            .run(
                json!({
                    "title": "整理五一咖啡活动复盘任务",
                    "description": "需要拆出投放、客服、商品三个子方向",
                    "owner_role": "ops",
                    "priority": 2,
                    "acceptance_criteria": "输出复盘结论和后续动作"
                }),
                context,
            )
            .await
            .expect("create task should succeed");

        assert_eq!(result.output.get("status"), Some(&json!("success")));
        assert_eq!(
            result.output.pointer("/task_creation/should_persist"),
            Some(&json!(true))
        );
        assert_eq!(
            result.output.pointer("/task_creation/title"),
            Some(&json!("整理五一咖啡活动复盘任务"))
        );
        assert_eq!(
            result.output.pointer("/task_creation/workspace_id"),
            Some(&json!("workspace-1"))
        );
        assert_eq!(
            result.metadata.get("task_creation_kind"),
            Some(&json!("workspace_task"))
        );
    }

    #[tokio::test]
    async fn coordination_create_campaign_returns_native_campaign_draft_payload() {
        let skill = super::CoordinationCreateCampaign::new();
        let context = SkillContext::new()
            .with_session_id("session-1")
            .with_agent_id("worker.ops")
            .with_metadata("message.workspace_id", json!("workspace-1"))
            .with_metadata("role_runtime", json!("ops"))
            .with_metadata("context.product_ids", json!("product-1"));

        let result = skill
            .run(
                json!({
                    "name": "五一咖啡促销活动",
                    "campaign_type": "promotion",
                    "budget": 5000,
                    "goal": "提升五一周末转化率",
                    "channels": ["抖音", "淘宝"],
                    "strategy": "先小预算测款，再放量高转化素材"
                }),
                context,
            )
            .await
            .expect("create campaign should succeed");

        assert_eq!(result.output.get("status"), Some(&json!("success")));
        assert_eq!(
            result.output.pointer("/campaign_creation/should_persist"),
            Some(&json!(true))
        );
        assert_eq!(
            result.output.pointer("/campaign_creation/product_id"),
            Some(&json!("product-1"))
        );
        assert_eq!(
            result.output.pointer("/campaign_creation/workspace_id"),
            Some(&json!("workspace-1"))
        );
        assert_eq!(
            result.output.pointer("/campaign_creation/campaign_type"),
            Some(&json!("promotion"))
        );
        assert_eq!(
            result.output.pointer("/campaign_creation/channels"),
            Some(&json!(vec!["douyin", "taobao"]))
        );
        assert_eq!(
            result.metadata.get("campaign_creation_kind"),
            Some(&json!("campaign_draft"))
        );
    }

    #[tokio::test]
    async fn coordination_save_memory_returns_native_memory_payload() {
        let skill = super::CoordinationSaveMemory::new();
        let context = SkillContext::new()
            .with_session_id("session-1")
            .with_agent_id("worker.ops")
            .with_metadata("message.workspace_id", json!("workspace-1"))
            .with_metadata("role_runtime", json!("ops"));

        let result = skill
            .run(
                json!({
                    "key": "Pricing Strategy 2024",
                    "content": "本品定价区间保持在 159-179 元，避免低于核心竞品均价。",
                    "memory_type": "decision"
                }),
                context,
            )
            .await
            .expect("save memory should succeed");

        assert_eq!(result.output.get("status"), Some(&json!("success")));
        assert_eq!(
            result.output.pointer("/memory_save/should_persist"),
            Some(&json!(true))
        );
        assert_eq!(
            result.output.pointer("/memory_save/key"),
            Some(&json!("pricing_strategy_2024"))
        );
        assert_eq!(
            result.output.pointer("/memory_save/workspace_id"),
            Some(&json!("workspace-1"))
        );
        assert_eq!(
            result.metadata.get("memory_save_kind"),
            Some(&json!("workspace_learning_record"))
        );
    }
}
