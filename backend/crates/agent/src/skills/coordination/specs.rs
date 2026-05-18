use serde_json::json;

use crate::skills::{SkillCategory, SkillInputField, SkillPriority, SkillSpec};

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
        "product_ref",
        "新产品商品引用，如 product:{uuid}；不传时优先使用当前上下文商品引用",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "旧商品ID兼容字段（UUID字符串，可选；不传时优先使用当前上下文商品）",
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
