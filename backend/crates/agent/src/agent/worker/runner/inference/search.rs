fn has_store_metrics_query_intent(text: &str) -> bool {
    if contains_any(
        text,
        &[
            "query_store_metrics",
            "store metrics",
            "store_metrics",
            "查询店铺指标",
            "加载店铺指标",
            "读取店铺指标",
            "查询店铺数据",
            "加载店铺数据",
            "读取店铺数据",
            "经营数据基线",
            "数据基线",
        ],
    ) {
        return true;
    }

    contains_any(text, &["店铺指标", "店铺数据", "经营指标", "经营数据"])
        && contains_any(
            text,
            &[
                "查询",
                "加载",
                "读取",
                "拉取",
                "拿一下",
                "看一下",
                "先",
                "作为",
                "基线",
                "load",
                "query",
                "read",
            ],
        )
}

fn has_product_metrics_query_intent(text: &str) -> bool {
    if contains_any(
        text,
        &[
            "query_product_metrics",
            "product metrics",
            "product_metrics",
            "查询商品指标",
            "加载商品指标",
            "读取商品指标",
            "查询商品数据",
            "加载商品数据",
            "读取商品数据",
            "商品表现",
            "单品表现",
            "SKU表现",
            "sku表现",
            "商品数据基线",
        ],
    ) {
        return true;
    }

    contains_any(
        text,
        &[
            "商品指标",
            "商品数据",
            "单品指标",
            "单品数据",
            "SKU指标",
            "sku指标",
        ],
    ) && contains_any(
        text,
        &[
            "查询",
            "加载",
            "读取",
            "拉取",
            "看一下",
            "复盘",
            "分析",
            "基线",
            "load",
            "query",
            "read",
        ],
    )
}

fn has_campaign_metrics_query_intent(text: &str) -> bool {
    if contains_any(
        text,
        &[
            "query_campaign_metrics",
            "campaign metrics",
            "campaign_metrics",
            "查询活动指标",
            "加载活动指标",
            "读取活动指标",
            "查询活动数据",
            "加载活动数据",
            "读取活动数据",
            "活动表现",
            "活动数据基线",
            "投放活动数据",
        ],
    ) {
        return true;
    }

    contains_any(
        text,
        &[
            "活动指标",
            "活动数据",
            "活动复盘",
            "campaign指标",
            "campaign数据",
        ],
    ) && contains_any(
        text,
        &[
            "查询",
            "加载",
            "读取",
            "拉取",
            "看一下",
            "复盘",
            "分析",
            "基线",
            "load",
            "query",
            "read",
        ],
    )
}

fn has_external_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "search",
            "google",
            "搜",
            "搜索",
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
            "fresh",
            "live",
            "source",
        ],
    )
}

fn is_platform_policy_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "平台规则",
            "平台政策",
            "政策变化",
            "最新规则",
            "平台公告",
            "算法调整",
            "流量规则",
            "佣金",
            "费率",
            "违规",
            "违禁词",
            "大促规则",
            "报名规则",
            "policy",
        ],
    ) && (has_external_search_intent(text) || contains_any(text, &["最新", "规则", "政策"]))
}

fn is_product_review_search_intent(text: &str) -> bool {
    contains_any(
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
            "review",
            "reviews",
        ],
    ) && has_external_search_intent(text)
}

fn is_industry_benchmark_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "行业基准",
            "行业均值",
            "行业平均",
            "基准数据",
            "benchmark",
            "benchmarks",
            "平均转化率",
            "平均客单价",
            "行业roi",
            "行业 ROI",
            "行业退款率",
        ],
    ) && (has_external_search_intent(text) || contains_any(text, &["行业", "基准", "均值"]))
}

fn infer_search_platform(text: &str) -> &'static str {
    policy_match_label(text, "platform.search").unwrap_or("电商")
}

fn infer_search_query(text: &str) -> String {
    let category = infer_category(text);
    if category != "电商消费品" {
        return category.to_string();
    }
    let keywords = extract_keyword_candidates(text);
    if let Some(keyword) = keywords.first() {
        return keyword.clone();
    }
    compact_task_text(text)
}

fn infer_search_query_for_task(text: &str, task: &Task) -> String {
    if refers_to_selected_product(text)
        && let Some(product) = first_selected_product(task)
    {
        if let Some(category) = product.category.filter(|value| !value.is_empty()) {
            return format!("{} {}", product.name, category);
        }
        return product.name;
    }
    infer_search_query(text)
}

fn infer_search_industry_for_task(text: &str, task: &Task) -> String {
    if refers_to_selected_product(text)
        && let Some(product) = first_selected_product(task)
        && let Some(category) = product.category.filter(|value| !value.is_empty())
    {
        return category;
    }
    infer_search_industry(text)
}

fn infer_search_industry(text: &str) -> String {
    infer_category(text).to_string()
}

fn infer_search_platform_for_task(text: &str, task: &Task) -> String {
    let explicit_platform = infer_search_platform(text);
    if explicit_platform != "电商" {
        return explicit_platform.to_string();
    }
    first_selected_product(task)
        .and_then(|product| product.platform)
        .filter(|platform| !platform.is_empty())
        .unwrap_or_else(|| explicit_platform.to_string())
}

fn refers_to_selected_product(text: &str) -> bool {
    contains_any(
        text,
        &[
            "这个商品",
            "这个产品",
            "该商品",
            "该产品",
            "本品",
            "当前商品",
            "所选商品",
            "选中商品",
            "这个SKU",
            "该SKU",
            "this product",
            "selected product",
        ],
    )
}

fn insert_selected_product_search_context(
    input: &mut serde_json::Map<String, serde_json::Value>,
    task: &Task,
) {
    if let Some(product) = first_selected_product(task) {
        input.insert(
            "product_ref".to_string(),
            serde_json::json!(product.ref_key()),
        );
        input.insert("product_name".to_string(), serde_json::json!(product.name));
        if let Some(category) = product.category.filter(|value| !value.is_empty()) {
            input.insert("product_category".to_string(), serde_json::json!(category));
        }
    }
}

fn insert_search_max_results(input: &mut serde_json::Map<String, serde_json::Value>, text: &str) {
    if let Some(max_results) = infer_search_max_results(text) {
        input.insert(
            "max_results".to_string(),
            serde_json::json!(max_results.clamp(1, 12)),
        );
    }
}

fn search_product_reviews_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "query".to_string(),
        serde_json::json!(infer_search_query_for_task(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_search_review_focus(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_industry_benchmarks_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "industry".to_string(),
        serde_json::json!(infer_search_industry_for_task(text, task)),
    );
    input.insert(
        "metric".to_string(),
        serde_json::json!(infer_search_benchmark_metric(text)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_competitor_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "query".to_string(),
        serde_json::json!(infer_search_query_for_task(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_search_competitor_focus(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_trends_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "query".to_string(),
        serde_json::json!(infer_search_query_for_task(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_market_info_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "query".to_string(),
        serde_json::json!(infer_search_query_for_task(text, task)),
    );
    input.insert(
        "topic".to_string(),
        serde_json::json!(infer_search_topic(text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_platform_policy_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "policy_type".to_string(),
        serde_json::json!(infer_search_policy_type(text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn infer_search_bypass_cache(text: &str) -> bool {
    contains_any(
        text,
        &[
            "实时",
            "最新",
            "刚刚",
            "今天",
            "联网",
            "公开来源",
            "公开资料",
            "外部资料",
            "真实",
            "真实评价",
            "真实口碑",
            "真实反馈",
            "不要缓存",
            "fresh",
            "live",
            "_needs_fresh_data",
            "bypass_cache",
        ],
    )
}

fn infer_search_max_results(text: &str) -> Option<i64> {
    for (start, ch) in text.char_indices() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if start > 0
            && text[.. start]
                .chars()
                .last()
                .is_some_and(|prev| prev.is_ascii_digit() || prev == '.')
        {
            continue;
        }

        let mut end = start + ch.len_utf8();
        let suffix_start = end;
        for (offset, next) in text[suffix_start ..].char_indices() {
            if next.is_ascii_digit() {
                end = suffix_start + offset + next.len_utf8();
                continue;
            }
            break;
        }

        let Ok(value) = text[start .. end].parse::<i64>() else {
            continue;
        };
        if value <= 0 {
            continue;
        }
        let before = text[.. start]
            .chars()
            .rev()
            .take(16)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        let after = text[end ..].chars().take(16).collect::<String>();

        if is_search_result_count_context(&before, &after) {
            return Some(value.clamp(1, 12));
        }
    }
    None
}

fn is_search_result_count_context(before: &str, after: &str) -> bool {
    let trimmed_after = after.trim_start();
    let has_after_result_unit = [
        "条来源",
        "个来源",
        "条结果",
        "个结果",
        "条公开资料",
        "个公开资料",
        "条证据",
        "个证据",
        "篇资料",
        "篇来源",
        "sources",
        "source",
        "results",
        "result",
        "links",
        "items",
    ]
    .iter()
    .any(|unit| trimmed_after.starts_with(unit));
    let has_before_limit_word = contains_any(
        before,
        &[
            "最多", "至多", "只要", "只看", "返回", "给我", "列出", "展示", "查", "搜", "top",
            "limit", "max", "maximum",
        ],
    );
    let has_after_short_unit = trimmed_after.starts_with('条')
        || trimmed_after.starts_with('个')
        || trimmed_after.starts_with("篇");

    has_after_result_unit || (has_before_limit_word && has_after_short_unit)
}

fn infer_search_policy_type(text: &str) -> &'static str {
    policy_match_label(text, "search.policy_type").unwrap_or("general")
}

fn infer_search_review_focus(text: &str) -> &'static str {
    policy_match_label(text, "search.review_focus").unwrap_or("overall")
}

fn infer_search_competitor_focus(text: &str) -> &'static str {
    policy_match_label(text, "search.competitor_focus").unwrap_or("general")
}

fn infer_search_benchmark_metric(text: &str) -> &'static str {
    policy_match_label(text, "search.benchmark_metric").unwrap_or("general")
}

fn infer_search_topic(text: &str) -> &'static str {
    policy_match_label(text, "search.topic").unwrap_or("general")
}
