fn autosuggest_search(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if is_platform_policy_search_intent(text) {
        let input = search_platform_policy_input(text, task);
        return Some(autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(input),
            "根据平台规则/政策实时查询意图自动补偿外部搜索 skill",
        ));
    }

    if is_product_review_search_intent(text) {
        let input = search_product_reviews_input(text, task);
        return Some(autosuggest_request(
            "search_product_reviews",
            serde_json::Value::Object(input),
            "根据用户评价/口碑实时查询意图自动补偿外部搜索 skill",
        ));
    }

    if is_industry_benchmark_search_intent(text) {
        let input = search_industry_benchmarks_input(text, task);
        return Some(autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(input),
            "根据行业基准/均值查询意图自动补偿外部搜索 skill",
        ));
    }

    if !has_external_search_intent(text) {
        return None;
    }

    if has_search_competitor_intent(text) {
        let input = search_competitor_input(text, task);
        return Some(autosuggest_request(
            search_competitor_skill_id(),
            serde_json::Value::Object(input),
            "根据竞品外部资料查询意图自动补偿外部搜索 skill",
        ));
    }

    if has_search_trend_intent(text) {
        let input = search_trends_input(text, task);
        return Some(autosuggest_request(
            search_trends_skill_id(),
            serde_json::Value::Object(input),
            "根据趋势/热点实时查询意图自动补偿外部搜索 skill",
        ));
    }

    if has_search_market_info_intent(text) {
        let input = search_market_info_input(text, task);
        return Some(autosuggest_request(
            search_market_info_skill_id(),
            serde_json::Value::Object(input),
            "根据市场行情/公开资料查询意图自动补偿外部搜索 skill",
        ));
    }

    None
}

fn autosuggest_query_store_metrics(
    text: &str,
    explicit_metrics_only: bool,
) -> Option<ParsedSkillRequest> {
    let input = infer_store_metrics_input(text);
    let has_explicit_metrics = store_metrics_input_has_metrics(&input);
    let has_query_intent = has_store_metrics_query_intent(text);
    if explicit_metrics_only && !has_explicit_metrics {
        return None;
    }
    if !has_query_intent {
        return None;
    }

    Some(autosuggest_request(
        "query_store_metrics",
        serde_json::Value::Object(input),
        if has_explicit_metrics {
            "根据用户显式提供的店铺指标自动补偿店铺指标查询 skill"
        } else {
            "根据查询/加载店铺指标意图自动补偿店铺指标查询 skill"
        },
    ))
}

fn autosuggest_product_or_campaign_metrics(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if has_campaign_metrics_query_intent(text) {
        let input = infer_campaign_metrics_input(text);
        let has_explicit_metrics = metric_query_input_has_metrics(&input);
        return Some(autosuggest_request(
            "query_campaign_metrics",
            serde_json::Value::Object(input),
            if has_explicit_metrics {
                "根据用户显式提供的活动指标自动补偿活动指标查询 skill"
            } else {
                "根据查询/加载活动指标意图自动补偿活动指标查询 skill"
            },
        ));
    }

    if has_product_metrics_query_intent(text) {
        let input = infer_product_metrics_input(text, task);
        let has_explicit_metrics = metric_query_input_has_metrics(&input);
        return Some(autosuggest_request(
            "query_product_metrics",
            serde_json::Value::Object(input),
            if has_explicit_metrics {
                "根据用户显式提供的商品指标自动补偿商品指标查询 skill"
            } else {
                "根据查询/加载商品指标意图自动补偿商品指标查询 skill"
            },
        ));
    }

    None
}
