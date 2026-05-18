fn autosuggest_accounting_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_accounting_native(text) {
        if is_accounting_finance_search_chain_intent(text, &primary.skill_id) {
            return vec![primary, autosuggest_accounting_finance_search(text, task)];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_request(
        "accounting_budget_plan",
        serde_json::json!({}),
        "根据财务岗位任务自动补偿预算编制 skill",
    )]
}

fn autosuggest_accounting(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_accounting_requests(text, task)
        .into_iter()
        .next()
}

fn autosuggest_accounting_native(text: &str) -> Option<ParsedSkillRequest> {
    let numbers = numbers_in_text(text);

    if is_accounting_finance_market_search_text(text) && !has_accounting_native_work_intent(text) {
        return None;
    }

    if contains_any(
        text,
        &[
            "财务健康",
            "财务诊断",
            "财务体检",
            "健康评级",
            "财务健康诊断",
            "financial narrative",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_financial_narrative",
            serde_json::Value::Object(infer_accounting_financial_input(text)),
            "根据财务健康/诊断意图自动补偿财务健康诊断 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "p&l",
            "P&L",
            "利润表",
            "损益表",
            "完整利润表",
            "pl statement",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_pl_statement",
            serde_json::Value::Object(infer_accounting_pl_input(text)),
            "根据 P&L/利润表意图自动补偿完整利润表 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "GMV瀑布",
            "gmv瀑布",
            "利润瀑布",
            "利润漏损",
            "漏损项",
            "waterfall",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_gmv_waterfall",
            serde_json::Value::Object(infer_accounting_gmv_waterfall_input(text)),
            "根据 GMV/利润瀑布意图自动补偿利润瀑布 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "预算vs实际",
            "预算VS实际",
            "预算与实际",
            "预算执行",
            "预算偏差",
            "超支",
            "budget vs actual",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_budget_vs_actual",
            serde_json::Value::Object(infer_accounting_budget_vs_actual_input(text)),
            "根据预算执行/预算偏差意图自动补偿预算vs实际 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "compliance",
            "合规",
            "税务",
            "纳税",
            "开票",
            "发票",
            "现金收款",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_compliance_check",
            serde_json::json!({
                "monthly_revenue": numbers.first().copied().unwrap_or(500_000.0),
                "tax_type": if contains_any(text, &["一般纳税人"]) { "一般纳税人" } else { "小规模" },
                "invoice_rate": infer_percent_number(text, 80.0),
                "cash_ratio": infer_second_percent_number(text, 10.0),
            }),
            "根据财税合规意图自动补偿合规检查 skill",
        ));
    }

    if contains_any(text, &["roi", "roas", "投产", "投入产出", "回报"]) {
        return Some(autosuggest_request(
            "accounting_roi_calc",
            serde_json::json!({
                "investment": numbers.first().copied(),
                "revenue_generated": numbers.get(1).copied(),
            }),
            "根据 ROI/投产意图自动补偿 ROI 计算 skill",
        ));
    }

    if contains_any(text, &["cost", "margin", "成本", "毛利"]) && numbers.len() >= 2 {
        return Some(autosuggest_request(
            "accounting_cost_calc",
            serde_json::json!({
                "purchase_cost": numbers[0],
                "selling_price": numbers[1],
            }),
            "根据成本/毛利意图自动补偿成本核算 skill",
        ));
    }

    if contains_any(text, &["profit", "利润", "盈利", "亏损"]) {
        return Some(autosuggest_request(
            "accounting_profit_analysis",
            serde_json::json!({
                "revenue": numbers.first().copied().unwrap_or(0.0),
            }),
            "根据利润分析意图自动补偿利润分析 skill",
        ));
    }

    if let Some(request) = autosuggest_query_store_metrics(text, false) {
        return Some(request);
    }

    None
}

fn infer_accounting_financial_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    insert_optional_metric_value(
        &mut input,
        "gmv",
        text,
        &["GMV", "gmv", "销售额", "成交额", "营收", "收入"],
    );
    insert_optional_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告费",
            "广告花费",
            "广告投放",
            "投放费",
            "ad_spend",
            "ad spend",
        ],
    );
    insert_optional_metric_value(
        &mut input,
        "orders",
        text,
        &["订单量", "订单数", "订单", "orders"],
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_accounting_analysis_days(text, 30)),
    );
    input.insert(
        "context".to_string(),
        serde_json::json!(compact_task_text(text)),
    );
    input
}

fn infer_accounting_pl_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    insert_optional_metric_value(
        &mut input,
        "gmv",
        text,
        &["GMV", "gmv", "销售额", "成交额", "营收", "收入"],
    );
    insert_optional_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告费",
            "广告花费",
            "广告投放",
            "投放费",
            "ad_spend",
            "ad spend",
        ],
    );
    insert_optional_metric_value(
        &mut input,
        "orders",
        text,
        &["订单量", "订单数", "订单", "orders"],
    );
    insert_optional_metric_value(
        &mut input,
        "cogs_rate",
        text,
        &["商品成本率", "成本率", "cogs_rate"],
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_accounting_analysis_days(text, 30)),
    );
    input
}

fn infer_accounting_gmv_waterfall_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_accounting_pl_input(text);
    insert_optional_metric_value(
        &mut input,
        "refund_rate",
        text,
        &["退款率", "退货率", "refund_rate"],
    );
    insert_optional_metric_value(
        &mut input,
        "logistics_rate",
        text,
        &["物流费率", "物流成本率", "logistics_rate"],
    );
    insert_optional_metric_value(
        &mut input,
        "platform_fee_rate",
        text,
        &["平台佣金率", "平台费率", "佣金率", "platform_fee_rate"],
    );
    input
}

fn infer_accounting_budget_vs_actual_input(
    text: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    insert_optional_metric_value(
        &mut input,
        "budget_gmv",
        text,
        &["预算GMV", "预算gmv", "目标GMV", "目标gmv", "预算销售额"],
    );
    insert_optional_metric_value(
        &mut input,
        "actual_gmv",
        text,
        &["实际GMV", "实际gmv", "实际销售额", "实际营收"],
    );
    insert_optional_metric_value(
        &mut input,
        "budget_ad_spend",
        text,
        &["预算广告费", "预算投放费", "预算广告投入"],
    );
    insert_optional_metric_value(
        &mut input,
        "actual_ad_spend",
        text,
        &["实际广告费", "实际投放费", "实际广告投入"],
    );
    input
}

fn insert_optional_metric_value(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    text: &str,
    aliases: &[&str],
) {
    if let Some(value) = extract_metric_value(text, aliases) {
        input.insert(key.to_string(), serde_json::json!(value));
    }
}

fn is_accounting_finance_market_search_text(text: &str) -> bool {
    contains_any(
        text,
        &[
            "行业财务基准",
            "财务基准",
            "财务对标",
            "财务benchmark",
            "财务 benchmark",
            "利润率基准",
            "行业利润率",
            "毛利率基准",
            "净利率基准",
            "广告ROI基准",
            "广告 ROI 基准",
        ],
    ) || (has_external_search_intent(text)
        && contains_any(
            text,
            &[
                "财务",
                "利润率",
                "毛利率",
                "净利率",
                "财报",
                "现金流",
                "广告ROI",
            ],
        ))
}

fn has_accounting_native_work_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "做",
            "分析",
            "诊断",
            "体检",
            "计算",
            "核算",
            "生成",
            "编制",
            "预测",
            "检查",
            "评估",
            "帮我看",
            "出一份",
            "利润表",
            "损益表",
            "GMV瀑布",
            "gmv瀑布",
            "利润瀑布",
            "预算执行",
            "预算偏差",
            "财务健康",
            "财务诊断",
        ],
    )
}

fn is_accounting_finance_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    matches!(
        native_skill_id,
        "accounting_financial_narrative"
            | "accounting_pl_statement"
            | "accounting_gmv_waterfall"
            | "accounting_budget_vs_actual"
    ) && (has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "行业财务基准",
                "财务基准",
                "财务对标",
                "财务benchmark",
                "财务 benchmark",
                "利润率基准",
                "行业利润率",
                "毛利率基准",
                "净利率基准",
                "广告ROI基准",
                "广告 ROI 基准",
                "财务公开来源",
            ],
        ))
}

fn autosuggest_accounting_finance_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_market_info",
        serde_json::Value::Object(accounting_finance_search_input(text, task)),
        "根据财务诊断中的行业财务基准/财务对标需求自动追加外部财务搜索 skill",
    )
}

fn accounting_finance_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(accounting_finance_search_query(text, task)),
    );
    input.insert("topic".to_string(), serde_json::json!("finance"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_accounting_search_days(text, 60)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn accounting_finance_search_query(text: &str, task: &Task) -> String {
    if let Some(product) = first_selected_product(task) {
        let category = product
            .category
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "电商消费品".to_string());
        return format!(
            "{} {} 电商行业 毛利率 净利率 广告ROI 财务基准 2026",
            product.name, category
        );
    }

    let category = infer_category(text);
    if category != "电商消费品" {
        format!("{category} 电商行业 毛利率 净利率 广告ROI 财务基准 2026")
    } else {
        "电商行业 毛利率 净利率 广告ROI 财务基准 2026".to_string()
    }
}

fn infer_accounting_analysis_days(text: &str, default: i64) -> i64 {
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

fn infer_accounting_search_days(text: &str, default: i64) -> i64 {
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
