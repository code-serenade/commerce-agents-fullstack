use super::*;

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
