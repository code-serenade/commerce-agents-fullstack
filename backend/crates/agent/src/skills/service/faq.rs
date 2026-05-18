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

