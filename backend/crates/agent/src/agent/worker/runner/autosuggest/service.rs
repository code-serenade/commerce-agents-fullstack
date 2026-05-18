fn autosuggest_service_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if is_service_benchmark_chain_intent(text)
        && let Some(primary) = autosuggest_service_native(text, task)
    {
        return vec![primary, autosuggest_service_benchmark_search(text, task)];
    }

    if let Some(primary) = autosuggest_service_native(text, task)
        && is_service_platform_policy_search_chain_intent(text, &primary.skill_id)
    {
        return vec![
            primary,
            autosuggest_service_platform_policy_search(text, task),
        ];
    }

    if let Some(primary) = autosuggest_service_native(text, task)
        && is_service_review_search_chain_intent(text, &primary.skill_id)
    {
        return vec![primary, autosuggest_service_review_search(text, task)];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    autosuggest_service_native(text, task).into_iter().collect()
}

fn autosuggest_service(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_service_requests(text, task).into_iter().next()
}

fn autosuggest_service_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "driver",
            "驱动因素",
            "满意度原因",
            "投诉分类",
            "nps驱动",
            "nps driver",
        ],
    ) {
        return Some(autosuggest_request(
            "service_nps_driver_analysis",
            serde_json::json!({
                "platform": infer_platform(text),
            }),
            "根据满意度驱动因素意图自动补偿NPS驱动分析 skill",
        ));
    }

    if contains_any(
        text,
        &["faq", "FAQ", "常见问题", "问答库", "标准话术", "话术库"],
    ) {
        return Some(autosuggest_request(
            "service_faq_playbook",
            serde_json::json!({
                "questions": infer_faq_questions(text),
                "platform": infer_platform(text),
                "product_ref": selected_product_ref(task),
            }),
            "根据FAQ/标准话术意图自动补偿FAQ话术库 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "查询商品",
            "查商品",
            "商品信息",
            "商品资料",
            "商品上下文",
            "商品详情",
            "商品知识",
            "引用商品",
            "选中商品",
        ],
    ) {
        return Some(autosuggest_request(
            "service_query_product",
            serde_json::json!({
                "product_ref": selected_product_ref(task),
                "product_name": first_selected_product(task).map(|product| product.name),
            }),
            "根据客服商品查询意图自动补偿商品上下文查询 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "ticket",
            "工单",
            "话术",
            "客服话术",
            "安抚",
            "客户投诉",
            "生气",
            "很气",
            "物流延迟",
            "发错",
            "质量问题",
        ],
    ) {
        return Some(autosuggest_request(
            "service_ticket_handler",
            serde_json::json!({
                "issue_type": infer_service_issue_type(text),
                "customer_message": compact_task_text(&task.description),
                "customer_emotion": infer_service_emotion(text),
                "platform": infer_platform(text),
                "product_ref": selected_product_ref(task),
            }),
            "根据客服工单/话术意图自动补偿工单处理 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "upgrade",
            "escalation",
            "升级",
            "主管",
            "投诉平台",
            "平台介入",
            "曝光",
            "举报",
            "维权",
            "威胁投诉",
        ],
    ) {
        return Some(autosuggest_request(
            "service_escalation_flow",
            serde_json::json!({
                "issue_description": compact_task_text(&task.description),
                "customer_emotion": infer_service_emotion(text),
                "previous_contacts": numbers_in_text(text).first().copied().map(|value| value.round() as i64),
                "product_ref": selected_product_ref(task),
            }),
            "根据客服升级/投诉意图自动补偿升级处理 skill",
        ));
    }

    if contains_any(text, &["return", "refund", "退款", "退货", "换货", "售后"]) {
        return Some(autosuggest_request(
            "service_return_handler",
            serde_json::json!({
                "reason": infer_return_reason(text),
                "days_since_receipt": numbers_in_text(text).first().copied().map(|value| value.round() as i64),
                "product_ref": selected_product_ref(task),
            }),
            "根据退换货/售后意图自动补偿退换货处理 skill",
        ));
    }

    if contains_any(text, &["nps", "NPS", "满意度", "推荐值", "客户忠诚"]) {
        return Some(autosuggest_request(
            "service_nps_analyzer",
            serde_json::json!({
                "platform": infer_platform(text),
            }),
            "根据NPS/满意度意图自动补偿NPS分析 skill",
        ));
    }

    if contains_any(text, &["情绪", "情感", "差评", "不满意", "愤怒", "投诉"]) {
        return Some(autosuggest_request(
            "service_sentiment_analyzer",
            serde_json::json!({
                "customer_text": compact_task_text(&task.description),
                "platform": infer_platform(text),
            }),
            "根据客服情绪识别意图自动补偿情感分析 skill",
        ));
    }

    Some(autosuggest_request(
        "service_dsr_improvement",
        serde_json::json!({
            "platform": infer_platform(text),
        }),
        "根据客服岗位任务自动补偿 DSR 提升 skill",
    ))
}

fn is_service_benchmark_chain_intent(text: &str) -> bool {
    let has_service_metric = contains_any(
        text,
        &[
            "dsr",
            "DSR",
            "服务评分",
            "店铺评分",
            "评分低",
            "满意度",
            "nps",
            "NPS",
            "推荐值",
            "客户忠诚",
            "驱动因素",
            "投诉分类",
            "客服响应",
        ],
    );
    has_service_metric
        && (is_industry_benchmark_search_intent(text)
            || contains_any(
                text,
                &["行业基准", "行业均值", "行业平均", "实时基准", "最新基准"],
            ))
}

fn autosuggest_service_benchmark_search(text: &str, task: &Task) -> ParsedSkillRequest {
    let input = search_industry_benchmarks_input(text, task);
    autosuggest_request(
        "search_industry_benchmarks",
        serde_json::Value::Object(input),
        "根据客服DSR/NPS分析中的行业基准需求自动追加外部搜索 skill",
    )
}

fn is_service_platform_policy_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    matches!(
        native_skill_id,
        "service_ticket_handler"
            | "service_faq_playbook"
            | "service_escalation_flow"
            | "service_return_handler"
    ) && is_service_platform_policy_search_intent(text)
        && has_service_policy_work_intent(text, native_skill_id)
}

fn is_service_platform_policy_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "平台规则",
            "平台政策",
            "平台公告",
            "最新规则",
            "最新政策",
            "售后规则",
            "售后政策",
            "退货规则",
            "退款规则",
            "退换货规则",
            "换货规则",
            "运费规则",
            "运费险",
            "赔付标准",
            "赔偿标准",
            "补偿标准",
            "投诉规则",
            "平台介入规则",
            "纠纷规则",
            "纠纷处理",
            "七天无理由",
            "7天无理由",
            "假一赔",
            "仅退款",
            "售后时效",
            "policy",
        ],
    ) && (has_external_search_intent(text)
        || contains_any(text, &["最新", "规则", "政策", "标准", "公告"]))
}

fn has_service_policy_work_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "service_ticket_handler" => contains_any(
            text,
            &[
                "工单", "处理", "话术", "回复", "安抚", "客户", "用户", "客服", "投诉", "ticket",
            ],
        ),
        "service_faq_playbook" => contains_any(
            text,
            &[
                "faq",
                "FAQ",
                "常见问题",
                "问答库",
                "标准话术",
                "话术库",
                "客服话术",
            ],
        ),
        "service_escalation_flow" => contains_any(
            text,
            &[
                "升级",
                "主管",
                "投诉平台",
                "平台介入",
                "维权",
                "举报",
                "纠纷",
                "escalation",
            ],
        ),
        "service_return_handler" => contains_any(
            text,
            &[
                "退款",
                "退货",
                "换货",
                "售后",
                "退换货",
                "返修",
                "赔付",
                "运费",
                "return",
                "refund",
            ],
        ),
        _ => false,
    }
}

fn autosuggest_service_platform_policy_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_platform_policy",
        serde_json::Value::Object(service_platform_policy_search_input(text, task)),
        "根据客服处理中的平台售后/赔付/投诉规则诉求自动追加平台规则搜索 skill",
    )
}

fn service_platform_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert("policy_type".to_string(), serde_json::json!("general"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_service_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn is_service_review_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    matches!(
        native_skill_id,
        "service_sentiment_analyzer"
            | "service_ticket_handler"
            | "service_return_handler"
            | "service_nps_driver_analysis"
    ) && is_product_review_search_intent(text)
        && has_service_review_work_intent(text, native_skill_id)
}

fn has_service_review_work_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "service_sentiment_analyzer" => contains_any(
            text,
            &[
                "分析",
                "识别",
                "判断",
                "研判",
                "情绪",
                "情感",
                "sentiment",
                "analyze",
            ],
        ),
        "service_ticket_handler" => contains_any(
            text,
            &[
                "工单",
                "处理",
                "话术",
                "回复",
                "安抚",
                "客户说",
                "用户说",
                "ticket",
            ],
        ),
        "service_return_handler" => contains_any(
            text,
            &[
                "退款",
                "退货",
                "换货",
                "售后",
                "退换货",
                "返修",
                "return",
                "refund",
            ],
        ),
        "service_nps_driver_analysis" => contains_any(
            text,
            &[
                "驱动因素",
                "满意度原因",
                "投诉分类",
                "nps驱动",
                "NPS",
                "nps",
            ],
        ),
        _ => false,
    }
}

fn autosuggest_service_review_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_product_reviews",
        serde_json::Value::Object(service_product_reviews_search_input(text, task)),
        "根据客服处理中的真实评价/口碑/差评诉求自动追加外部评价搜索 skill",
    )
}

fn service_product_reviews_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_product_reviews_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(service_review_search_subject(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_service_review_search_focus(text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_service_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn service_review_search_subject(text: &str, task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| infer_search_query_for_task(text, task))
}

fn infer_service_review_search_focus(text: &str) -> &'static str {
    if contains_any(
        text,
        &["差评", "负面", "投诉", "吐槽", "不满意", "negative"],
    ) {
        "negative"
    } else if contains_any(text, &["好评", "正面", "满意", "positive"]) {
        "positive"
    } else {
        "overall"
    }
}

fn infer_service_search_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}
