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
