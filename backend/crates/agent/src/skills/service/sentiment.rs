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

