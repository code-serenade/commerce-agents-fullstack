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

