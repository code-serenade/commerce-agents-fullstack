use std::collections::BTreeMap;

use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillResult, SkillSpec, object_params, optional_f64_param, optional_i64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("service".to_owned())
}

fn string_schema() -> Value {
    json!({"type": "string"})
}

fn number_schema() -> Value {
    json!({"type": "number"})
}

fn integer_schema() -> Value {
    json!({"type": "integer"})
}

fn product_id_schema() -> Value {
    json!({"oneOf": [{"type": "integer"}, {"type": "string"}]})
}

fn string_array_schema() -> Value {
    json!({"type": "array", "items": {"type": "string"}})
}

fn object_schema() -> Value {
    json!({"type": "object"})
}

fn outcome(output: Value, summary: impl Into<String>) -> SkillResult {
    Ok(SkillOutcome::new(output).with_summary(summary))
}

pub fn specs() -> Vec<SkillSpec> {
    vec![
        service_ticket_handler_spec(),
        service_faq_playbook_spec(),
        service_escalation_flow_spec(),
        service_dsr_improvement_spec(),
        service_return_handler_spec(),
        service_query_product_spec(),
        service_nps_analyzer_spec(),
        service_sentiment_analyzer_spec(),
        service_nps_driver_analysis_spec(),
    ]
}

pub fn service_ticket_handler_spec() -> SkillSpec {
    SkillSpec::new(
        "service_ticket_handler",
        "工单处理",
        "生成客服工单处理方案、优先级、SLA和完整话术包；不伪造实时平台政策",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::required(
        "issue_type",
        "问题类型：物流延迟/质量问题/退款退货/发错货/售后咨询/投诉",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "customer_message",
        "客户原始消息",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "customer_emotion",
        "客户情绪：普通/焦虑/愤怒/满意",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_ref",
        "新产品商品引用，如 product:{uuid}；仅作为工单关联记录",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "旧整数商品ID（可选，用于工单字段记录）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "order_no",
        "订单号（可选）",
        string_schema(),
    ))
    .with_tag("deterministic")
}

pub fn service_faq_playbook_spec() -> SkillSpec {
    SkillSpec::new(
        "service_faq_playbook",
        "FAQ话术库",
        "按常见问题生成客服标准话术和处理指引；用户提供政策时优先引用",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::optional(
        "product_ref",
        "新产品商品引用，如 product:{uuid}；作为FAQ适用范围记录",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "旧整数商品ID，作为FAQ适用范围记录",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "questions",
        "需要生成答案的问题列表（留空则生成通用FAQ）",
        string_array_schema(),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "policies",
        "店铺政策，如 {发货时效: '48小时', 退货政策: '7天无理由'}",
        object_schema(),
    ))
    .with_tag("deterministic")
}

pub fn service_escalation_flow_spec() -> SkillSpec {
    SkillSpec::new(
        "service_escalation_flow",
        "升级流程",
        "生成客服升级处理流程、主管接管话术、复核点和纠纷预防动作",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::required(
        "issue_description",
        "问题描述",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "customer_emotion",
        "客户情绪：平静/不满/愤怒/威胁投诉",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "previous_contacts",
        "此前联系次数",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_ref",
        "新产品商品引用，如 product:{uuid}",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "旧整数商品ID（可选）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "customer_history",
        "客户历史（高价值/新客/疑似羊毛党）",
        string_schema(),
    ))
    .with_tag("deterministic")
}

pub fn service_dsr_improvement_spec() -> SkillSpec {
    SkillSpec::new(
        "service_dsr_improvement",
        "DSR提升方案",
        "根据当前DSR评分（可自动从店铺数据加载），输出各维度提升方案",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::optional(
        "description_score",
        "描述相符评分(1-5)；不填则自动加载",
        number_schema(),
    ))
    .with_input(SkillInputField::optional(
        "service_score",
        "服务态度评分(1-5)；不填则自动加载",
        number_schema(),
    ))
    .with_input(SkillInputField::optional(
        "logistics_score",
        "物流服务评分(1-5)；不填则自动加载",
        number_schema(),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音",
        string_schema(),
    ))
    .with_tag("deterministic")
}

pub fn service_query_product_spec() -> SkillSpec {
    SkillSpec::new(
        "service_query_product",
        "查询商品信息",
        "根据本轮显式选择的商品上下文查询商品信息，用于客服应答；不默认注入未选择商品",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::optional(
        "product_ref",
        "新产品商品引用，如 product:{uuid}；只匹配已显式注入的商品上下文",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "旧商品ID兼容字段，可为整数或字符串；只匹配已显式注入的商品上下文",
        product_id_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_name",
        "商品名称关键词（当ID不明时使用）；只匹配已显式注入的商品上下文",
        string_schema(),
    ))
    .with_tag("deterministic")
    .with_tag("data_source")
}

pub fn service_return_handler_spec() -> SkillSpec {
    SkillSpec::new(
        "service_return_handler",
        "退换货处理",
        "根据退换货原因和订单信息，输出处理方案和运费承担方",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::required(
        "reason",
        "退换货原因：不喜欢/质量问题/发错货/尺码不合/破损",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "order_amount",
        "订单金额",
        number_schema(),
    ))
    .with_input(SkillInputField::optional(
        "days_since_receipt",
        "收货天数",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_ref",
        "新产品商品引用，如 product:{uuid}；用于关联商品信息",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "旧整数商品ID（可选，用于关联商品信息）",
        integer_schema(),
    ))
    .with_tag("deterministic")
}

pub struct ServiceReturnHandler {
    spec: SkillSpec,
}

impl ServiceReturnHandler {
    pub fn new() -> Self {
        Self {
            spec: service_return_handler_spec(),
        }
    }
}

impl Default for ServiceReturnHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceReturnHandler {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        let reason = string_param(&params, "reason")?;
        let amount = optional_f64_param(&params, "order_amount")?.unwrap_or(100.0);
        let days = optional_i64_param(&params, "days_since_receipt")?.unwrap_or(3);
        let product_ref = optional_string_param(&params, "product_ref")?;
        let product_id = optional_i64_param(&params, "product_id")?;
        let seller_fault = matches!(reason.as_str(), "质量问题" | "发错货" | "破损");
        let within_policy = days <= 7;

        let mut output = if !within_policy && !seller_fault {
            json!({
                "退换货原因": reason,
                "收货天数": days,
                "处理结果": "超出7天无理由退货期限",
                "建议": "婉拒并解释政策，非卖家过失不受理",
                "话术": "亲，很抱歉该订单已超出7天无理由退货期限，按平台规则无法受理。如有其他问题随时联系~",
                "升级建议": "若客户坚持，请升级至主管协商处理"
            })
        } else if !within_policy && seller_fault {
            json!({
                "退换货原因": reason,
                "收货天数": days,
                "处理结果": "虽超期但卖家过失，特殊受理",
                "责任方": "卖家",
                "处理方案": {
                    "退款金额": amount,
                    "运费补贴": round2(amount * 0.1),
                    "补偿": "赠送10元无门槛券"
                },
                "话术": "亲，虽然已超出退货期限，但这是我们的问题，我们特殊为您受理，请放心~"
            })
        } else {
            let shipping_bearer = if seller_fault { "卖家" } else { "买家" };
            json!({
                "退换货原因": reason,
                "订单金额": amount,
                "收货天数": days,
                "是否在退货期": "是",
                "责任方": if seller_fault { "卖家" } else { "买家" },
                "运费承担": shipping_bearer,
                "处理方案": {
                    "退款金额": amount,
                    "运费补贴": if seller_fault { round2(amount * 0.1) } else { 0.0 },
                    "补偿": if seller_fault { "赠送10元无门槛券" } else { "无" }
                },
                "处理流程": [
                    "确认退货原因",
                    "提供退货地址",
                    format!("运费由{shipping_bearer}承担"),
                    "收到退货后48小时内退款"
                ],
                "话术": format!("亲，退货已受理，运费由{shipping_bearer}承担。请将商品寄回，我们收到后尽快为您退款~")
            })
        };
        if let Some(map) = output.as_object_mut() {
            map.insert("status".to_owned(), json!("success"));
            map.insert("has_data".to_owned(), json!(true));
            map.insert("adapter_status".to_owned(), json!("manual_input"));
            map.insert("data_readiness".to_owned(), json!("user_provided"));
            map.insert("summary".to_owned(), json!("退换货处理方案已生成"));
            map.insert("商品引用".to_owned(), option_string_json(product_ref));
            map.insert("商品ID".to_owned(), option_i64_json(product_id));
        }

        outcome(output, "退换货处理方案已生成")
    }
}

pub struct ServiceTicketHandler {
    spec: SkillSpec,
}

impl ServiceTicketHandler {
    pub fn new() -> Self {
        Self {
            spec: service_ticket_handler_spec(),
        }
    }
}

impl Default for ServiceTicketHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceTicketHandler {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let issue_type = string_param(&params, "issue_type")?;
        let customer_message = optional_string_param(&params, "customer_message")?
            .unwrap_or_else(|| "客户未提供完整原文".to_owned());
        let customer_emotion =
            optional_string_param(&params, "customer_emotion")?.unwrap_or_else(|| {
                infer_customer_emotion(&format!("{issue_type}\n{customer_message}")).to_owned()
            });
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let product_ref = optional_string_param(&params, "product_ref")?;
        let product_id = optional_i64_param(&params, "product_id")?;
        let order_no = optional_string_param(&params, "order_no")?;
        let priority = service_priority(&issue_type, &customer_emotion, &customer_message);
        let sla = service_sla(priority);
        let seller_fault =
            is_seller_fault_issue(&issue_type) || is_seller_fault_issue(&customer_message);
        let solution = ticket_solution(&issue_type, seller_fault);
        let first_action = ticket_first_action(&issue_type);

        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "manual_input",
                "data_readiness": "user_provided",
                "summary": "客服工单处理方案已生成",
                "执行模式": "native_service_ticket",
                "数据来源": "用户输入 + 客服SOP规则；未读取实时平台政策",
                "平台": platform,
                "问题类型": issue_type,
                "客户情绪": customer_emotion,
                "客户原文摘要": truncate_chars(&customer_message, 120),
                "优先级": priority,
                "SLA": sla,
                "处理策略": [
                    first_action,
                    "先确认事实和用户诉求，再给出明确下一步，不只回复原则性安抚",
                    "涉及金额、赔偿、平台投诉或差评威胁时升级主管复核",
                    "平台规则、赔付标准和特殊类目政策需接入 search_platform_policy 后校验"
                ],
                "客服话术": {
                    "接单确认": format!("亲，您反馈的{}问题我已经看到，会马上帮您核实处理。", issue_type),
                    "安抚解释": ticket_empathy_script(&customer_emotion),
                    "解决方案": solution,
                    "跟进": format!("我会在{}内给您同步进展，如需要补充凭证也会一次性说明清楚。", sla),
                    "邀评": "问题处理完成后，如果这次服务有帮到您，欢迎对本次服务做个评价，我们也会继续优化。"
                },
                "内部工单字段": {
                    "issue_type": issue_type,
                    "priority": priority,
                    "sla": sla,
                    "product_ref": product_ref,
                    "product_id": product_id,
                    "order_no": order_no,
                    "seller_fault_likely": seller_fault
                },
                "升级条件": [
                    "客户明确表示投诉平台、曝光、报警或维权",
                    "同一问题重复联系超过2次",
                    "涉及退款、赔偿、补发、拒收等金额动作",
                    "疑似质量批量问题或商品描述不符"
                ],
                "政策边界": "当前为本地SOP执行结果，未联网读取平台最新规则；上线 search_platform_policy 后应二次校验赔付和时效。"
            }),
            "客服工单处理方案已生成",
        )
    }
}

pub struct ServiceFaqPlaybook {
    spec: SkillSpec,
}

impl ServiceFaqPlaybook {
    pub fn new() -> Self {
        Self {
            spec: service_faq_playbook_spec(),
        }
    }
}

impl Default for ServiceFaqPlaybook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceFaqPlaybook {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let product_ref = optional_string_param(&params, "product_ref")?;
        let product_id = optional_i64_param(&params, "product_id")?;
        let questions =
            optional_string_vec_param(&params, "questions")?.unwrap_or_else(default_faq_questions);
        let policies = params
            .get("policies")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let faq = questions
            .into_iter()
            .take(20)
            .map(|question| faq_answer(&question, &platform, &policies))
            .collect::<Vec<_>>();

        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "manual_input",
                "data_readiness": if policies.is_empty() { "general_sop_defaults" } else { "user_policy_provided" },
                "summary": "客服FAQ话术库已生成",
                "执行模式": "native_faq_playbook",
                "数据来源": if policies.is_empty() { "通用客服SOP；未提供店铺政策" } else { "用户提供店铺政策 + 通用客服SOP" },
                "平台": platform,
                "商品引用": product_ref,
                "商品ID": product_id,
                "FAQ": faq,
                "政策来源说明": "用户输入政策优先；未提供时采用保守通用话术，不冒充实时平台政策。",
                "需人工确认": [
                    "发货时效",
                    "退换货细则",
                    "特殊类目售后限制",
                    "活动价格保护",
                    "发票类型和开票主体"
                ],
                "后续适配器": "接入 product_profile/shop_policy/search_platform_policy 后可自动替换商品参数和平台最新规则。"
            }),
            "客服FAQ话术库已生成",
        )
    }
}

pub struct ServiceEscalationFlow {
    spec: SkillSpec,
}

impl ServiceEscalationFlow {
    pub fn new() -> Self {
        Self {
            spec: service_escalation_flow_spec(),
        }
    }
}

impl Default for ServiceEscalationFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceEscalationFlow {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let issue_description = string_param(&params, "issue_description")?;
        let customer_emotion = optional_string_param(&params, "customer_emotion")?
            .unwrap_or_else(|| infer_customer_emotion(&issue_description).to_owned());
        let previous_contacts = optional_i64_param(&params, "previous_contacts")?.unwrap_or(0);
        let product_ref = optional_string_param(&params, "product_ref")?;
        let product_id = optional_i64_param(&params, "product_id")?;
        let customer_history = optional_string_param(&params, "customer_history")?
            .unwrap_or_else(|| "普通客户".to_owned());
        let severity =
            escalation_severity(&issue_description, &customer_emotion, previous_contacts);
        let requires_review = matches!(severity, "S1" | "S2");

        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "manual_input",
                "data_readiness": "user_provided",
                "summary": "客服升级处理流程已生成",
                "执行模式": "native_escalation_flow",
                "数据来源": "用户输入 + 客服升级SOP；未读取实时平台政策",
                "问题摘要": truncate_chars(&issue_description, 140),
                "客户情绪": customer_emotion,
                "此前联系次数": previous_contacts,
                "客户历史": customer_history,
                "商品引用": product_ref,
                "商品ID": product_id,
                "严重度": severity,
                "升级路径": escalation_path(severity),
                "主管接管话术": escalation_manager_script(severity),
                "赔偿建议": compensation_suggestion(severity),
                "内部工单模板": {
                    "severity": severity,
                    "owner": if requires_review { "客服主管" } else { "一线客服" },
                    "next_update_sla": if severity == "S1" { "10分钟" } else if severity == "S2" { "30分钟" } else { "2小时" },
                    "evidence_required": ["订单信息", "聊天记录", "商品/物流凭证", "客户诉求"],
                    "review_required": requires_review
                },
                "纠纷预防": [
                    "先承认用户体验问题，再核实事实，不与用户争论责任归属",
                    "给出明确处理时限和下一次同步时间",
                    "金额补偿、拒赔、平台申诉相关动作必须复核",
                    "处理完成后回填根因，避免同类问题重复发生"
                ],
                "需审批/复核": requires_review,
                "政策边界": "赔付金额、平台投诉口径和特殊类目售后规则需要后续 search_platform_policy / shop_policy adapter 校验。"
            }),
            "客服升级处理流程已生成",
        )
    }
}

pub struct ServiceQueryProduct {
    spec: SkillSpec,
}

impl ServiceQueryProduct {
    pub fn new() -> Self {
        Self {
            spec: service_query_product_spec(),
        }
    }
}

impl Default for ServiceQueryProduct {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceQueryProduct {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let query_product_ref = optional_string_param(&params, "product_ref")?;
        let query_product_id = optional_product_id_param(&params, "product_id")?;
        let query_product_name = optional_string_param(&params, "product_name")?;
        let product_context = explicit_product_context(&context.metadata);

        if !product_context.has_explicit_context() {
            return outcome(
                json!({
                    "status": "deferred",
                    "has_data": false,
                    "adapter_status": "context_required",
                    "data_readiness": "explicit_product_context_required",
                    "summary": "未发现本轮显式选择的商品上下文",
                    "skill": self.spec.id,
                    "query": {
                        "product_ref": query_product_ref,
                        "product_id": query_product_id,
                        "product_name": query_product_name,
                    },
                    "deferred_reason": "当前 Agent 上下文没有显式选择的商品 ID、商品引用、商品摘要或商品来源；为避免默认注入错误商品，已停止商品查询。",
                    "required_action": "请用户在前端引用商品中手动勾选要分析的商品，或上传/注入明确商品资料后重试。",
                    "answerPolicy": {
                        "source": "explicit_agent_product_context_required",
                        "no_default_product": true,
                        "live_platform_lookup": false
                    }
                }),
                "未发现本轮显式选择的商品上下文",
            );
        }

        let match_result = product_context.match_query(
            query_product_ref.as_deref(),
            query_product_id.as_deref(),
            query_product_name.as_deref(),
        );
        if !match_result.matched {
            return outcome(
                json!({
                    "status": "deferred",
                    "has_data": false,
                    "adapter_status": "context_filter_not_matched",
                    "data_readiness": "explicit_product_context_not_matched",
                    "summary": "已读取商品上下文，但没有匹配请求的商品条件",
                    "skill": self.spec.id,
                    "query": {
                        "product_ref": query_product_ref,
                        "product_id": query_product_id,
                        "product_name": query_product_name,
                    },
                    "availableProductIds": product_context.product_ids,
                    "refs": product_context.refs,
                    "sources": product_context.sources,
                    "productContextSummary": product_context.summary,
                    "deferred_reason": "本轮确实有显式商品上下文，但输入的商品ID或商品名没有在该上下文中命中；不会改用默认商品或隐式搜索其它商品。",
                    "answerPolicy": {
                        "source": "explicit_agent_product_context",
                        "no_default_product": true,
                        "live_platform_lookup": false
                    }
                }),
                "已读取商品上下文，但没有匹配请求的商品条件",
            );
        }

        let summary = product_context.summary.clone();
        let product_ids = if match_result.product_ids.is_empty() {
            product_context.product_ids.clone()
        } else {
            match_result.product_ids
        };
        let refs = if match_result.refs.is_empty() {
            product_context.refs.clone()
        } else {
            match_result.refs
        };
        let sources = if match_result.sources.is_empty() {
            product_context.sources.clone()
        } else {
            match_result.sources
        };

        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "context_metadata",
                "data_readiness": "explicit_product_context",
                "summary": "商品上下文查询已完成",
                "skill": self.spec.id,
                "query": {
                    "product_ref": query_product_ref,
                    "product_id": query_product_id,
                    "product_name": query_product_name,
                },
                "matchedProductIds": product_ids,
                "refs": refs,
                "sources": sources,
                "productContextSummary": summary,
                "answerPolicy": {
                    "source": "当前 Agent 运行时显式注入的商品上下文",
                    "no_default_product": true,
                    "live_platform_lookup": false,
                    "usage": "客服、运营或内容岗位可基于这些商品资料回答，但不能声称已查询实时平台数据库。"
                }
            }),
            "商品上下文查询已完成",
        )
    }
}

#[derive(Clone, Debug, Default)]
struct ExplicitProductContext {
    product_ids: Vec<String>,
    refs: Vec<String>,
    sources: Vec<Value>,
    source_titles: Vec<String>,
    summary: String,
    explicit_signal: bool,
}

#[derive(Clone, Debug, Default)]
struct ProductContextMatch {
    matched: bool,
    product_ids: Vec<String>,
    refs: Vec<String>,
    sources: Vec<Value>,
}

impl ExplicitProductContext {
    fn has_explicit_context(&self) -> bool {
        self.explicit_signal
    }

    fn match_query(
        &self,
        product_ref: Option<&str>,
        product_id: Option<&str>,
        product_name: Option<&str>,
    ) -> ProductContextMatch {
        if let Some(product_ref) = product_ref.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_ref(product_ref) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: matching_product_ids_by_ref(&self.product_ids, product_ref),
                refs: matching_refs(&self.refs, product_ref),
                sources: matching_sources_by_product_ref(&self.sources, product_ref),
            };
        }

        if let Some(product_id) = product_id.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_id(product_id) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: matching_product_ids(&self.product_ids, product_id),
                refs: matching_refs(&self.refs, product_id),
                sources: matching_sources_by_product_id(&self.sources, product_id),
            };
        }

        if let Some(product_name) = product_name.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_name(product_name) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: self.product_ids.clone(),
                refs: self.refs.clone(),
                sources: matching_sources_by_text(&self.sources, product_name),
            };
        }

        ProductContextMatch {
            matched: true,
            product_ids: self.product_ids.clone(),
            refs: self.refs.clone(),
            sources: self.sources.clone(),
        }
    }

    fn matches_product_ref(&self, product_ref: &str) -> bool {
        let product_ref = product_ref.trim();
        let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
        self.product_ids
            .iter()
            .any(|item| eq_ignore_ascii_case_trimmed(item, product_id))
            || self
                .refs
                .iter()
                .any(|item| eq_ignore_ascii_case_trimmed(item, product_ref))
            || self.sources.iter().any(|source| {
                source_product_id(source)
                    .as_deref()
                    .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                    || text_contains_case_insensitive(&source.to_string(), product_ref)
                    || text_contains_case_insensitive(&source.to_string(), product_id)
            })
            || text_contains_case_insensitive(&self.summary, product_ref)
            || text_contains_case_insensitive(&self.summary, product_id)
            || self.source_titles.iter().any(|item| {
                text_contains_case_insensitive(item, product_ref)
                    || text_contains_case_insensitive(item, product_id)
            })
    }

    fn matches_product_id(&self, product_id: &str) -> bool {
        let product_id = product_id.trim();
        self.product_ids
            .iter()
            .any(|item| eq_ignore_ascii_case_trimmed(item, product_id))
            || self
                .refs
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_id))
            || self.sources.iter().any(|source| {
                source_product_id(source)
                    .as_deref()
                    .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                    || text_contains_case_insensitive(&source.to_string(), product_id)
            })
            || text_contains_case_insensitive(&self.summary, product_id)
            || self
                .source_titles
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_id))
    }

    fn matches_product_name(&self, product_name: &str) -> bool {
        let product_name = product_name.trim();
        text_contains_case_insensitive(&self.summary, product_name)
            || self
                .source_titles
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_name))
            || self
                .sources
                .iter()
                .any(|source| text_contains_case_insensitive(&source.to_string(), product_name))
    }
}

fn explicit_product_context(metadata: &Map<String, Value>) -> ExplicitProductContext {
    let product_ids = collect_metadata_list(
        metadata,
        &["request.context.product_ids", "context.product_ids"],
    );
    let refs = collect_metadata_list(
        metadata,
        &[
            "request.context.refs",
            "context.refs",
            "request.context.combined_refs",
            "context.combined_refs",
        ],
    );
    let sources = collect_metadata_sources(
        metadata,
        &["request.context.sources_json", "context.sources_json"],
    );
    let source_titles = collect_metadata_text_parts(
        metadata,
        &["request.context.source_titles", "context.source_titles"],
    );
    let selected_summaries = collect_metadata_texts(
        metadata,
        &[
            "request.context.selected_summary",
            "context.selected_summary",
            "request.context.summary",
            "context.summary",
        ],
    );
    let combined_summaries = collect_metadata_texts(
        metadata,
        &[
            "request.context.combined_summary",
            "context.combined_summary",
        ],
    );
    let mut summary_parts = selected_summaries.clone();
    summary_parts.extend(combined_summaries.clone());
    let summary = truncate_chars(&summary_parts.join("\n\n"), 4000);

    let explicit_signal = !product_ids.is_empty()
        || refs.iter().any(|item| item.starts_with("product:"))
        || !sources.is_empty()
        || !source_titles.is_empty()
        || !selected_summaries.is_empty();

    ExplicitProductContext {
        product_ids,
        refs,
        sources,
        source_titles,
        summary,
        explicit_signal,
    }
}

fn optional_product_id_param(params: &Value, name: &str) -> Result<Option<String>, SkillError> {
    Ok(match object_params(params)?.get(name) {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) => {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            }
        }
        Some(Value::Number(value)) => Some(value.to_string()),
        Some(_) => {
            return Err(SkillError::invalid_parameter(
                name,
                "string or integer product id",
            ));
        }
    })
}

fn collect_metadata_texts(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for key in keys {
        if let Some(value) = metadata_string_value(metadata, key) {
            push_unique_non_empty(&mut out, value);
        }
    }
    out
}

fn collect_metadata_text_parts(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for value in collect_metadata_texts(metadata, keys) {
        for part in value.split('|') {
            push_unique_non_empty(&mut out, part.trim().to_owned());
        }
    }
    out
}

fn collect_metadata_list(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for key in keys {
        let Some(value) = metadata.get(*key) else {
            continue;
        };
        match value {
            Value::String(text) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    collect_list_value(&mut out, &parsed);
                } else {
                    for item in text.split(',') {
                        push_unique_non_empty(&mut out, item.trim().to_owned());
                    }
                }
            }
            other => collect_list_value(&mut out, other),
        }
    }
    out
}

fn collect_list_value(out: &mut Vec<String>, value: &Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_list_value(out, item);
            }
        }
        Value::String(text) => {
            for item in text.split(',') {
                push_unique_non_empty(out, item.trim().to_owned());
            }
        }
        Value::Number(number) => push_unique_non_empty(out, number.to_string()),
        Value::Bool(value) => push_unique_non_empty(out, value.to_string()),
        Value::Object(_) | Value::Null => {}
    }
}

fn collect_metadata_sources(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<Value> {
    let mut out = Vec::new();
    for key in keys {
        let Some(value) = metadata.get(*key) else {
            continue;
        };
        match value {
            Value::String(text) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    collect_source_value(&mut out, parsed);
                }
            }
            other => collect_source_value(&mut out, other.clone()),
        }
    }
    out
}

fn collect_source_value(out: &mut Vec<Value>, value: Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_source_value(out, item);
            }
        }
        Value::Object(_) | Value::String(_) => {
            if !out.iter().any(|item| item == &value) {
                out.push(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn matching_product_ids(product_ids: &[String], product_id: &str) -> Vec<String> {
    product_ids
        .iter()
        .filter(|item| eq_ignore_ascii_case_trimmed(item, product_id))
        .cloned()
        .collect()
}

fn matching_product_ids_by_ref(product_ids: &[String], product_ref: &str) -> Vec<String> {
    let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
    matching_product_ids(product_ids, product_id)
}

fn matching_refs(refs: &[String], product_id: &str) -> Vec<String> {
    refs.iter()
        .filter(|item| text_contains_case_insensitive(item, product_id))
        .cloned()
        .collect()
}

fn matching_sources_by_product_ref(sources: &[Value], product_ref: &str) -> Vec<Value> {
    let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
    sources
        .iter()
        .filter(|source| {
            source_product_id(source)
                .as_deref()
                .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                || text_contains_case_insensitive(&source.to_string(), product_ref)
                || text_contains_case_insensitive(&source.to_string(), product_id)
        })
        .cloned()
        .collect()
}

fn matching_sources_by_product_id(sources: &[Value], product_id: &str) -> Vec<Value> {
    sources
        .iter()
        .filter(|source| {
            source_product_id(source)
                .as_deref()
                .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                || text_contains_case_insensitive(&source.to_string(), product_id)
        })
        .cloned()
        .collect()
}

fn matching_sources_by_text(sources: &[Value], text: &str) -> Vec<Value> {
    sources
        .iter()
        .filter(|source| text_contains_case_insensitive(&source.to_string(), text))
        .cloned()
        .collect()
}

fn source_product_id(source: &Value) -> Option<String> {
    source
        .get("product_id")
        .or_else(|| source.get("productId"))
        .and_then(|value| match value {
            Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_owned()),
            Value::Number(number) => Some(number.to_string()),
            _ => None,
        })
}

fn option_string_json(value: Option<String>) -> Value {
    value
        .filter(|value| !value.trim().is_empty())
        .map(Value::String)
        .unwrap_or(Value::Null)
}

fn option_i64_json(value: Option<i64>) -> Value {
    value.map(|value| json!(value)).unwrap_or(Value::Null)
}

fn strip_product_ref_prefix(value: &str) -> Option<&str> {
    value
        .strip_prefix("product:")
        .or_else(|| value.strip_prefix("product/"))
        .filter(|value| !value.trim().is_empty())
}

fn metadata_string_value(metadata: &Map<String, Value>, key: &str) -> Option<String> {
    match metadata.get(key)? {
        Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_owned()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn push_unique_non_empty(out: &mut Vec<String>, value: String) {
    let value = value.trim();
    if !value.is_empty() && !out.iter().any(|item| item == value) {
        out.push(value.to_owned());
    }
}

fn eq_ignore_ascii_case_trimmed(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

fn text_contains_case_insensitive(text: &str, needle: &str) -> bool {
    let needle = needle.trim();
    !needle.is_empty() && text.to_lowercase().contains(&needle.to_lowercase())
}

fn infer_customer_emotion(text: &str) -> &'static str {
    if contains_any_local(
        text,
        &[
            "威胁投诉",
            "投诉",
            "曝光",
            "举报",
            "维权",
            "骗子",
            "垃圾",
            "气死",
            "差评",
        ],
    ) {
        "威胁投诉/愤怒"
    } else if contains_any_local(text, &["着急", "急", "催", "怎么还", "等了", "马上"]) {
        "焦虑"
    } else if contains_any_local(text, &["谢谢", "满意", "好评", "不错"]) {
        "满意"
    } else if contains_any_local(text, &["不满", "失望", "太差", "不满意"]) {
        "不满"
    } else {
        "普通"
    }
}

fn service_priority(
    issue_type: &str,
    customer_emotion: &str,
    customer_message: &str,
) -> &'static str {
    if contains_any_local(
        &format!("{issue_type}\n{customer_emotion}\n{customer_message}"),
        &[
            "威胁投诉",
            "投诉",
            "曝光",
            "举报",
            "维权",
            "骗子",
            "假货",
            "平台介入",
        ],
    ) {
        "P0"
    } else if contains_any_local(
        &format!("{issue_type}\n{customer_emotion}\n{customer_message}"),
        &[
            "质量", "破损", "发错", "退款", "退货", "换货", "赔偿", "愤怒", "不满",
        ],
    ) {
        "P1"
    } else {
        "P2"
    }
}

fn service_sla(priority: &str) -> &'static str {
    match priority {
        "P0" => "2分钟",
        "P1" => "30分钟",
        _ => "2小时",
    }
}

fn is_seller_fault_issue(text: &str) -> bool {
    contains_any_local(
        text,
        &["质量", "发错", "破损", "少发", "漏发", "假货", "描述不符"],
    )
}

fn ticket_first_action(issue_type: &str) -> &'static str {
    if contains_any_local(issue_type, &["物流", "延迟", "未到", "快递"]) {
        "先核实物流节点和预计送达时间，必要时同步催派或补发方案"
    } else if contains_any_local(issue_type, &["质量", "破损", "发错"]) {
        "先收集凭证并判断卖家责任，给出退换货、补发或补偿路径"
    } else if contains_any_local(issue_type, &["退款", "退货", "换货", "售后"]) {
        "先确认收货天数、商品状态和责任归属，再明确运费承担与退款时效"
    } else if contains_any_local(issue_type, &["投诉", "差评", "维权"]) {
        "先升级主管关注，避免一线反复解释导致情绪升级"
    } else {
        "先确认订单和商品信息，再按客户诉求给出下一步"
    }
}

fn ticket_solution(issue_type: &str, seller_fault: bool) -> String {
    if contains_any_local(issue_type, &["物流", "延迟", "未到", "快递"]) {
        "我会马上核实物流轨迹并帮您催派；如果超过承诺时效，会同步补偿或补发可选方案。".to_owned()
    } else if contains_any_local(issue_type, &["退款", "退货", "换货", "售后"]) {
        let freight = if seller_fault {
            "运费由我们承担"
        } else {
            "运费按店铺/平台规则确认"
        };
        format!(
            "我先帮您登记售后申请，确认商品状态后给出退换货路径，{freight}，退款进度会持续同步。"
        )
    } else if seller_fault {
        "如果核实确属我们发错、破损或质量问题，会优先给您补发/退换，并同步可执行补偿方案。"
            .to_owned()
    } else {
        "我会先核实订单、商品和沟通记录，给您一个明确处理结论；涉及特殊政策的部分会提交主管复核。"
            .to_owned()
    }
}

fn ticket_empathy_script(customer_emotion: &str) -> &'static str {
    if contains_any_local(customer_emotion, &["威胁投诉", "愤怒"]) {
        "确实让您有很差的体验了，这个问题我先帮您推进处理，不让您反复解释。"
    } else if contains_any_local(customer_emotion, &["焦虑", "不满"]) {
        "理解您现在比较着急，我会把处理进度和时间点说清楚，避免您一直等待。"
    } else if contains_any_local(customer_emotion, &["满意"]) {
        "感谢您的理解，我们会继续把后续服务处理完整。"
    } else {
        "收到，我会按订单情况帮您核实，并尽快给出明确答复。"
    }
}

fn faq_answer(question: &str, platform: &str, policies: &Map<String, Value>) -> Value {
    let policy_text = |keys: &[&str]| {
        keys.iter()
            .find_map(|key| {
                policies
                    .get(*key)
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .unwrap_or_default()
    };
    let q = question.trim();
    let (short, detail, guide) = if contains_any_local(q, &["发货", "多久", "时效"]) {
        let policy = policy_text(&["发货时效", "发货时间", "delivery"]);
        let detail = if policy.is_empty() {
            format!(
                "正常会按{platform}店铺承诺时效发出；如遇大促、预售或缺货，会以订单页和客服确认为准。"
            )
        } else {
            format!("本店当前发货规则：{policy}。如订单异常，会主动同步进度。")
        };
        (
            "正常按店铺承诺时效发货，异常会同步处理。",
            detail,
            "查询订单状态；预售/缺货/大促订单需单独标记并同步预计时间。",
        )
    } else if contains_any_local(q, &["退", "换", "退款", "售后"]) {
        let policy = policy_text(&["退货政策", "售后政策", "return"]);
        let detail = if policy.is_empty() {
            "支持按平台和店铺规则申请售后；质量、发错、破损等卖家责任问题优先处理，非质量原因按商品状态确认。".to_owned()
        } else {
            format!("售后规则：{policy}。具体责任和运费以订单、凭证和平台规则复核为准。")
        };
        (
            "可以按规则申请售后，我们会协助处理。",
            detail,
            "先确认收货天数、商品状态、责任方和凭证，再选择退货、换货、补发或补偿。",
        )
    } else if contains_any_local(q, &["质量", "正品", "保障", "假"]) {
        (
            "商品会按页面承诺和店铺标准发出。",
            "如收到后发现破损、发错、明显质量异常，请保留外包装、商品照片和订单号，我们会优先核实处理。"
                .to_owned(),
            "质量类问题需要凭证留存；疑似批量问题要升级质检和商品负责人。",
        )
    } else if contains_any_local(q, &["优惠", "活动", "价格", "保价"]) {
        (
            "优惠以当前商品页和活动页为准。",
            "您可以先领取店铺券/平台券后再下单；价格保护、活动补差等需要按平台当期规则核实。"
                .to_owned(),
            "价格类问题不要承诺固定补差，先核对活动规则和订单时间。",
        )
    } else if contains_any_local(q, &["尺码", "规格", "颜色", "怎么选"]) {
        (
            "可以按页面规格说明选择，不确定可以发使用场景给客服。",
            "建议结合商品规格、使用场景和个人偏好选择；如果您告诉我们身高体重/设备型号/使用需求，可以进一步建议。"
                .to_owned(),
            "规格建议不能替代用户最终选择；高退货风险规格要提示确认。",
        )
    } else if contains_any_local(q, &["包邮", "运费"]) {
        (
            "运费以订单结算页显示为准。",
            "不同地区、活动和商品组合可能影响运费；请以下单页展示为准，特殊地区可能有额外运费。"
                .to_owned(),
            "不要口头承诺所有地区包邮，先引导用户查看结算页。",
        )
    } else if contains_any_local(q, &["发票", "开票"]) {
        (
            "支持情况以店铺开票规则为准。",
            "如需发票，请提供抬头、税号和开票信息；电子或纸质发票类型需按店铺配置确认。".to_owned(),
            "开票主体、类目和时效需要人工确认后回复。",
        )
    } else {
        (
            "这个问题我可以先帮您核实。",
            "请您补充订单号、商品规格或具体诉求，我们会按订单情况给出明确处理方案。".to_owned(),
            "无法判断的问题进入人工工单，不要编造商品参数或平台政策。",
        )
    };

    json!({
        "问题": q,
        "简短版": short,
        "详细版": detail,
        "处理指引": guide
    })
}

fn escalation_severity(
    issue_description: &str,
    customer_emotion: &str,
    previous_contacts: i64,
) -> &'static str {
    let text = format!("{issue_description}\n{customer_emotion}");
    if contains_any_local(
        &text,
        &["报警", "媒体", "曝光", "监管", "诉讼", "假货", "人身"],
    ) {
        "S1"
    } else if contains_any_local(&text, &["威胁投诉", "投诉", "维权", "平台介入", "赔偿"])
        || previous_contacts >= 3
    {
        "S2"
    } else if contains_any_local(&text, &["愤怒", "差评", "退款", "破损", "质量", "发错"])
        || previous_contacts >= 2
    {
        "S3"
    } else {
        "S4"
    }
}

fn escalation_path(severity: &str) -> Vec<&'static str> {
    match severity {
        "S1" => vec![
            "一线客服立即停止争辩并记录完整事实",
            "客服主管10分钟内接管",
            "法务/平台运营/商品负责人同步复核",
            "形成书面处理口径和证据包",
        ],
        "S2" => vec![
            "一线客服先安抚并承诺明确回访时间",
            "客服主管30分钟内复核责任和补偿方案",
            "需要金额动作时提交审批",
            "处理后回填纠纷根因",
        ],
        "S3" => vec![
            "一线客服核实订单和凭证",
            "按SOP给出退换/补发/解释方案",
            "2小时内回访结果",
            "未解决再升级主管",
        ],
        _ => vec![
            "一线客服常规处理",
            "记录客户诉求和处理结论",
            "必要时同步商品/物流信息",
        ],
    }
}

fn escalation_manager_script(severity: &str) -> &'static str {
    match severity {
        "S1" => {
            "您好，我是本店客服主管。这个问题我已经接管，会先保护您的权益并核实全部证据，接下来由我直接同步处理进展。"
        }
        "S2" => {
            "您好，我是客服主管。看到您已经多次沟通仍未解决，我先向您道歉，我会在约定时间内给出明确处理方案。"
        }
        "S3" => "您好，我来协助跟进这个售后问题，会先核实凭证和订单情况，再给您同步可执行方案。",
        _ => "您好，我会帮您确认当前问题并推动处理，有进展会及时告知您。",
    }
}

fn compensation_suggestion(severity: &str) -> Value {
    match severity {
        "S1" => json!({
            "建议": "暂停一线承诺，主管/法务/平台运营复核后给最终方案",
            "可选动作": ["全额退款", "补发", "专项补偿", "平台申诉协助"],
            "审批要求": "必须审批"
        }),
        "S2" => json!({
            "建议": "优先给出可执行补救方案，金额补偿走审批",
            "可选动作": ["退换货优先处理", "补发", "优惠券补偿", "运费补贴"],
            "审批要求": "金额或平台动作需审批"
        }),
        "S3" => json!({
            "建议": "按SOP处理，必要时给低风险安抚补偿",
            "可选动作": ["售后登记", "物流催派", "退换货协助"],
            "审批要求": "超SOP补偿需复核"
        }),
        _ => json!({
            "建议": "常规处理，不主动承诺赔付",
            "可选动作": ["解释规则", "协助查询", "引导补充信息"],
            "审批要求": "通常无需审批"
        }),
    }
}

fn contains_any_local(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}

pub struct ServiceDsrImprovement {
    spec: SkillSpec,
}

impl ServiceDsrImprovement {
    pub fn new() -> Self {
        Self {
            spec: service_dsr_improvement_spec(),
        }
    }
}

impl Default for ServiceDsrImprovement {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceDsrImprovement {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let desc = optional_f64_param(&params, "description_score")?.unwrap_or(4.6);
        let svc = optional_f64_param(&params, "service_score")?.unwrap_or(4.7);
        let logi = optional_f64_param(&params, "logistics_score")?.unwrap_or(4.5);
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let overall = round2((desc + svc + logi) / 3.0);
        let lowest = [("描述相符", desc), ("服务态度", svc), ("物流服务", logi)]
            .into_iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(("描述相符", desc));
        let has_user_scores = params.get("description_score").is_some()
            || params.get("service_score").is_some()
            || params.get("logistics_score").is_some();
        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "manual_input",
                "data_readiness": if has_user_scores { "user_provided" } else { "calculated_from_defaults" },
                "summary": "DSR提升方案已生成",
                "平台": platform,
                "执行模式": "native_dsr_improvement",
                "数据来源": "用户提供/默认样例；未读取实时店铺指标和行业基准",
                "DSR总评": overall,
                "各维度评分": {"描述相符": desc, "服务态度": svc, "物流服务": logi},
                "健康状态": if overall >= 4.8 { "优秀" } else if overall >= 4.6 { "正常" } else { "需改善" },
                "优先改善维度": lowest.0,
                "改善方案": dsr_action_plan(lowest.0),
                "7天动作": [
                    "拉取低分评价和售后工单，按商品描述/服务响应/物流时效分组",
                    "为低分订单建立回访池，优先处理可补救客户",
                    "把高频问题同步商品、仓配和客服负责人",
                    "每日复盘新低分原因并更新客服话术"
                ],
                "30天追踪指标": ["DSR总评", "低分评价占比", "售后响应时长", "退款纠纷率", "物流异常率"],
                "后续适配器": "接入 store_metrics/fresh_benchmark 后可用真实店铺指标替换默认样例。"
            }),
            "DSR提升方案已生成",
        )
    }
}

fn dsr_action_plan(dimension: &str) -> Vec<&'static str> {
    match dimension {
        "描述相符" => vec![
            "排查主图、详情页、规格参数和实际商品不一致点",
            "把容易误解的尺寸、材质、颜色、适配范围前置说明",
            "对高退货SKU补充实拍图和购买前确认话术",
        ],
        "服务态度" => vec![
            "把P0/P1客诉SLA前置到客服排班和质检规则",
            "沉淀高频售后问题的标准话术，减少反复追问",
            "对差评/投诉威胁场景启动主管接管流程",
        ],
        "物流服务" => vec![
            "标记高异常地区和高延迟仓配线路",
            "对延迟订单提前触达并给出预计时间",
            "将物流异常与补偿/催派流程合并到客服工单",
        ],
        _ => vec![
            "定位低分订单与评价关键词",
            "建立问题分类",
            "制定客服/物流/描述修正动作",
        ],
    }
}

fn default_faq_questions() -> Vec<String> {
    vec![
        "什么时候发货？".to_owned(),
        "支持退货退款吗？怎么操作？".to_owned(),
        "质量有保障吗？".to_owned(),
        "有优惠活动吗？".to_owned(),
        "怎么选尺码/规格？".to_owned(),
        "包邮吗？运费多少？".to_owned(),
        "可以开发票吗？".to_owned(),
        "商品是正品吗？".to_owned(),
    ]
}

pub fn service_nps_analyzer_spec() -> SkillSpec {
    SkillSpec::new(
        "service_nps_analyzer",
        "NPS客户满意度分析",
        "计算NPS净推荐值、CSAT满意度评分和客户忠诚度分层",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "total_respondents",
        "调查总人数",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "score_distribution",
        "评分分布，key=评分，value=人数",
        object_schema(),
    ))
    .with_input(SkillInputField::optional(
        "promoters_count",
        "推荐者数量（9-10分）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "passives_count",
        "被动者数量（7-8分）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "detractors_count",
        "批评者数量（0-6分）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "top_issues",
        "批评者主要问题列表",
        string_array_schema(),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音/综合",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "days",
        "若无调查数据，从外部数据推算的天数",
        integer_schema(),
    ))
    .with_tag("deterministic")
}

pub struct ServiceNpsAnalyzer {
    spec: SkillSpec,
}

impl ServiceNpsAnalyzer {
    pub fn new() -> Self {
        Self {
            spec: service_nps_analyzer_spec(),
        }
    }
}

impl Default for ServiceNpsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceNpsAnalyzer {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let mut promoters = 0_i64;
        let mut passives = 0_i64;
        let mut detractors = 0_i64;
        let mut total = 0_i64;
        let mut data_source = "用户提供";

        if let Some(distribution) = params.get("score_distribution").and_then(Value::as_object) {
            for (score, count) in distribution {
                let score = score.parse::<i64>().map_err(|_| {
                    SkillError::invalid_parameter("score_distribution", "score keys 0-10")
                })?;
                let count = count.as_i64().ok_or_else(|| {
                    SkillError::invalid_parameter("score_distribution", "integer counts")
                })?;
                total += count;
                if score >= 9 {
                    promoters += count;
                } else if score >= 7 {
                    passives += count;
                } else {
                    detractors += count;
                }
            }
        } else if params.get("promoters_count").is_some() {
            promoters = optional_i64_param(&params, "promoters_count")?.unwrap_or(0);
            passives = optional_i64_param(&params, "passives_count")?.unwrap_or(0);
            detractors = optional_i64_param(&params, "detractors_count")?.unwrap_or(0);
            total = optional_i64_param(&params, "total_respondents")?
                .unwrap_or(promoters + passives + detractors);
        } else {
            data_source = "默认样例";
            promoters = 45;
            passives = 35;
            detractors = 20;
            total = 100;
        }

        if total <= 0 {
            total = promoters + passives + detractors;
        }
        if total <= 0 {
            return Err(SkillError::execution(
                "无有效数据，请提供评分分布或各分段人数",
            ));
        }

        let prom_pct = round1(promoters as f64 / total as f64 * 100.0);
        let pass_pct = round1(passives as f64 / total as f64 * 100.0);
        let detr_pct = round1(detractors as f64 / total as f64 * 100.0);
        let nps = round1(prom_pct - detr_pct);
        let csat_pct = round1((promoters + passives) as f64 / total as f64 * 100.0);
        let top_issues = optional_string_vec_param_or_empty(&params, "top_issues")?;
        let data_readiness = if data_source == "默认样例" {
            "calculated_from_defaults"
        } else {
            "user_provided"
        };

        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": data_readiness,
            "summary": "NPS与CSAT分析已完成",
            "数据来源": data_source,
            "调查总人数": total,
            "NPS净推荐值": nps,
            "NPS评级": nps_grade(nps),
            "电商行业基准": "NPS 优秀≥50 | 正常≥0 | 危险<-10",
            "分群详情": {
                "推荐者(9-10分)": format!("{promoters}人（{prom_pct}%）"),
                "被动者(7-8分)": format!("{passives}人（{pass_pct}%）"),
                "批评者(0-6分)": format!("{detractors}人（{detr_pct}%）")
            },
            "CSAT满意度": format!("{csat_pct}%（{}）", csat_grade(csat_pct)),
            "忠诚度洞察": {
                "推荐者价值": format!("贡献约{prom_pct}%的口碑传播和复购"),
                "被动者风险": format!("{pass_pct}%的客户随时可能流向竞品，需主动维系"),
                "批评者负面影响": format!("预估影响约{}人的购买决策", (detractors as f64 * 5.6).round() as i64),
                "转化优先级": "被动者→推荐者成本最低，批评者→中立是当务之急"
            },
            "批评者主要问题": top_issues.iter().take(5).collect::<Vec<_>>(),
            "改善优先级": top_issues.first().map(|issue| format!("重点解决：{issue}（批评者最常提及）"))
        });

        outcome(output, "NPS与CSAT分析已完成")
    }
}

pub fn service_sentiment_analyzer_spec() -> SkillSpec {
    SkillSpec::new(
        "service_sentiment_analyzer",
        "客服情感分析",
        "对客户消息进行0-10情感强度评分，输出紧急等级、优先级和处理建议",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "customer_text",
        "客户消息文本",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音",
        string_schema(),
    ))
    .with_input(
        SkillInputField::optional("channel", "来源渠道：im/review/complaint", string_schema())
            .with_default(json!("im")),
    )
    .with_input(SkillInputField::optional(
        "batch_texts",
        "批量分析多条消息（最多20条）",
        string_array_schema(),
    ))
    .with_tag("deterministic")
}

pub struct ServiceSentimentAnalyzer {
    spec: SkillSpec,
}

impl ServiceSentimentAnalyzer {
    pub fn new() -> Self {
        Self {
            spec: service_sentiment_analyzer_spec(),
        }
    }
}

impl Default for ServiceSentimentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceSentimentAnalyzer {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let channel = optional_string_param(&params, "channel")?.unwrap_or_else(|| "im".to_owned());
        let channel_display = match channel.as_str() {
            "im" => "旺旺/IM",
            "review" => "买家评价",
            "complaint" => "投诉中心",
            other => other,
        };

        if let Some(texts) = optional_string_vec_param_or_empty(&params, "batch_texts")
            .ok()
            .filter(|v| !v.is_empty())
        {
            let details = texts
                .iter()
                .take(20)
                .enumerate()
                .map(|(index, text)| {
                    let scored = score_sentiment(text);
                    json!({
                        "序号": index + 1,
                        "文本摘要": truncate_chars(text, 50),
                        "情感强度": scored.intensity,
                        "紧急等级": scored.urgency,
                        "处理优先级": scored.priority,
                        "触发关键词": scored.triggered
                    })
                })
                .collect::<Vec<_>>();
            let scores = details
                .iter()
                .filter_map(|item| item.get("情感强度").and_then(Value::as_f64))
                .collect::<Vec<_>>();
            let avg = if scores.is_empty() {
                5.0
            } else {
                round1(scores.iter().sum::<f64>() / scores.len() as f64)
            };
            let high_risk = details
                .iter()
                .filter(|item| item.get("处理优先级").and_then(Value::as_i64) == Some(1))
                .count();
            let negative = details
                .iter()
                .filter(|item| item.get("处理优先级").and_then(Value::as_i64) == Some(2))
                .count();
            let positive = details
                .iter()
                .filter(|item| {
                    item.get("处理优先级")
                        .and_then(Value::as_i64)
                        .is_some_and(|p| p >= 4)
                })
                .count();
            return outcome(
                json!({
                    "status": "success",
                    "has_data": true,
                    "adapter_status": "manual_input",
                    "data_readiness": "user_batch_provided",
                    "summary": "批量情感分析已完成",
                    "分析模式": "批量分析",
                    "消息数量": details.len(),
                    "平台": platform,
                    "渠道": channel_display,
                    "整体情感评分": avg,
                    "分布统计": {
                        "高危需立即处理": high_risk,
                        "负面优先处理": negative,
                        "正常处理": details.len().saturating_sub(high_risk + negative + positive),
                        "正面/高满意": positive
                    },
                    "建议": if high_risk > 0 { format!("有{high_risk}条高危消息需立即处理") } else { "无高危消息，正常处理队列".to_owned() },
                    "明细": details
                }),
                "批量情感分析已完成",
            );
        }

        let text = optional_string_param(&params, "customer_text")?
            .ok_or_else(|| SkillError::missing_parameter("customer_text"))?;
        let scored = score_sentiment(&text);
        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "manual_input",
                "data_readiness": "user_text_provided",
                "summary": "客服情感分析已完成",
                "平台": platform,
                "渠道": channel_display,
                "客户消息摘要": truncate_chars(&text, 80),
                "情感强度评分": format!("{}/10", scored.intensity),
                "情感状态": sentiment_state(scored.intensity),
                "紧急等级": scored.urgency,
                "处理优先级": scored.priority,
                "触发关键词": scored.triggered,
                "电商行业参考": "高危=需2分钟内响应，负面=30分钟内，中性=2小时内SLA",
                "建议": sentiment_advice(scored.intensity)
            }),
            "客服情感分析已完成",
        )
    }
}

pub fn service_nps_driver_analysis_spec() -> SkillSpec {
    SkillSpec::new(
        "service_nps_driver_analysis",
        "NPS驱动因素分析",
        "4类投诉来源根因诊断，计算各类别NPS影响系数，识别最大满意度驱动因素",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "promoter_count",
        "推荐者数量（评分9-10）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "detractor_count",
        "贬低者数量（评分0-6）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "passive_count",
        "被动者数量（评分7-8）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "complaint_distribution",
        "投诉分布：{物流配送: 120, 商品质量: 85}",
        object_schema(),
    ))
    .with_input(SkillInputField::optional(
        "total_tickets",
        "总工单数",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音",
        string_schema(),
    ))
    .with_tag("deterministic")
}

pub struct ServiceNpsDriverAnalysis {
    spec: SkillSpec,
}

impl ServiceNpsDriverAnalysis {
    pub fn new() -> Self {
        Self {
            spec: service_nps_driver_analysis_spec(),
        }
    }
}

impl Default for ServiceNpsDriverAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceNpsDriverAnalysis {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let mut promoter = optional_i64_param(&params, "promoter_count")?.unwrap_or(0);
        let mut detractor = optional_i64_param(&params, "detractor_count")?.unwrap_or(0);
        let mut passive = optional_i64_param(&params, "passive_count")?.unwrap_or(0);
        let has_user_nps_counts = params.get("promoter_count").is_some()
            || params.get("detractor_count").is_some()
            || params.get("passive_count").is_some();
        if promoter + detractor + passive == 0 {
            promoter = 45;
            passive = 35;
            detractor = 20;
        }

        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let mut dist = parse_string_i64_map(params.get("complaint_distribution"))?;
        let has_user_complaint_distribution = !dist.is_empty();
        if dist.is_empty() {
            dist.extend([
                ("物流配送".to_owned(), 38),
                ("商品质量".to_owned(), 25),
                ("售后退款".to_owned(), 18),
                ("客服响应".to_owned(), 12),
                ("商品描述".to_owned(), 7),
            ]);
        }

        let total_tickets = optional_i64_param(&params, "total_tickets")?
            .unwrap_or_else(|| dist.values().sum::<i64>())
            .max(1);
        let total_respondents = promoter + detractor + passive;
        let promoter_pct = promoter as f64 / total_respondents as f64;
        let detractor_pct = detractor as f64 / total_respondents as f64;
        let nps_score = round1((promoter_pct - detractor_pct) * 100.0);

        let mut rows = dist.into_iter().map(|(category, count)| {
            let rate = count as f64 / total_tickets as f64;
            let coeff = nps_coefficient(&category);
            let impact = round2(rate * coeff * 100.0);
            json!({
                "问题类别": category,
                "投诉量": count,
                "投诉率": format!("{:.1}%", rate * 100.0),
                "NPS拖累": format!("{impact:+.2}分"),
                "处理优先级": if impact.abs() > 3.0 { "P0" } else if impact.abs() > 1.5 { "P1" } else { "P2" },
                "_impact_sort": impact
            })
        }).collect::<Vec<_>>();
        rows.sort_by(|a, b| {
            a.get("_impact_sort")
                .and_then(Value::as_f64)
                .partial_cmp(&b.get("_impact_sort").and_then(Value::as_f64))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let max_uplift = round1(
            rows.iter()
                .filter_map(|row| row.get("_impact_sort").and_then(Value::as_f64))
                .sum::<f64>()
                .abs(),
        );
        for row in &mut rows {
            if let Some(map) = row.as_object_mut() {
                map.remove("_impact_sort");
            }
        }
        let industry_avg = match platform.as_str() {
            "淘宝" => 45,
            "京东" => 52,
            "拼多多" => 30,
            "抖音" => 40,
            _ => 40,
        };
        let nps_gap = round1(nps_score - industry_avg as f64);
        let max_pain = rows
            .first()
            .and_then(|row| row.get("问题类别"))
            .cloned()
            .unwrap_or_else(|| json!("无数据"));
        let data_readiness = if has_user_nps_counts && has_user_complaint_distribution {
            "user_provided"
        } else {
            "calculated_from_defaults"
        };

        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "manual_input",
                "data_readiness": data_readiness,
                "summary": "NPS驱动因素分析已完成",
                "平台": platform,
                "NPS分数": format!("{nps_score:+.1}"),
                "NPS构成": {
                    "推荐者(9-10分)": format!("{:.0}%", promoter_pct * 100.0),
                    "被动者(7-8分)": format!("{:.0}%", passive as f64 / total_respondents as f64 * 100.0),
                    "贬低者(0-6分)": format!("{:.0}%", detractor_pct * 100.0),
                    "受访总数": total_respondents
                },
                "行业基准": {
                    "行业平均NPS": industry_avg,
                    "差距": format!("{nps_gap:+.1}分"),
                    "位置": if nps_gap >= 0.0 { "优于行业" } else { "低于行业" }
                },
                "驱动因素分析": rows,
                "理论最大NPS提升": format!("+{max_uplift}分（解决全部投诉问题）"),
                "最大痛点": max_pain,
                "总工单数": total_tickets
            }),
            "NPS驱动因素分析已完成",
        )
    }
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn optional_string_vec_param_or_empty(
    params: &Value,
    name: &str,
) -> Result<Vec<String>, SkillError> {
    Ok(optional_string_vec_param(params, name)?.unwrap_or_default())
}

fn nps_grade(nps: f64) -> &'static str {
    if nps >= 70.0 {
        "极优秀（世界级）"
    } else if nps >= 50.0 {
        "优秀"
    } else if nps >= 30.0 {
        "良好"
    } else if nps >= 0.0 {
        "正常"
    } else {
        "需改善（批评者过多）"
    }
}

fn csat_grade(csat: f64) -> &'static str {
    if csat >= 90.0 {
        "优秀"
    } else if csat >= 75.0 {
        "良好"
    } else {
        "需改善"
    }
}

struct SentimentScore {
    intensity: f64,
    urgency: &'static str,
    priority: i64,
    triggered: Vec<&'static str>,
}

fn score_sentiment(text: &str) -> SentimentScore {
    let keywords = [
        ("骗子", -3.0),
        ("欺诈", -3.0),
        ("举报", -2.8),
        ("曝光", -2.5),
        ("投诉到", -2.5),
        ("差评", -2.0),
        ("投诉", -2.0),
        ("维权", -2.0),
        ("退款", -1.5),
        ("破损", -1.5),
        ("质量差", -1.5),
        ("假货", -2.0),
        ("不满意", -1.2),
        ("等了好久", -1.0),
        ("还没到", -1.0),
        ("赔偿", -1.5),
        ("退货", -0.8),
        ("问题", -0.5),
        ("延误", -0.8),
        ("发错", -1.0),
        ("缺货", -0.8),
        ("好评", 0.8),
        ("满意", 0.8),
        ("感谢", 0.6),
        ("棒", 0.5),
        ("超快", 0.5),
        ("完美", 1.0),
        ("推荐", 0.8),
        ("再次购买", 1.0),
        ("愤怒", -0.9),
        ("气死", -0.9),
        ("太差了", -0.75),
        ("垃圾", -0.75),
        ("失望", -0.6),
        ("挺好", 0.3),
        ("不错", 0.3),
        ("很好", 0.45),
        ("太好了", 0.6),
    ];
    let mut score = 5.0;
    let mut triggered = Vec::new();
    for (keyword, delta) in keywords {
        if text.contains(keyword) {
            score += delta;
            triggered.push(keyword);
        }
    }
    if text.chars().count() > 200 && score < 4.0 {
        score -= 0.5;
    }
    let exclamations = text.matches('！').count() + text.matches("!!").count();
    if exclamations >= 2 && score < 5.0 {
        score -= 0.3 * exclamations.min(3) as f64;
    }
    let intensity = round1(score.clamp(0.0, 10.0));
    let (urgency, priority) = if intensity < 2.0 {
        ("高危-立即介入（2分钟内）", 1)
    } else if intensity < 3.5 {
        ("负面-优先处理（30分钟内）", 2)
    } else if intensity < 5.5 {
        ("中性-正常排队（2小时内）", 3)
    } else if intensity < 7.5 {
        ("正面-无需人工干预", 4)
    } else {
        ("高满意-可邀请好评", 5)
    };
    SentimentScore {
        intensity,
        urgency,
        priority,
        triggered: triggered.into_iter().take(5).collect(),
    }
}

fn sentiment_state(intensity: f64) -> &'static str {
    if intensity < 2.0 {
        "极度负面"
    } else if intensity < 4.0 {
        "负面"
    } else if intensity < 6.0 {
        "中性"
    } else if intensity < 8.0 {
        "正面"
    } else {
        "极度正面"
    }
}

fn sentiment_advice(intensity: f64) -> &'static str {
    if intensity < 2.0 {
        "立即人工介入，先承认问题并给出明确补偿或升级路径"
    } else if intensity < 4.0 {
        "优先响应，围绕事实核验、解决时限和补偿预期沟通"
    } else if intensity < 6.0 {
        "按常规SLA处理，保持信息透明"
    } else if intensity < 8.0 {
        "维持友好沟通，可引导收藏或复购"
    } else {
        "可邀请评价或沉淀为好评素材"
    }
}

fn truncate_chars(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

fn parse_string_i64_map(value: Option<&Value>) -> Result<BTreeMap<String, i64>, SkillError> {
    let mut out = BTreeMap::new();
    let Some(value) = value else {
        return Ok(out);
    };
    let object = value
        .as_object()
        .ok_or_else(|| SkillError::invalid_parameter("complaint_distribution", "object"))?;
    for (key, value) in object {
        let count = value.as_i64().ok_or_else(|| {
            SkillError::invalid_parameter("complaint_distribution", "integer counts")
        })?;
        out.insert(key.clone(), count);
    }
    Ok(out)
}

fn nps_coefficient(category: &str) -> f64 {
    match category {
        "物流配送" => -0.35,
        "商品质量" => -0.48,
        "客服响应" => -0.22,
        "售后退款" => -0.30,
        "包装体验" => -0.12,
        "价格感知" => -0.18,
        "商品描述" => -0.20,
        "平台体验" => -0.10,
        _ => -0.15,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::Skill;

    fn assert_success_contract(output: &Value, summary: &str) {
        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(output.get("summary").and_then(Value::as_str), Some(summary));
    }

    #[tokio::test]
    async fn service_ticket_handler_marks_success_for_user_issue() {
        let outcome = ServiceTicketHandler::new()
            .run(
                json!({
                    "issue_type": "物流延迟",
                    "customer_message": "包裹还没到，客户很生气并威胁投诉平台",
                    "customer_emotion": "愤怒",
                    "platform": "淘宝",
                    "product_ref": "product:11111111-1111-1111-1111-111111111111"
                }),
                SkillContext::default(),
            )
            .await
            .expect("service ticket should run");
        let output = outcome.output;

        assert_success_contract(&output, "客服工单处理方案已生成");
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert_eq!(
            output.get("问题类型").and_then(Value::as_str),
            Some("物流延迟")
        );
        assert!(output.get("客服话术").is_some());
        assert!(output.get("升级条件").is_some());
        assert_eq!(
            output
                .pointer("/内部工单字段/product_ref")
                .and_then(Value::as_str),
            Some("product:11111111-1111-1111-1111-111111111111")
        );
    }

    #[tokio::test]
    async fn service_faq_playbook_marks_success_for_policy_input() {
        let outcome = ServiceFaqPlaybook::new()
            .run(
                json!({
                    "platform": "京东",
                    "product_ref": "product:11111111-1111-1111-1111-111111111111",
                    "questions": ["什么时候发货？", "支持退货吗？"],
                    "policies": {
                        "发货时效": "48小时内",
                        "退货政策": "7天无理由"
                    }
                }),
                SkillContext::default(),
            )
            .await
            .expect("service FAQ should run");
        let output = outcome.output;

        assert_success_contract(&output, "客服FAQ话术库已生成");
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_policy_provided")
        );
        assert_eq!(output.get("平台").and_then(Value::as_str), Some("京东"));
        assert_eq!(
            output.get("商品引用").and_then(Value::as_str),
            Some("product:11111111-1111-1111-1111-111111111111")
        );
        assert!(
            output
                .get("FAQ")
                .and_then(Value::as_array)
                .is_some_and(|items| items.len() == 2)
        );
    }

    #[tokio::test]
    async fn service_escalation_flow_marks_success_for_complaint() {
        let outcome = ServiceEscalationFlow::new()
            .run(
                json!({
                    "issue_description": "客户连续三次反馈质量问题，并表示要投诉平台和曝光",
                    "customer_emotion": "威胁投诉",
                    "previous_contacts": 3,
                    "customer_history": "高价值客户",
                    "product_ref": "product:11111111-1111-1111-1111-111111111111"
                }),
                SkillContext::default(),
            )
            .await
            .expect("service escalation should run");
        let output = outcome.output;

        assert_success_contract(&output, "客服升级处理流程已生成");
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert_eq!(output.get("严重度").and_then(Value::as_str), Some("S1"));
        assert_eq!(
            output.get("需审批/复核").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            output.get("商品引用").and_then(Value::as_str),
            Some("product:11111111-1111-1111-1111-111111111111")
        );
    }

    #[tokio::test]
    async fn service_dsr_improvement_marks_success_for_scores() {
        let outcome = ServiceDsrImprovement::new()
            .run(
                json!({
                    "description_score": 4.4,
                    "service_score": 4.7,
                    "logistics_score": 4.5,
                    "platform": "抖音"
                }),
                SkillContext::default(),
            )
            .await
            .expect("service DSR should run");
        let output = outcome.output;

        assert_success_contract(&output, "DSR提升方案已生成");
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert_eq!(
            output.get("优先改善维度").and_then(Value::as_str),
            Some("描述相符")
        );
        assert!(output.get("改善方案").is_some());
    }

    #[tokio::test]
    async fn service_return_handler_marks_success_for_return_case() {
        let outcome = ServiceReturnHandler::new()
            .run(
                json!({
                    "reason": "质量问题",
                    "order_amount": 268.0,
                    "days_since_receipt": 9,
                    "product_ref": "product:11111111-1111-1111-1111-111111111111"
                }),
                SkillContext::default(),
            )
            .await
            .expect("service return should run");
        let output = outcome.output;

        assert_success_contract(&output, "退换货处理方案已生成");
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert_eq!(output.get("责任方").and_then(Value::as_str), Some("卖家"));
        assert!(output.get("处理方案").is_some());
        assert_eq!(
            output.get("商品引用").and_then(Value::as_str),
            Some("product:11111111-1111-1111-1111-111111111111")
        );
    }

    #[tokio::test]
    async fn service_query_product_uses_explicit_product_context() {
        let context = SkillContext::new()
            .with_metadata("request.context.product_ids", json!("product-1"))
            .with_metadata("request.context.refs", json!("product:product-1"))
            .with_metadata(
                "request.context.selected_summary",
                json!("商品：冷萃咖啡。规格：12瓶装。卖点：低糖、即饮、适合办公室囤货。"),
            )
            .with_metadata(
                "request.context.sources_json",
                json!(
                    json!([
                        {
                            "source_type": "product",
                            "id": "product-1",
                            "product_id": "product-1",
                            "title": "冷萃咖啡"
                        }
                    ])
                    .to_string()
                ),
            );
        let outcome = ServiceQueryProduct::new()
            .run(json!({"product_name": "冷萃咖啡"}), context)
            .await
            .expect("service query product should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("context_metadata")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("explicit_product_context")
        );
        assert!(
            output
                .get("productContextSummary")
                .and_then(Value::as_str)
                .is_some_and(|summary| summary.contains("冷萃咖啡"))
        );
        assert_eq!(
            output.pointer("/answerPolicy/no_default_product"),
            Some(&json!(true))
        );
    }

    #[tokio::test]
    async fn service_query_product_accepts_product_ref_filter() {
        let product_id = "11111111-1111-1111-1111-111111111111";
        let context = SkillContext::new()
            .with_metadata("request.context.product_ids", json!(product_id))
            .with_metadata(
                "request.context.refs",
                json!("product:11111111-1111-1111-1111-111111111111"),
            )
            .with_metadata(
                "request.context.selected_summary",
                json!("Product source product:11111111-1111-1111-1111-111111111111 | name=五一咖啡组合 | platform=抖音 | category=咖啡 | brand=千乐 | price=59 CNY | status=active | description=低糖便携"),
            )
            .with_metadata(
                "request.context.sources_json",
                json!(
                    json!([
                        {
                            "source_type": "product",
                            "id": product_id,
                            "product_id": product_id,
                            "title": "五一咖啡组合"
                        }
                    ])
                    .to_string()
                ),
            );
        let outcome = ServiceQueryProduct::new()
            .run(
                json!({"product_ref": "product:11111111-1111-1111-1111-111111111111"}),
                context,
            )
            .await
            .expect("service query product should accept product_ref");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.pointer("/query/product_ref").and_then(Value::as_str),
            Some("product:11111111-1111-1111-1111-111111111111")
        );
        assert_eq!(
            output
                .pointer("/matchedProductIds/0")
                .and_then(Value::as_str),
            Some(product_id)
        );
        assert_eq!(
            output.pointer("/refs/0").and_then(Value::as_str),
            Some("product:11111111-1111-1111-1111-111111111111")
        );
        assert_eq!(
            output.pointer("/answerPolicy/no_default_product"),
            Some(&json!(true))
        );
    }

    #[tokio::test]
    async fn service_query_product_defers_when_product_ref_misses_selected_context() {
        let context = SkillContext::new()
            .with_metadata("request.context.product_ids", json!("product-1"))
            .with_metadata("request.context.refs", json!("product:product-1"))
            .with_metadata(
                "request.context.selected_summary",
                json!("Product source product:product-1 | name=冷萃咖啡 | platform=抖音"),
            );
        let outcome = ServiceQueryProduct::new()
            .run(json!({"product_ref": "product:product-2"}), context)
            .await
            .expect("service query product should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("context_filter_not_matched")
        );
        assert_eq!(
            output.pointer("/query/product_ref").and_then(Value::as_str),
            Some("product:product-2")
        );
    }

    #[tokio::test]
    async fn service_query_product_defers_without_explicit_product_context() {
        let outcome = ServiceQueryProduct::new()
            .run(json!({}), SkillContext::default())
            .await
            .expect("service query product should not fail without context");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(false));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("context_required")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("explicit_product_context_required")
        );
        assert_eq!(
            output.pointer("/answerPolicy/no_default_product"),
            Some(&json!(true))
        );
    }

    #[tokio::test]
    async fn service_query_product_defers_when_filter_misses_selected_context() {
        let context = SkillContext::new()
            .with_metadata("request.context.product_ids", json!("product-1"))
            .with_metadata(
                "request.context.selected_summary",
                json!("商品：冷萃咖啡。规格：12瓶装。"),
            );
        let outcome = ServiceQueryProduct::new()
            .run(json!({"product_name": "洗衣液"}), context)
            .await
            .expect("service query product should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("deferred")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("context_filter_not_matched")
        );
        assert!(
            output
                .get("deferred_reason")
                .and_then(Value::as_str)
                .is_some_and(|reason| reason.contains("不会改用默认商品"))
        );
    }

    #[tokio::test]
    async fn service_nps_analyzer_marks_success_for_score_distribution() {
        let outcome = ServiceNpsAnalyzer::new()
            .run(
                json!({
                    "score_distribution": {
                        "10": 42,
                        "9": 18,
                        "8": 20,
                        "6": 9,
                        "4": 3
                    },
                    "top_issues": ["物流慢", "客服响应慢"]
                }),
                SkillContext::default(),
            )
            .await
            .expect("service NPS should run");
        let output = outcome.output;

        assert_success_contract(&output, "NPS与CSAT分析已完成");
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert!(output.get("NPS净推荐值").is_some());
        assert!(output.get("忠诚度洞察").is_some());
    }

    #[tokio::test]
    async fn service_sentiment_analyzer_marks_success_for_customer_text() {
        let outcome = ServiceSentimentAnalyzer::new()
            .run(
                json!({
                    "customer_text": "物流太慢了，我要投诉并给差评！",
                    "channel": "complaint",
                    "platform": "拼多多"
                }),
                SkillContext::default(),
            )
            .await
            .expect("service sentiment should run");
        let output = outcome.output;

        assert_success_contract(&output, "客服情感分析已完成");
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_text_provided")
        );
        assert_eq!(output.get("渠道").and_then(Value::as_str), Some("投诉中心"));
        assert!(output.get("处理优先级").is_some());
    }

    #[tokio::test]
    async fn service_nps_driver_analysis_marks_defaults_disclosed() {
        let outcome = ServiceNpsDriverAnalysis::new()
            .run(json!({}), SkillContext::default())
            .await
            .expect("service NPS driver should run");
        let output = outcome.output;

        assert_success_contract(&output, "NPS驱动因素分析已完成");
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("calculated_from_defaults")
        );
        assert!(output.get("驱动因素分析").is_some());
        assert!(output.get("最大痛点").is_some());
    }
}
