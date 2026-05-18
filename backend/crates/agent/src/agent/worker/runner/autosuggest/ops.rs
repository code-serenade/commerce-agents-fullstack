fn autosuggest_ops_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_ops_native(text, task) {
        if is_ops_search_chain_intent(text, &primary.skill_id) {
            let search = autosuggest_ops_search(text, task, &primary.skill_id);
            return vec![primary, search];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_request(
        "ops_execution_plan",
        serde_json::Value::Object(infer_execution_plan_input(text, task)),
        "根据运营岗位任务自动补偿执行计划 skill",
    )]
}

fn autosuggest_ops(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_ops_requests(text, task).into_iter().next()
}


fn autosuggest_ops_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "记住",
            "记下来",
            "保存记忆",
            "保存这个结论",
            "保存这个事实",
            "记下这个决策",
            "记住这个风险",
            "remember this",
        ],
    ) {
        return Some(autosuggest_request(
            "coordination_save_memory",
            serde_json::json!({
                "key": infer_memory_key(text),
                "content": infer_memory_content(text, task),
                "memory_type": infer_memory_type(text),
            }),
            "根据记忆保存意图自动补偿协调保存记忆 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "创建任务",
            "建个任务",
            "新建任务",
            "安排任务",
            "分派任务",
            "待办",
            "todo",
            "to-do",
        ],
    ) {
        return Some(autosuggest_request(
            "coordination_create_task",
            serde_json::json!({
                "title": infer_task_title(text, task),
                "description": compact_task_text(&task.description),
                "owner_role": infer_task_owner_role(text),
                "priority": infer_task_priority(text),
                "acceptance_criteria": infer_task_acceptance(text),
            }),
            "根据建任务/待办/任务安排意图自动补偿协调建任务 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "channel",
            "渠道",
            "投放渠道",
            "流量渠道",
            "渠道分配",
            "渠道策略",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_channel_strategy",
            serde_json::Value::Object(infer_channel_strategy_input(text, task)),
            "根据渠道分配/渠道策略意图自动补偿渠道策略 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "listing",
            "上架",
            "上新文案",
            "上架文案",
            "标题卖点",
            "商品标题",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_listing_copy",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "features": selected_product_features_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据上架文案/标题卖点意图自动补偿上架文案 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "assortment",
            "sku组合",
            "sku 组合",
            "货盘",
            "选品组合",
            "组合规划",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_assortment_planning",
            serde_json::Value::Object(infer_assortment_planning_input(text, task)),
            "根据选品组合/货盘规划意图自动补偿选品规划 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "smart pricing",
            "dynamic pricing",
            "智能定价",
            "动态定价",
            "价格优化",
            "利润定价",
            "最优定价",
            "调价建议",
            "涨价建议",
            "降价建议",
            "清仓定价",
            "库存压力价",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_smart_pricing",
            serde_json::Value::Object(infer_smart_pricing_input(text, task)),
            "根据智能定价/动态调价意图自动补偿智能定价 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "inventory optimizer",
            "inventory optimization",
            "库存优化",
            "库存健康",
            "安全库存",
            "再订货点",
            "经济订货量",
            "周转天数",
            "EOQ",
            "eoq",
            "ROP",
            "rop",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_inventory_optimizer",
            serde_json::Value::Object(infer_inventory_optimizer_input(text, task)),
            "根据库存优化/EOQ/再订货点意图自动补偿库存优化 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "ABC-XYZ",
            "abc-xyz",
            "ABCXYZ",
            "abcxyz",
            "ABC分类",
            "abc分类",
            "XYZ分类",
            "xyz分类",
            "ABC库存",
            "XYZ库存",
            "9格矩阵",
            "九宫格库存",
            "库存分类矩阵",
            "SKU分层",
            "sku分层",
            "高价值SKU",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_abc_xyz_classifier",
            serde_json::Value::Object(infer_abc_xyz_classifier_input(text, task)),
            "根据ABC-XYZ库存分类/九宫格矩阵意图自动补偿ABC-XYZ库存分类 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "ad fatigue",
            "fatigue",
            "ctr decay",
            "click-through decay",
            "click rate decay",
            "广告疲劳",
            "素材疲劳",
            "创意疲劳",
            "CTR衰减",
            "点击率衰减",
            "频次过高",
            "曝光频次",
            "ROAS健康",
            "roas健康",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_ad_fatigue_detector",
            serde_json::Value::Object(infer_ad_fatigue_input(text, task)),
            "根据广告疲劳/CTR衰减/曝光频次意图自动补偿广告疲劳检测 skill",
        ));
    }

    if contains_any(
        text,
        &["price", "pricing", "cost", "定价", "价格", "售价", "成本"],
    ) {
        let numbers = numbers_in_text(text);
        if text.contains("成本") || text.to_ascii_lowercase().contains("cost") {
            let cost = numbers.first().copied().unwrap_or(1.0).max(0.01);
            return Some(autosuggest_request(
                "ops_pricing_strategy",
                serde_json::Value::Object(infer_pricing_strategy_input(text, task, cost)),
                "根据运营定价意图自动补偿定价策略 skill",
            ));
        }
    }

    if contains_any(text, &["inventory", "stock", "库存", "补货", "备货"]) {
        let numbers = numbers_in_text(text);
        if numbers.len() >= 2 {
            return Some(autosuggest_request(
                "ops_inventory_planning",
                serde_json::Value::Object(infer_inventory_planning_input(
                    text,
                    task,
                    numbers[0],
                    numbers[1].round().max(1.0) as i64,
                )),
                "根据库存/补货意图自动补偿库存规划 skill",
            ));
        }
    }

    if let Some(request) = autosuggest_product_or_campaign_metrics(text, task) {
        return Some(request);
    }

    if contains_any(
        text,
        &[
            "campaign",
            "promotion",
            "promo",
            "活动",
            "促销",
            "大促",
            "五一",
            "618",
            "双11",
        ],
    ) {
        let numbers = numbers_in_text(text);
        if let Some(budget) = infer_promo_budget(text).or_else(|| {
            numbers
                .into_iter()
                .filter(|value| !matches!(*value as i64, 618 | 11))
                .find(|value| *value >= 100.0)
        }) {
            return Some(autosuggest_request(
                "ops_promo_planning",
                serde_json::Value::Object(infer_promo_planning_input(text, task, budget)),
                "根据活动/促销意图自动补偿促销策划 skill",
            ));
        }
    }

    if let Some(request) = autosuggest_query_store_metrics(text, false) {
        return Some(request);
    }

    None
}

fn is_ops_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "ops_pricing_strategy" | "ops_smart_pricing" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "市场定价",
                        "竞品价格区间",
                        "实时价格",
                        "最新价格",
                        "最新竞品价",
                        "当前竞品价",
                        "公开竞品价",
                    ],
                )
        }
        "ops_promo_planning" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "大促规则",
                        "报名规则",
                        "平台活动规则",
                        "活动节点",
                        "营销节点",
                        "热门玩法",
                        "促销玩法",
                        "平台玩法",
                    ],
                )
        }
        "ops_channel_strategy" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "渠道趋势",
                        "流量趋势",
                        "获客成本",
                        "投放趋势",
                        "平台流量",
                        "渠道基准",
                        "ROI基准",
                        "roi基准",
                    ],
                )
        }
        "ops_assortment_planning" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "品类趋势",
                        "选品趋势",
                        "热卖趋势",
                        "爆款趋势",
                        "蓝海品类",
                        "市场机会",
                    ],
                )
        }
        "ops_listing_copy" => {
            is_listing_copy_policy_search_intent(text)
                || is_listing_copy_review_search_intent(text)
                || (has_external_search_intent(text)
                    && contains_any(
                        text,
                        &[
                            "标题规则",
                            "标题规范",
                            "上架规则",
                            "发布规则",
                            "违禁词",
                            "审核规则",
                            "热搜词",
                            "标题趋势",
                            "卖点趋势",
                            "搜索词",
                            "搜索量",
                            "买家痛点",
                            "购买顾虑",
                            "用户反馈",
                            "真实反馈",
                            "高点击",
                            "高转化",
                        ],
                    ))
        }
        _ => false,
    }
}

fn autosuggest_ops_search(text: &str, task: &Task, native_skill_id: &str) -> ParsedSkillRequest {
    match native_skill_id {
        "ops_pricing_strategy" | "ops_smart_pricing" => autosuggest_request(
            "search_competitor",
            serde_json::Value::Object(ops_competitor_price_search_input(text, task)),
            "根据运营定价中的实时竞品价/市场定价需求自动追加外部竞品搜索 skill",
        ),
        "ops_promo_planning" if is_ops_platform_policy_search_intent(text) => autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(ops_promotion_policy_search_input(text, task)),
            "根据运营促销中的平台活动规则/报名规则需求自动追加平台政策搜索 skill",
        ),
        "ops_promo_planning" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(ops_promotion_market_search_input(text, task)),
            "根据运营促销中的最新活动节点/热门玩法需求自动追加外部市场搜索 skill",
        ),
        "ops_channel_strategy" if is_industry_benchmark_search_intent(text) => autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(ops_channel_benchmark_search_input(text, task)),
            "根据运营渠道策略中的行业投放基准需求自动追加行业基准搜索 skill",
        ),
        "ops_channel_strategy" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(ops_channel_market_search_input(text, task)),
            "根据运营渠道策略中的实时渠道/流量趋势需求自动追加外部市场搜索 skill",
        ),
        "ops_assortment_planning" => autosuggest_request(
            "search_trends",
            serde_json::Value::Object(ops_assortment_trend_search_input(text, task)),
            "根据运营选品/货盘中的实时品类趋势需求自动追加外部趋势搜索 skill",
        ),
        "ops_listing_copy" if is_listing_copy_policy_search_intent(text) => autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(listing_copy_policy_search_input(text, task)),
            "根据上架文案中的标题/发布/内容规则诉求自动追加平台规则搜索 skill",
        ),
        "ops_listing_copy" if is_listing_copy_review_search_intent(text) => autosuggest_request(
            "search_product_reviews",
            serde_json::Value::Object(listing_copy_reviews_search_input(text, task)),
            "根据上架文案中的买家评价/痛点诉求自动追加外部评价搜索 skill",
        ),
        "ops_listing_copy" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(listing_copy_market_search_input(text, task)),
            "根据上架文案中的最新标题/卖点/搜索趋势诉求自动追加外部市场搜索 skill",
        ),
        _ => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(ops_market_search_input(text, task)),
            "根据运营实时资料需求自动追加外部市场搜索 skill",
        ),
    }
}

fn is_ops_platform_policy_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "平台规则",
            "平台政策",
            "平台活动规则",
            "大促规则",
            "报名规则",
            "活动规则",
            "活动政策",
            "平台公告",
            "佣金",
            "费率",
        ],
    )
}

fn is_listing_copy_policy_search_intent(text: &str) -> bool {
    (has_external_search_intent(text)
        || contains_any(text, &["最新规则", "平台规则", "平台政策", "规则变化"]))
        && contains_any(
            text,
            &[
                "平台规则",
                "平台政策",
                "标题规则",
                "标题规范",
                "上架规则",
                "发布规则",
                "内容规则",
                "内容规范",
                "违禁词",
                "审核规则",
                "发布审核",
                "违规",
            ],
        )
}

fn is_listing_copy_review_search_intent(text: &str) -> bool {
    has_external_search_intent(text)
        && contains_any(
            text,
            &[
                "用户评价",
                "真实评价",
                "商品评价",
                "买家评价",
                "口碑",
                "差评",
                "好评",
                "吐槽",
                "真实反馈",
                "用户反馈",
                "买家痛点",
                "购买痛点",
                "购买顾虑",
                "消费顾虑",
                "痛点",
                "顾虑",
                "review",
                "reviews",
            ],
        )
}

fn ops_competitor_price_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_competitor_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(ops_search_subject(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("focus".to_string(), serde_json::json!("price"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 14)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_promotion_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("policy_type".to_string(), serde_json::json!("promotion"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_promotion_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 促销活动 热门玩法 活动节点 2026",
            selected_product_platform_or_infer(task, text),
            ops_search_subject(text, task)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("news"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_channel_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 渠道趋势 流量趋势 获客成本 投放ROI 2026",
            selected_product_platform_or_infer(task, text),
            selected_product_category_or_infer(task, text)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_channel_benchmark_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_industry_benchmarks_input(text, task);
    input.insert(
        "industry".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    input.insert("metric".to_string(), serde_json::json!("roi"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_assortment_trend_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 热卖趋势 爆款趋势 选品机会 2026",
            selected_product_platform_or_infer(task, text),
            selected_product_category_or_infer(task, text)
        )),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn listing_copy_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("policy_type".to_string(), serde_json::json!("content"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn listing_copy_reviews_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_product_reviews_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(ops_search_subject(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_listing_copy_review_search_focus(text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn listing_copy_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 上架标题 标题关键词 卖点表达 搜索趋势 高点击 2026",
            selected_product_platform_or_infer(task, text),
            ops_search_subject(text, task)
        )),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn infer_listing_copy_review_search_focus(text: &str) -> &'static str {
    if contains_any(
        text,
        &[
            "差评",
            "负面",
            "投诉",
            "吐槽",
            "不满意",
            "痛点",
            "顾虑",
            "踩雷",
            "negative",
        ],
    ) {
        "negative"
    } else if contains_any(text, &["好评", "正面", "满意", "positive"]) {
        "positive"
    } else {
        "overall"
    }
}

fn ops_market_search_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 电商运营 市场情报 2026",
            selected_product_platform_or_infer(task, text),
            ops_search_subject(text, task)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_search_subject(text: &str, task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| infer_search_query_for_task(text, task))
}

fn infer_ops_search_days(text: &str, default: i64) -> i64 {
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
