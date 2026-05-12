use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillInputField, SkillOutcome, SkillPriority, SkillResult,
    SkillSpec, object_params, optional_bool_param, optional_f64_param, optional_i64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("engineering".to_owned())
}

pub fn specs() -> Vec<SkillSpec> {
    vec![
        engineering_arch_review_spec(),
        engineering_bug_analysis_spec(),
        engineering_perf_optimize_spec(),
        engineering_tech_selection_spec(),
        engineering_system_check_spec(),
        engineering_sla_monitor_spec(),
    ]
}

pub fn engineering_arch_review_spec() -> SkillSpec {
    native_spec(
        "engineering_arch_review",
        "架构评估",
        "AI深度评审系统架构，输出5维评分/核心风险/分级改造方案/大促备战清单",
        vec![
            SkillInputField::required(
                "system_type",
                "系统类型：电商平台/ERP/CRM/数据平台/小程序",
                json!({"type": "string"}),
            ),
            SkillInputField::optional("daily_orders", "日均订单量", json!({"type": "integer"})),
            SkillInputField::optional(
                "tech_stack",
                "技术栈列表（如 Python/MySQL/Redis/Nginx）",
                json!({"type": "array", "items": {"type": "string"}}),
            ),
            SkillInputField::optional(
                "pain_points",
                "当前主要痛点",
                json!({"type": "array", "items": {"type": "string"}}),
            ),
            SkillInputField::optional(
                "current_metrics",
                "当前系统指标，如 {响应时间: '200ms', QPS: 500}",
                json!({"type": "object"}),
            ),
        ],
    )
}

pub fn engineering_bug_analysis_spec() -> SkillSpec {
    native_spec(
        "engineering_bug_analysis",
        "Bug排查方案",
        "AI深度诊断Bug，输出根因分析/10步排查流程/修复代码示例/临时止血方案/预防措施",
        vec![
            SkillInputField::required(
                "error_type",
                "错误类型：接口超时/数据异常/页面白屏/支付失败/库存不一致/OOM/死锁等",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "error_message",
                "完整错误信息或日志片段",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "frequency",
                "频率：偶发/频繁/必现",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "tech_stack",
                "技术栈，如 Python/FastAPI/MySQL",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "context",
                "发生场景（如大促/特定操作/定时任务）",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn engineering_perf_optimize_spec() -> SkillSpec {
    native_spec(
        "engineering_perf_optimize",
        "性能优化方案",
        "AI生成具体性能优化方案，含代码示例/配置参数/分阶段实施计划/压测脚本",
        vec![
            SkillInputField::required(
                "bottleneck",
                "瓶颈类型：数据库/接口/前端/缓存/并发/网络",
                json!({"type": "string"}),
            ),
            SkillInputField::optional("current_qps", "当前QPS/TPS", json!({"type": "number"})),
            SkillInputField::optional("target_qps", "目标QPS/TPS", json!({"type": "number"})),
            SkillInputField::optional("tech_stack", "技术栈", json!({"type": "string"})),
            SkillInputField::optional(
                "metrics",
                "当前系统指标，如 {P99延迟: '1200ms', CPU使用率: '85%'}",
                json!({"type": "object"}),
            ),
        ],
    )
}

pub fn engineering_tech_selection_spec() -> SkillSpec {
    native_spec(
        "engineering_tech_selection",
        "技术选型",
        "AI生成技术选型深度报告，含候选方案对比/迁移方案/POC验证清单",
        vec![
            SkillInputField::required(
                "domain",
                "选型领域：Web框架/数据库/消息队列/搜索引擎/缓存/容器编排/监控",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "requirements",
                "核心需求列表",
                json!({"type": "array", "items": {"type": "string"}}),
            ),
            SkillInputField::optional(
                "constraints",
                "约束条件（预算/团队技能/合规/时间）",
                json!({"type": "array", "items": {"type": "string"}}),
            ),
            SkillInputField::optional("team_size", "团队规模", json!({"type": "integer"})),
            SkillInputField::optional(
                "current_stack",
                "现有技术栈（迁移评估用）",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn engineering_system_check_spec() -> SkillSpec {
    SkillSpec::new(
        "engineering_system_check",
        "平台接口健康检查",
        "检查用户已配置的电商平台API连接状态，诊断同步和接口问题",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::optional(
        "test_live",
        "是否实际调用平台API测试连通性（默认仅检查配置）",
        json!({"type": "boolean"}),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "指定平台（留空则检查全部已配置平台）",
        json!({"type": "string"}),
    ))
    .with_tag("deferred")
    .with_tag("pending_approval")
}

pub fn engineering_sla_monitor_spec() -> SkillSpec {
    SkillSpec::new(
        "engineering_sla_monitor",
        "SLA监控",
        "P50/P90/P99延迟分位数分析 + 可用性SLO监控 + 错误预算消耗率计算",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "latency_samples",
        "延迟样本列表（毫秒），至少10个数据点",
        json!({"type": "array", "items": {"type": "number"}}),
    ))
    .with_input(SkillInputField::optional(
        "p50_ms",
        "P50延迟（毫秒）",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "p90_ms",
        "P90延迟（毫秒）",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "p99_ms",
        "P99延迟（毫秒）",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "total_requests",
        "统计周期内总请求数",
        json!({"type": "integer"}),
    ))
    .with_input(SkillInputField::optional(
        "error_count",
        "错误请求数",
        json!({"type": "integer"}),
    ))
    .with_input(SkillInputField::optional(
        "downtime_minutes",
        "本月宕机分钟数",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "period_days",
        "统计周期天数，默认30天",
        json!({"type": "integer"}),
    ))
    .with_input(SkillInputField::optional(
        "slo_availability",
        "可用性SLO目标（%），默认99.9",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "slo_p99_ms",
        "P99延迟SLO目标（毫秒），默认2000",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "system_name",
        "系统名称",
        json!({"type": "string"}),
    ))
    .with_tag("deterministic")
}

pub struct EngineeringSlaMonitor {
    spec: SkillSpec,
}

fn native_spec(id: &str, name: &str, description: &str, inputs: Vec<SkillInputField>) -> SkillSpec {
    inputs.into_iter().fold(
        SkillSpec::new(id, name, description)
            .with_category(category())
            .with_priority(SkillPriority::High)
            .with_tag("deterministic")
            .with_tag("p1")
            .with_tag("analysis_engine"),
        SkillSpec::with_input,
    )
}

macro_rules! native_engineering_skill {
    ($type_name:ident, $spec_fn:ident, required [$($required:literal),*], defaults {$($key:literal => $value:expr),* $(,)?}) => {
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
                let mut normalized = Map::new();
                $(normalized.insert($key.to_owned(), json!($value(&params)?));)*
                let mut output = engineering_native_output(&self.spec.id, &normalized);
                normalize_engineering_output(&self.spec.id, &params, &mut output);
                Ok(SkillOutcome::new(output).with_summary(engineering_summary(&self.spec.id)))
            }
        }
    };
}

native_engineering_skill!(
    EngineeringArchReview,
    engineering_arch_review_spec,
    required ["system_type"],
    defaults {
        "system_type" => |p: &Value| string_param(p, "system_type").map(Some),
        "daily_orders" => |p: &Value| Ok(optional_i64_param(p, "daily_orders")?.unwrap_or(1000)),
        "tech_stack" => |p: &Value| Ok(optional_string_vec_param(p, "tech_stack")?.unwrap_or_default()),
        "pain_points" => |p: &Value| Ok(optional_string_vec_param(p, "pain_points")?.unwrap_or_default()),
        "current_metrics" => |p: &Value| Ok(p.get("current_metrics").cloned().unwrap_or_else(|| json!({}))),
    }
);
native_engineering_skill!(
    EngineeringBugAnalysis,
    engineering_bug_analysis_spec,
    required ["error_type"],
    defaults {
        "error_type" => |p: &Value| string_param(p, "error_type").map(Some),
        "error_message" => |p: &Value| optional_string_param(p, "error_message"),
        "frequency" => |p: &Value| Ok(optional_string_param(p, "frequency")?.unwrap_or_else(|| "偶发".to_owned())),
        "tech_stack" => |p: &Value| optional_string_param(p, "tech_stack"),
        "context" => |p: &Value| optional_string_param(p, "context"),
    }
);
native_engineering_skill!(
    EngineeringPerfOptimize,
    engineering_perf_optimize_spec,
    required ["bottleneck"],
    defaults {
        "bottleneck" => |p: &Value| string_param(p, "bottleneck").map(Some),
        "current_qps" => |p: &Value| Ok(optional_f64_param(p, "current_qps")?.unwrap_or(500.0)),
        "target_qps" => |p: &Value| Ok(optional_f64_param(p, "target_qps")?.unwrap_or(2000.0)),
        "tech_stack" => |p: &Value| optional_string_param(p, "tech_stack"),
        "metrics" => |p: &Value| Ok(p.get("metrics").cloned().unwrap_or_else(|| json!({}))),
    }
);
native_engineering_skill!(
    EngineeringTechSelection,
    engineering_tech_selection_spec,
    required ["domain"],
    defaults {
        "domain" => |p: &Value| string_param(p, "domain").map(Some),
        "requirements" => |p: &Value| Ok(optional_string_vec_param(p, "requirements")?.unwrap_or_default()),
        "constraints" => |p: &Value| Ok(optional_string_vec_param(p, "constraints")?.unwrap_or_default()),
        "team_size" => |p: &Value| Ok(optional_i64_param(p, "team_size")?.unwrap_or(5)),
        "current_stack" => |p: &Value| optional_string_param(p, "current_stack"),
    }
);

fn normalize_engineering_output(skill_id: &str, input: &Value, output: &mut Value) {
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
        json!("本阶段基于用户输入和内置工程分析规则生成，不读取真实代码仓库、日志、APM、监控系统、外部搜索或实时性能基准。"),
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

fn engineering_native_output(skill_id: &str, input: &Map<String, Value>) -> Value {
    match skill_id {
        "engineering_arch_review" => arch_review_output(input),
        "engineering_bug_analysis" => bug_analysis_output(input),
        "engineering_perf_optimize" => perf_optimize_output(input),
        "engineering_tech_selection" => tech_selection_output(input),
        _ => json!({"has_data": false, "error": format!("unknown engineering skill: {skill_id}")}),
    }
}

fn engineering_summary(skill_id: &str) -> &'static str {
    match skill_id {
        "engineering_arch_review" => "工程架构评估报告已生成",
        "engineering_bug_analysis" => "Bug排查方案已生成",
        "engineering_perf_optimize" => "性能优化方案已生成",
        "engineering_tech_selection" => "技术选型报告已生成",
        "engineering_sla_monitor" => "SLA监控分析已完成",
        _ => "工程分析报告已生成",
    }
}

fn arch_review_output(input: &Map<String, Value>) -> Value {
    let system_type = str_field(input, "system_type", "电商系统");
    let daily_orders = i64_field(input, "daily_orders").unwrap_or(1000).max(0);
    let tech_stack = string_vec_field(input, "tech_stack");
    let pain_points = string_vec_field(input, "pain_points");
    let current_metrics = input
        .get("current_metrics")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let scale_level = if daily_orders >= 100_000 {
        "高并发大规模"
    } else if daily_orders >= 10_000 {
        "中高规模"
    } else {
        "成长规模"
    };
    let risk_level = architecture_risk_level(daily_orders, &pain_points, &current_metrics);

    json!({
        "执行模式": "native_engineering_arch_review",
        "数据来源": "用户输入 + 内置工程评审规则",
        "系统类型": system_type,
        "日均订单量": daily_orders,
        "规模判断": scale_level,
        "技术栈": if tech_stack.is_empty() { vec!["未提供，按通用电商架构评估".to_owned()] } else { tech_stack },
        "当前痛点": if pain_points.is_empty() { vec!["未提供，按扩展性、可靠性和可观测性默认检查".to_owned()] } else { pain_points },
        "当前指标": current_metrics,
        "五维评分": [
            {"维度": "扩展性", "评分": arch_score(daily_orders, "scale"), "判断": "订单量、横向扩展和热点隔离能力"},
            {"维度": "可靠性", "评分": arch_score(daily_orders, "reliability"), "判断": "失败隔离、重试、幂等和降级能力"},
            {"维度": "数据一致性", "评分": arch_score(daily_orders, "data"), "判断": "库存、订单、支付等关键链路一致性"},
            {"维度": "可观测性", "评分": arch_score(daily_orders, "observability"), "判断": "日志、指标、链路追踪和告警闭环"},
            {"维度": "交付治理", "评分": arch_score(daily_orders, "delivery"), "判断": "发布、回滚、压测和变更审批"}
        ],
        "核心风险": [
            {"等级": risk_level, "风险": "大促峰值下订单、库存、支付链路可能被同步依赖放大", "建议": "拆出关键链路限流、幂等、队列削峰和降级开关"},
            {"等级": "中", "风险": "缺少统一观测会让故障定位依赖人工经验", "建议": "补齐接口维度指标、业务埋点和错误码聚合"},
            {"等级": "中", "风险": "复杂变更若无灰度和回滚策略，容易扩大影响面", "建议": "按接口、店铺、用户或流量比例灰度"}
        ],
        "分级改造方案": [
            {"阶段": "P0", "周期": "1-2周", "动作": ["梳理订单/库存/支付关键链路", "补幂等键和超时重试边界", "建立P95/P99和错误率告警"]},
            {"阶段": "P1", "周期": "2-4周", "动作": ["热点接口缓存和限流", "任务队列削峰", "慢查询和大事务治理"]},
            {"阶段": "P2", "周期": "1-2月", "动作": ["服务边界重整", "压测基线常态化", "容量规划和大促预案自动化"]}
        ],
        "大促备战清单": ["全链路压测", "库存扣减幂等校验", "支付回调补偿", "核心接口降级预案", "告警值班和回滚演练"]
    })
}

fn bug_analysis_output(input: &Map<String, Value>) -> Value {
    let error_type = str_field(input, "error_type", "未知错误");
    let error_message = opt_str_field(input, "error_message");
    let frequency = str_field(input, "frequency", "偶发");
    let tech_stack = opt_str_field(input, "tech_stack").unwrap_or_else(|| "未提供".to_owned());
    let context = opt_str_field(input, "context").unwrap_or_else(|| "未提供".to_owned());

    json!({
        "执行模式": "native_engineering_bug_analysis",
        "数据来源": "用户输入 + 内置故障排查规则",
        "错误类型": error_type,
        "错误信息": error_message,
        "发生频率": frequency,
        "技术栈": tech_stack,
        "发生场景": context,
        "根因假设": bug_hypotheses(&error_type),
        "10步排查流程": [
            "确认影响面：用户、店铺、平台、接口、时间段",
            "锁定首次出现时间，并对齐发布、配置、流量和平台变更",
            "提取同 request_id / trace_id 的完整链路日志",
            "检查上游入参、鉴权、限流和网关状态",
            "检查下游数据库、缓存、队列、第三方接口耗时和错误码",
            "复现最小输入，区分数据问题、代码问题和外部依赖问题",
            "检查幂等、重试、超时和事务边界",
            "评估是否需要降级、回滚、熔断或手动补偿",
            "补充监控指标和告警规则",
            "沉淀根因、修复、验证和预防清单"
        ],
        "临时止血方案": bug_stopgap(&error_type),
        "长期修复方案": bug_fix_plan(&error_type),
        "验证清单": ["复现用例通过", "异常路径单测覆盖", "关键指标恢复", "日志可定位", "补偿任务可追踪"],
        "预防措施": ["发布前回归高风险链路", "错误码标准化", "慢接口和失败率告警", "高风险动作进入审批或灰度"]
    })
}

fn perf_optimize_output(input: &Map<String, Value>) -> Value {
    let bottleneck = str_field(input, "bottleneck", "接口");
    let current_qps = f64_field(input, "current_qps").unwrap_or(500.0).max(0.0);
    let target_qps = f64_field(input, "target_qps").unwrap_or(2000.0).max(1.0);
    let tech_stack = opt_str_field(input, "tech_stack").unwrap_or_else(|| "未提供".to_owned());
    let metrics = input.get("metrics").cloned().unwrap_or_else(|| json!({}));
    let capacity_gap = round1(target_qps / current_qps.max(1.0));

    json!({
        "执行模式": "native_engineering_perf_optimize",
        "数据来源": "用户输入 + 内置性能治理规则",
        "瓶颈类型": bottleneck,
        "当前QPS": current_qps,
        "目标QPS": target_qps,
        "容量差距倍数": capacity_gap,
        "技术栈": tech_stack,
        "当前指标": metrics,
        "优先诊断": perf_findings(&bottleneck, capacity_gap),
        "优化方案": perf_actions(&bottleneck),
        "配置与代码方向": [
            {"方向": "超时与重试", "建议": "接口超时、重试次数和退避策略必须按下游SLA配置，避免重试风暴"},
            {"方向": "缓存", "建议": "只缓存可接受短暂不一致的数据，并明确失效策略和击穿保护"},
            {"方向": "数据库", "建议": "先看慢查询、索引、分页和事务时长，再考虑分库分表"},
            {"方向": "队列", "建议": "写链路削峰时要补幂等键、死信队列和补偿任务"}
        ],
        "分阶段实施计划": [
            {"阶段": "P0", "目标": "止血", "动作": ["加限流和降级", "修复最慢接口", "补核心告警"]},
            {"阶段": "P1", "目标": "提升容量", "动作": ["缓存热点读", "拆异步任务", "治理慢SQL和大对象"]},
            {"阶段": "P2", "目标": "稳定增长", "动作": ["常态化压测", "容量模型", "自动扩缩容和演练"]}
        ],
        "压测脚本骨架": {
            "工具": "k6 或 wrk",
            "场景": ["基线流量", "目标流量", "峰值2倍", "下游降级"],
            "通过标准": ["P99低于SLO", "错误率低于阈值", "资源使用率不过载", "无数据不一致"]
        }
    })
}

fn tech_selection_output(input: &Map<String, Value>) -> Value {
    let domain = str_field(input, "domain", "Web框架");
    let requirements = string_vec_field(input, "requirements");
    let constraints = string_vec_field(input, "constraints");
    let team_size = i64_field(input, "team_size").unwrap_or(5).max(1);
    let current_stack = opt_str_field(input, "current_stack");
    let candidates = tech_candidates(&domain);

    json!({
        "执行模式": "native_engineering_tech_selection",
        "数据来源": "用户输入 + 内置选型评估规则；不包含实时外部 benchmark",
        "选型领域": domain,
        "核心需求": if requirements.is_empty() { vec!["稳定性".to_owned(), "团队可维护性".to_owned(), "生态成熟度".to_owned()] } else { requirements },
        "约束条件": if constraints.is_empty() { vec!["控制迁移成本".to_owned(), "避免过早复杂化".to_owned()] } else { constraints },
        "团队规模": team_size,
        "现有技术栈": current_stack,
        "候选方案": candidates,
        "评分维度": [
            {"维度": "业务匹配", "权重": "30%", "说明": "是否覆盖当前业务模式和未来增长"},
            {"维度": "团队掌握", "权重": "25%", "说明": "团队学习和招聘成本"},
            {"维度": "生态成熟", "权重": "20%", "说明": "社区、插件、文档和长期维护"},
            {"维度": "迁移风险", "权重": "15%", "说明": "双写、兼容、回滚和数据迁移复杂度"},
            {"维度": "运行成本", "权重": "10%", "说明": "部署、监控、扩容和故障处理成本"}
        ],
        "推荐结论": tech_recommendation(&domain, team_size),
        "POC验证清单": ["核心用例能跑通", "压测达到目标SLO", "故障注入可恢复", "监控指标完整", "迁移回滚路径清晰"],
        "迁移路径": [
            {"阶段": "评估", "动作": "用最小业务闭环做POC，不改生产主链路"},
            {"阶段": "灰度", "动作": "按低风险流量或内部店铺接入"},
            {"阶段": "替换", "动作": "保留旧链路回滚窗口，逐步扩大覆盖"},
            {"阶段": "治理", "动作": "沉淀模板、规范、监控和事故预案"}
        ]
    })
}

fn architecture_risk_level(
    daily_orders: i64,
    pain_points: &[String],
    current_metrics: &Value,
) -> &'static str {
    let pain_text = pain_points.join(" ");
    let metrics_text = current_metrics.to_string();
    if daily_orders >= 100_000
        || pain_text.contains("不一致")
        || pain_text.contains("超时")
        || metrics_text.contains("1200")
        || metrics_text.contains("高")
    {
        "高"
    } else if daily_orders >= 10_000
        || !pain_points.is_empty()
        || current_metrics
            .as_object()
            .is_some_and(|value| !value.is_empty())
    {
        "中"
    } else {
        "低"
    }
}

fn arch_score(daily_orders: i64, dimension: &str) -> i64 {
    let base = if daily_orders >= 100_000 {
        62
    } else if daily_orders >= 10_000 {
        72
    } else {
        82
    };
    let adjustment = match dimension {
        "scale" => -2,
        "reliability" => 0,
        "data" => 3,
        "observability" => -5,
        "delivery" => 2,
        _ => 0,
    };
    (base + adjustment).clamp(45_i64, 92_i64)
}

fn bug_hypotheses(error_type: &str) -> Vec<Value> {
    if error_type.contains("超时") {
        vec![
            json!({"可能根因": "下游接口或数据库慢查询导致请求堆积", "验证方式": "按 trace_id 比对网关、服务、DB 和第三方耗时"}),
            json!({"可能根因": "重试放大或连接池耗尽", "验证方式": "检查连接池等待时间、重试次数和并发峰值"}),
            json!({"可能根因": "热点缓存击穿", "验证方式": "检查缓存命中率和同一 key 请求集中度"}),
        ]
    } else if error_type.contains("库存") || error_type.contains("不一致") {
        vec![
            json!({"可能根因": "扣减链路缺少幂等或事务边界不完整", "验证方式": "核对订单、库存流水和补偿任务"}),
            json!({"可能根因": "异步同步延迟或失败未补偿", "验证方式": "检查消息队列积压、死信和消费失败"}),
            json!({"可能根因": "并发写入缺少版本号或锁策略", "验证方式": "复现并发扣减和取消订单场景"}),
        ]
    } else if error_type.contains("白屏") || error_type.contains("页面") {
        vec![
            json!({"可能根因": "前端资源加载失败或接口响应结构变更", "验证方式": "检查浏览器控制台、静态资源和接口 schema"}),
            json!({"可能根因": "登录态刷新或路由守卫异常", "验证方式": "用有效/过期 token 分别复现"}),
            json!({"可能根因": "运行时异常未被错误边界捕获", "验证方式": "定位 sourcemap 和错误堆栈"}),
        ]
    } else {
        vec![
            json!({"可能根因": "发布、配置、数据或外部依赖变化引发异常", "验证方式": "按时间线关联变更记录和错误曲线"}),
            json!({"可能根因": "边界输入缺少校验", "验证方式": "抽样失败请求并构造最小复现"}),
            json!({"可能根因": "异常路径缺少降级或补偿", "验证方式": "故障注入并检查恢复流程"}),
        ]
    }
}

fn bug_stopgap(error_type: &str) -> Vec<String> {
    if error_type.contains("超时") {
        vec![
            "临时提高非关键链路降级比例".to_owned(),
            "对热点接口加限流和缓存保护".to_owned(),
            "暂停会放大流量的批量任务".to_owned(),
        ]
    } else if error_type.contains("库存") || error_type.contains("支付") {
        vec![
            "冻结高风险写操作或切到人工复核".to_owned(),
            "开启补偿任务并记录受影响订单".to_owned(),
            "对外展示明确处理中状态，避免重复提交".to_owned(),
        ]
    } else {
        vec![
            "回滚最近高风险发布或配置".to_owned(),
            "缩小影响入口并保留核心链路".to_owned(),
            "补充临时监控看板和人工巡检".to_owned(),
        ]
    }
}

fn bug_fix_plan(error_type: &str) -> Vec<Value> {
    if error_type.contains("超时") {
        vec![
            json!({"层级": "代码", "动作": "拆分慢接口、减少串行下游调用、明确超时边界"}),
            json!({"层级": "数据", "动作": "治理慢查询、补索引、限制大分页和大事务"}),
            json!({"层级": "治理", "动作": "建立P95/P99、连接池等待和下游错误率告警"}),
        ]
    } else if error_type.contains("库存") || error_type.contains("不一致") {
        vec![
            json!({"层级": "代码", "动作": "引入幂等键、版本号或库存流水校验"}),
            json!({"层级": "任务", "动作": "补偿任务必须可重试、可审计、可人工介入"}),
            json!({"层级": "测试", "动作": "增加并发扣减、取消、退款和失败回调用例"}),
        ]
    } else {
        vec![
            json!({"层级": "代码", "动作": "补输入校验、错误边界和可恢复分支"}),
            json!({"层级": "发布", "动作": "新增灰度、回滚和配置变更审计"}),
            json!({"层级": "测试", "动作": "把本次故障最小复现沉淀成回归用例"}),
        ]
    }
}

fn perf_findings(bottleneck: &str, capacity_gap: f64) -> Vec<String> {
    let mut findings = Vec::new();
    if capacity_gap >= 3.0 {
        findings.push(format!(
            "目标吞吐约为当前 {capacity_gap} 倍，需要先做容量模型和压测，不能只靠加机器"
        ));
    }
    if bottleneck.contains("数据库") {
        findings.push("数据库瓶颈优先检查慢查询、索引、事务时长、连接池和热点行竞争".to_owned());
    } else if bottleneck.contains("缓存") {
        findings.push("缓存瓶颈优先检查命中率、热 key、击穿保护和过期策略".to_owned());
    } else if bottleneck.contains("前端") {
        findings.push("前端瓶颈优先检查首屏资源、接口瀑布流、图片体积和运行时错误".to_owned());
    } else if bottleneck.contains("并发") {
        findings.push("并发瓶颈优先检查锁、队列积压、连接池、线程池和背压策略".to_owned());
    } else {
        findings.push("接口瓶颈优先按网关、应用、下游、数据库、缓存分段拆耗时".to_owned());
    }
    findings
}

fn perf_actions(bottleneck: &str) -> Vec<Value> {
    if bottleneck.contains("数据库") {
        vec![
            json!({"优先级": "P0", "动作": "抓取TOP慢查询并补索引/改分页", "预期": "降低P95/P99和连接占用"}),
            json!({"优先级": "P1", "动作": "拆长事务和批量写入", "预期": "降低锁等待和回滚成本"}),
            json!({"优先级": "P2", "动作": "建立读写分离或冷热数据归档", "预期": "提升长期容量"}),
        ]
    } else if bottleneck.contains("缓存") {
        vec![
            json!({"优先级": "P0", "动作": "为热点key增加互斥重建和随机过期", "预期": "避免击穿"}),
            json!({"优先级": "P1", "动作": "按业务容忍度设置缓存一致性策略", "预期": "控制脏读风险"}),
            json!({"优先级": "P2", "动作": "监控命中率、热key和内存淘汰", "预期": "容量可观测"}),
        ]
    } else if bottleneck.contains("前端") {
        vec![
            json!({"优先级": "P0", "动作": "减少首屏阻塞资源和大图体积", "预期": "改善首屏加载"}),
            json!({"优先级": "P1", "动作": "合并关键接口并延迟非首屏请求", "预期": "减少等待链路"}),
            json!({"优先级": "P2", "动作": "接入前端错误和性能指标", "预期": "持续观测体验"}),
        ]
    } else {
        vec![
            json!({"优先级": "P0", "动作": "分段打点并定位最慢依赖", "预期": "明确瓶颈归属"}),
            json!({"优先级": "P1", "动作": "缓存只读热点、异步化非关键写入", "预期": "减少主链路耗时"}),
            json!({"优先级": "P2", "动作": "压测并建立容量基线", "预期": "预测大促风险"}),
        ]
    }
}

fn tech_candidates(domain: &str) -> Vec<Value> {
    if domain.contains("数据库") {
        vec![
            json!({"方案": "PostgreSQL", "适用": "事务和复杂查询", "风险": "超大规模写入需容量治理"}),
            json!({"方案": "MySQL", "适用": "团队熟悉、生态成熟", "风险": "复杂分析能力相对有限"}),
            json!({"方案": "ClickHouse", "适用": "高吞吐分析查询", "风险": "不适合作为订单事务主库"}),
        ]
    } else if domain.contains("消息") || domain.contains("队列") {
        vec![
            json!({"方案": "RabbitMQ", "适用": "业务消息和路由灵活", "风险": "大吞吐日志流不是强项"}),
            json!({"方案": "Kafka", "适用": "高吞吐事件流和日志", "风险": "运维和消费语义复杂"}),
            json!({"方案": "Redis Stream", "适用": "轻量队列和现有Redis体系", "风险": "复杂消费治理能力有限"}),
        ]
    } else if domain.contains("搜索") {
        vec![
            json!({"方案": "Elasticsearch / OpenSearch", "适用": "全文检索和商品搜索", "风险": "集群调优成本高"}),
            json!({"方案": "Meilisearch", "适用": "轻量搜索和快速落地", "风险": "复杂电商搜索能力有限"}),
            json!({"方案": "数据库全文索引", "适用": "早期低复杂度场景", "风险": "扩展和相关性控制有限"}),
        ]
    } else if domain.contains("监控") {
        vec![
            json!({"方案": "Prometheus + Grafana", "适用": "指标监控和告警", "风险": "链路追踪需另接"}),
            json!({"方案": "OpenTelemetry", "适用": "统一日志/指标/追踪采集", "风险": "落地需要规范和采样策略"}),
            json!({"方案": "SaaS APM", "适用": "快速可用", "风险": "成本和数据合规需评估"}),
        ]
    } else {
        vec![
            json!({"方案": "延续现有主栈", "适用": "风险低、团队熟悉", "风险": "可能保留历史包袱"}),
            json!({"方案": "渐进引入新框架", "适用": "局部能力升级", "风险": "双栈维护成本"}),
            json!({"方案": "重构替换", "适用": "旧系统不可维护", "风险": "迁移周期和回滚复杂"}),
        ]
    }
}

fn tech_recommendation(domain: &str, team_size: i64) -> String {
    if team_size <= 3 {
        format!("{domain} 选型建议优先选择团队已掌握且运维成本低的方案，先用POC验证关键链路。")
    } else if domain.contains("数据库") || domain.contains("消息") {
        format!("{domain} 选型建议优先保证一致性、可观测和回滚路径，再评估吞吐上限。")
    } else {
        format!("{domain} 选型建议采用渐进迁移，先在低风险模块验证，再扩大覆盖。")
    }
}

fn str_field(input: &Map<String, Value>, key: &str, default: &str) -> String {
    opt_str_field(input, key).unwrap_or_else(|| default.to_owned())
}

fn opt_str_field(input: &Map<String, Value>, key: &str) -> Option<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn string_vec_field(input: &Map<String, Value>, key: &str) -> Vec<String> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn i64_field(input: &Map<String, Value>, key: &str) -> Option<i64> {
    input.get(key).and_then(Value::as_i64)
}

fn f64_field(input: &Map<String, Value>, key: &str) -> Option<f64> {
    input.get(key).and_then(Value::as_f64)
}

pub struct EngineeringSystemCheck {
    spec: SkillSpec,
}

impl EngineeringSystemCheck {
    pub fn new() -> Self {
        Self {
            spec: engineering_system_check_spec(),
        }
    }
}

impl Default for EngineeringSystemCheck {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for EngineeringSystemCheck {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let test_live = optional_bool_param(&params, "test_live")?.unwrap_or(false);
        let platform = optional_string_param(&params, "platform")?;
        let status = if test_live {
            "pending_approval"
        } else {
            "deferred"
        };
        Ok(SkillOutcome::new(json!({
            "status": status,
            "approval_required": test_live,
            "deferred_reason": if test_live { "实际调用平台API健康检查属于外部平台动作，需要显式审批和适配器接入" } else { "平台连接配置读取尚未接入Rust skill runtime" },
            "skill": self.spec.id,
            "platform": platform,
            "planned_checks": ["凭证完整性", "连接配置", "最近同步状态", "错误码分布"],
            "no_side_effects": true
        })).with_summary("平台接口健康检查骨架已返回"))
    }
}

impl EngineeringSlaMonitor {
    pub fn new() -> Self {
        Self {
            spec: engineering_sla_monitor_spec(),
        }
    }
}

impl Default for EngineeringSlaMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for EngineeringSlaMonitor {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let samples = params
            .get("latency_samples")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
            .unwrap_or_default();

        let (p50, p90, p99) = if samples.is_empty() {
            (
                optional_f64_param(&params, "p50_ms")?.unwrap_or(150.0),
                optional_f64_param(&params, "p90_ms")?.unwrap_or(400.0),
                optional_f64_param(&params, "p99_ms")?.unwrap_or(1800.0),
            )
        } else {
            (
                percentile(samples.clone(), 50.0),
                percentile(samples.clone(), 90.0),
                percentile(samples, 99.0),
            )
        };

        let period_days = optional_i64_param(&params, "period_days")?
            .unwrap_or(30)
            .max(1);
        let system_name =
            optional_string_param(&params, "system_name")?.unwrap_or_else(|| "电商系统".to_owned());
        let total_requests = optional_i64_param(&params, "total_requests")?
            .unwrap_or(0)
            .max(0);
        let error_count = optional_i64_param(&params, "error_count")?
            .unwrap_or(0)
            .max(0);
        let downtime_min = optional_f64_param(&params, "downtime_minutes")?
            .unwrap_or(0.0)
            .max(0.0);
        let slo_avail = optional_f64_param(&params, "slo_availability")?.unwrap_or(99.9);
        let slo_p99 = optional_f64_param(&params, "slo_p99_ms")?.unwrap_or(2000.0);
        let period_minutes = period_days as f64 * 24.0 * 60.0;

        let availability = if downtime_min > 0.0 {
            round4((1.0 - downtime_min / period_minutes) * 100.0)
        } else if total_requests > 0 {
            round4((1.0 - error_count as f64 / total_requests as f64) * 100.0)
        } else {
            99.9
        };

        let budget_total = (1.0 - slo_avail / 100.0) * period_minutes;
        let budget_used_pct = if budget_total > 0.0 {
            round1(downtime_min / budget_total * 100.0)
        } else {
            0.0
        };
        let budget_left_pct = round1(100.0 - budget_used_pct);
        let p99_pass = p99 <= slo_p99;
        let p90_pass = p90 <= 500.0;
        let availability_pass = availability >= slo_avail;
        let violations = [p99_pass, p90_pass, availability_pass]
            .iter()
            .filter(|passed| !**passed)
            .count();
        let peak_p99 = p99 * 2.5;

        let mut output = json!({
            "系统": system_name,
            "统计周期": format!("{period_days}天"),
            "延迟分位数": {
                "P50（中位数）": format!("{p50:.0}ms"),
                "P90（90%用户体验）": format!("{p90:.0}ms"),
                "P99（长尾体验）": format!("{p99:.0}ms"),
                "目标P99": format!("{slo_p99:.0}ms"),
                "P99状态": if p99_pass { "达标".to_owned() } else { format!("超标{}%", round1((p99 - slo_p99) / slo_p99 * 100.0)) }
            },
            "可用性": {
                "实际可用性": format!("{availability:.3}%"),
                "SLO目标": format!("{slo_avail:.3}%"),
                "本期宕机": format!("{downtime_min:.1}分钟"),
                "状态": if availability_pass { "达标" } else { "未达标" }
            },
            "错误预算": {
                "本期可用预算": format!("{budget_total:.1}分钟"),
                "已消耗": format!("{downtime_min:.1}分钟 ({budget_used_pct}%)"),
                "剩余预算": format!("{budget_left_pct}%"),
                "预算状态": budget_state(budget_left_pct)
            },
            "SLO达标情况": [
                {"指标": "P99延迟", "目标": slo_p99, "实际": round1(p99), "状态": if p99_pass { "达标" } else { "未达标" }},
                {"指标": "P90延迟", "目标": 500, "实际": round1(p90), "状态": if p90_pass { "达标" } else { "未达标" }},
                {"指标": "可用性", "目标": slo_avail, "实际": availability, "状态": if availability_pass { "达标" } else { "未达标" }}
            ],
            "SLO违规项数": violations,
            "大促峰值风险": {
                "预估大促P99": format!("{peak_p99:.0}ms（当前×2.5倍流量）"),
                "风险等级": if peak_p99 > slo_p99 * 3.0 { "高风险" } else if peak_p99 > slo_p99 * 1.5 { "中风险" } else { "低风险" }
            }
        });

        if total_requests > 0 {
            output["请求统计"] = json!({
                "总请求数": total_requests,
                "错误数": error_count,
                "错误率": format!("{:.3}%", error_count as f64 / total_requests as f64 * 100.0)
            });
        }

        normalize_engineering_output("engineering_sla_monitor", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("SLA监控分析已完成"))
    }
}

fn percentile(mut data: Vec<f64>, pct: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = pct / 100.0 * (data.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(data.len() - 1);
    let frac = idx - lo as f64;
    data[lo] * (1.0 - frac) + data[hi] * frac
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn budget_state(left: f64) -> &'static str {
    if left <= 0.0 {
        "预算耗尽"
    } else if left < 20.0 {
        "预算紧张"
    } else if left < 50.0 {
        "预算警告"
    } else {
        "预算充足"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_success_contract(output: &Value, summary: &str, readiness: &str) {
        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some(readiness)
        );
        assert_eq!(output.get("summary").and_then(Value::as_str), Some(summary));
        assert!(
            output
                .get("data_boundary")
                .and_then(Value::as_str)
                .is_some_and(|value| value.contains("不读取真实代码仓库"))
        );
    }

    #[tokio::test]
    async fn engineering_arch_review_generates_native_report() {
        let outcome = EngineeringArchReview::new()
            .run(
                json!({
                    "system_type": "电商平台",
                    "daily_orders": 12000,
                    "tech_stack": ["Rust", "PostgreSQL", "Redis"],
                    "pain_points": ["大促接口超时"]
                }),
                SkillContext::default(),
            )
            .await
            .expect("architecture review should run");

        assert_success_contract(&outcome.output, "工程架构评估报告已生成", "user_provided");
        assert_eq!(
            outcome.output.get("系统类型").and_then(Value::as_str),
            Some("电商平台")
        );
        assert!(
            outcome
                .output
                .get("五维评分")
                .and_then(Value::as_array)
                .is_some_and(|items| items.len() == 5)
        );
    }

    #[tokio::test]
    async fn engineering_bug_analysis_generates_native_debug_plan() {
        let outcome = EngineeringBugAnalysis::new()
            .run(
                json!({
                    "error_type": "库存不一致",
                    "error_message": "stock mismatch",
                    "frequency": "频繁"
                }),
                SkillContext::default(),
            )
            .await
            .expect("bug analysis should run");

        assert_success_contract(&outcome.output, "Bug排查方案已生成", "user_provided");
        assert!(
            outcome
                .output
                .get("10步排查流程")
                .and_then(Value::as_array)
                .is_some_and(|items| items.len() == 10)
        );
    }

    #[tokio::test]
    async fn engineering_perf_optimize_generates_native_plan() {
        let outcome = EngineeringPerfOptimize::new()
            .run(
                json!({
                    "bottleneck": "数据库",
                    "current_qps": 500.0,
                    "target_qps": 2000.0,
                    "metrics": {"P99": "1200ms"}
                }),
                SkillContext::default(),
            )
            .await
            .expect("perf optimize should run");

        assert_success_contract(&outcome.output, "性能优化方案已生成", "user_provided");
        assert_eq!(
            outcome.output.get("容量差距倍数").and_then(Value::as_f64),
            Some(4.0)
        );
    }

    #[tokio::test]
    async fn engineering_tech_selection_generates_native_report() {
        let outcome = EngineeringTechSelection::new()
            .run(
                json!({
                    "domain": "数据库",
                    "requirements": ["事务一致性", "复杂查询"],
                    "team_size": 6
                }),
                SkillContext::default(),
            )
            .await
            .expect("tech selection should run");

        assert_success_contract(&outcome.output, "技术选型报告已生成", "user_provided");
        assert!(
            outcome
                .output
                .get("候选方案")
                .and_then(Value::as_array)
                .is_some_and(|items| items.len() >= 3)
        );
    }

    #[tokio::test]
    async fn engineering_sla_monitor_discloses_default_inputs() {
        let outcome = EngineeringSlaMonitor::new()
            .run(json!({}), SkillContext::default())
            .await
            .expect("SLA monitor should run with defaults");

        assert_success_contract(
            &outcome.output,
            "SLA监控分析已完成",
            "calculated_from_defaults",
        );
        assert_eq!(
            outcome.output.get("系统").and_then(Value::as_str),
            Some("电商系统")
        );
    }

    #[tokio::test]
    async fn engineering_system_check_stays_pending_approval_for_live_test() {
        let outcome = EngineeringSystemCheck::new()
            .run(
                json!({"test_live": true, "platform": "淘宝"}),
                SkillContext::default(),
            )
            .await
            .expect("system check should return approval boundary");

        assert_eq!(
            outcome.output.get("status").and_then(Value::as_str),
            Some("pending_approval")
        );
        assert_eq!(
            outcome
                .output
                .get("approval_required")
                .and_then(Value::as_bool),
            Some(true)
        );
    }
}
