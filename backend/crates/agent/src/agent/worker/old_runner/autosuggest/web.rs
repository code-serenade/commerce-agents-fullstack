fn autosuggest_web_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_web_native(text, task) {
        if is_web_search_chain_intent(text, &primary.skill_id) {
            let search = autosuggest_web_search(text, task, &primary.skill_id);
            return vec![primary, search];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_web_title_seo(text, task)]
}

fn autosuggest_web(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_web_requests(text, task).into_iter().next()
}

fn autosuggest_web_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["seo", "SEO", "搜索优化", "排名", "提排"]) {
        return Some(autosuggest_request(
            "web_seo_optimize",
            serde_json::json!({
                "target_keywords": extract_keyword_candidates(text),
                "current_rank": infer_web_current_rank(text),
                "platform": selected_product_platform_or_infer(task, text),
                "product_ref": selected_product_ref(task),
            }),
            "根据 SEO/提排意图自动补偿 SEO 优化 skill",
        ));
    }

    if contains_any(text, &["keyword", "关键词", "搜索词", "长尾词", "蓝海词"]) {
        let product_name = first_selected_product(task).map(|product| product.name);
        let seed_keyword = product_name.unwrap_or_else(|| web_search_subject(text, task));
        return Some(autosuggest_request(
            "web_keyword_research",
            serde_json::json!({
                "seed_keyword": seed_keyword,
                "category": selected_product_category_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
                "product_ref": selected_product_ref(task),
            }),
            "根据关键词/SEO研究意图自动补偿关键词矩阵 skill",
        ));
    }

    if contains_any(text, &["标题", "title", "标题生成", "起标题"]) {
        return Some(autosuggest_request(
            "web_title_generator",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "keywords": extract_keyword_candidates(text),
                "platform": selected_product_platform_or_infer(task, text),
                "style": if contains_any(text, &["品牌"]) { "品牌型" } else if contains_any(text, &["信息"]) { "信息型" } else { "营销型" },
            }),
            "根据标题生成意图自动补偿标题生成 skill",
        ));
    }

    if contains_any(text, &["店铺", "装修", "store"]) {
        return Some(autosuggest_request(
            "web_store_design",
            serde_json::json!({
                "store_type": if contains_any(text, &["旗舰"]) { "旗舰店" } else if contains_any(text, &["专营"]) { "专营店" } else { "电商店铺" },
                "category": selected_product_category_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
                "brand_tone": if contains_any(text, &["高端"]) { "高端奢华" } else if contains_any(text, &["年轻"]) { "年轻活力" } else { "专业可信" },
            }),
            "根据店铺装修意图自动补偿店铺设计 skill",
        ));
    }

    if contains_any(text, &["详情文案", "产品文案", "卖点文案", "description"]) {
        return Some(autosuggest_request(
            "web_product_description",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "platform": selected_product_platform_or_infer(task, text),
                "style": if contains_any(text, &["故事"]) { "故事型" } else if contains_any(text, &["信息"]) { "信息型" } else { "营销型" },
            }),
            "根据详情页文案意图自动补偿产品描述 skill",
        ));
    }

    if contains_any(
        text,
        &["page", "conversion", "页面", "转化", "详情页", "落地页"],
    ) {
        return Some(autosuggest_request(
            "web_page_conversion",
            serde_json::json!({
                "page_type": infer_page_type(text),
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据页面转化意图自动补偿页面转化 skill",
        ));
    }

    None
}

fn autosuggest_web_title_seo(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "web_title_seo_scorer",
        serde_json::json!({
            "title": extract_title_candidate(text).unwrap_or_else(|| compact_task_text(&task.description)),
            "target_keywords": extract_keyword_candidates(text),
            "platform": infer_platform(text),
        }),
        "根据 Web/SEO 岗位任务自动补偿标题 SEO 评分 skill",
    )
}

fn is_web_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    let has_fresh_or_external = has_web_external_search_intent(text);
    match native_skill_id {
        "web_seo_optimize" => {
            has_fresh_or_external
                || contains_any(
                    text,
                    &[
                        "算法",
                        "搜索算法",
                        "平台规则",
                        "平台政策",
                        "算法调整",
                        "流量规则",
                        "排名机制",
                        "关键词竞争",
                        "搜索量",
                        "流量趋势",
                        "权重",
                        "CPC",
                        "cpc",
                    ],
                )
        }
        "web_keyword_research" => {
            has_fresh_or_external
                && contains_any(
                    text,
                    &[
                        "关键词竞争",
                        "搜索量",
                        "CPC",
                        "cpc",
                        "热度",
                        "流量",
                        "竞争",
                        "出价",
                    ],
                )
        }
        "web_page_conversion" => {
            is_industry_benchmark_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "转化率基准",
                        "行业转化率",
                        "行业均值",
                        "行业平均",
                        "平均转化率",
                        "实时基准",
                        "最新基准",
                        "benchmark",
                        "CVR",
                        "cvr",
                    ],
                )
        }
        "web_title_generator" => {
            is_listing_copy_policy_search_intent(text)
                || (has_fresh_or_external
                    && contains_any(
                        text,
                        &[
                            "标题规则",
                            "标题规范",
                            "平台规则",
                            "流量规则",
                            "违禁词",
                            "审核规则",
                            "关键词竞争",
                            "搜索量",
                            "CPC",
                            "cpc",
                            "热搜词",
                            "标题趋势",
                            "高点击",
                            "点击率",
                            "CTR",
                            "ctr",
                        ],
                    ))
        }
        "web_product_description" => {
            is_listing_copy_policy_search_intent(text)
                || is_listing_copy_review_search_intent(text)
                || (has_fresh_or_external
                    && contains_any(
                        text,
                        &[
                            "详情页趋势",
                            "详情文案趋势",
                            "卖点趋势",
                            "高转化",
                            "转化卖点",
                            "买家痛点",
                            "购买顾虑",
                            "用户反馈",
                            "真实反馈",
                            "用户评价",
                            "真实评价",
                            "商品评价",
                            "买家评价",
                            "口碑",
                        ],
                    ))
        }
        _ => false,
    }
}

fn has_web_external_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "google",
            "搜一下",
            "搜搜",
            "查一下",
            "查查",
            "联网",
            "实时",
            "最新",
            "公开来源",
            "公开资料",
            "外部资料",
            "网上",
            "资料来源",
            "来源",
            "不要缓存",
            "当前",
            "近期",
            "今年",
            "今日",
            "今天",
            "站外",
            "fresh",
            "live",
            "source",
        ],
    )
}

fn autosuggest_web_search(text: &str, task: &Task, native_skill_id: &str) -> ParsedSkillRequest {
    match native_skill_id {
        "web_page_conversion" => autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(web_conversion_benchmark_search_input(text, task)),
            "根据页面转化分析中的行业转化率基准需求自动追加外部搜索 skill",
        ),
        "web_seo_optimize" if is_web_platform_policy_search_intent(text) => autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(web_platform_policy_search_input(text, task)),
            "根据 SEO 优化中的平台搜索算法/流量规则需求自动追加平台政策搜索 skill",
        ),
        "web_seo_optimize" | "web_keyword_research" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(web_seo_market_search_input(text, task, native_skill_id)),
            "根据 Web 增长中的关键词竞争/搜索量/SEO实时资料需求自动追加外部市场搜索 skill",
        ),
        "web_title_generator" if is_listing_copy_policy_search_intent(text) => autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(listing_copy_policy_search_input(text, task)),
            "根据标题生成中的平台标题/内容规则诉求自动追加平台规则搜索 skill",
        ),
        "web_title_generator" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(web_listing_copy_market_search_input(
                text,
                task,
                native_skill_id,
            )),
            "根据标题生成中的最新关键词/搜索趋势/高点击诉求自动追加外部市场搜索 skill",
        ),
        "web_product_description" if is_listing_copy_review_search_intent(text) => {
            autosuggest_request(
                "search_product_reviews",
                serde_json::Value::Object(listing_copy_reviews_search_input(text, task)),
                "根据详情文案中的买家评价/痛点诉求自动追加外部评价搜索 skill",
            )
        }
        "web_product_description" if is_listing_copy_policy_search_intent(text) => {
            autosuggest_request(
                "search_platform_policy",
                serde_json::Value::Object(listing_copy_policy_search_input(text, task)),
                "根据详情文案中的平台内容/发布规则诉求自动追加平台规则搜索 skill",
            )
        }
        "web_product_description" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(web_listing_copy_market_search_input(
                text,
                task,
                native_skill_id,
            )),
            "根据详情文案中的最新详情页/卖点/转化趋势诉求自动追加外部市场搜索 skill",
        ),
        _ => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(web_seo_market_search_input(text, task, native_skill_id)),
            "根据 Web 增长实时资料需求自动追加外部市场搜索 skill",
        ),
    }
}

fn is_web_platform_policy_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "算法",
            "搜索算法",
            "平台规则",
            "平台政策",
            "算法调整",
            "流量规则",
            "排名机制",
            "权重",
            "规则",
            "policy",
            "algorithm",
        ],
    )
}

fn web_seo_market_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(web_seo_market_query(text, task, native_skill_id)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_web_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn web_platform_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("policy_type".to_string(), serde_json::json!("algorithm"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_web_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn web_conversion_benchmark_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_industry_benchmarks_input(text, task);
    input.insert(
        "industry".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    input.insert("metric".to_string(), serde_json::json!("conversion"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn web_listing_copy_market_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(web_listing_copy_market_query(text, task, native_skill_id)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_web_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn web_listing_copy_market_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = web_search_subject(text, task);
    match native_skill_id {
        "web_title_generator" => {
            format!("{platform} {subject} 商品标题 关键词 搜索趋势 高点击 CTR 2026")
        }
        "web_product_description" => {
            format!("{platform} {subject} 详情页文案 卖点表达 转化趋势 买家痛点 2026")
        }
        _ => format!("{platform} {subject} Web增长 文案趋势 2026"),
    }
}

fn web_seo_market_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = web_search_subject(text, task);
    match native_skill_id {
        "web_keyword_research" => {
            format!("{platform} {subject} 关键词竞争 搜索量 CPC 流量热度 2026")
        }
        _ => format!("{platform} SEO 搜索优化 {subject} 关键词竞争 流量 2026"),
    }
}

fn web_search_subject(text: &str, task: &Task) -> String {
    if let Some(product) = first_selected_product(task) {
        return product.name;
    }
    extract_keyword_candidates(text)
        .into_iter()
        .find(|keyword| !matches!(keyword.as_str(), "SEO" | "关键词" | "标题" | "转化"))
        .unwrap_or_else(|| infer_category(text).to_owned())
}

fn infer_web_search_days(text: &str, default: i64) -> i64 {
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

fn infer_web_current_rank(text: &str) -> Option<i64> {
    if !contains_any(
        text,
        &[
            "当前排名",
            "现在排名",
            "排名第",
            "第",
            "名",
            "排第",
            "提排到",
            "rank",
            "ranking",
        ],
    ) {
        return None;
    }
    numbers_in_text(text)
        .first()
        .copied()
        .map(|value| value.round().max(1.0) as i64)
}
