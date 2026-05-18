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

