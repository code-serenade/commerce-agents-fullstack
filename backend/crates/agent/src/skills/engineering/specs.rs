use serde_json::json;

use super::category;
use crate::skills::{SkillInputField, SkillPriority, SkillSpec};

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

pub(super) fn engineering_arch_review_spec() -> SkillSpec {
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

pub(super) fn engineering_bug_analysis_spec() -> SkillSpec {
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

pub(super) fn engineering_perf_optimize_spec() -> SkillSpec {
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

pub(super) fn engineering_tech_selection_spec() -> SkillSpec {
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

pub(super) fn engineering_system_check_spec() -> SkillSpec {
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

pub(super) fn engineering_sla_monitor_spec() -> SkillSpec {
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
