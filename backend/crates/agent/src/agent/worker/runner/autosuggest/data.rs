fn autosuggest_data(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "comprehensive diagnosis",
            "comprehensive_diagnosis",
            "综合诊断",
            "全面诊断",
            "业务诊断",
            "经营诊断",
            "经营体检",
            "业务体检",
            "一体化诊断",
            "整体诊断",
            "全链路诊断",
        ],
    ) {
        return Some(autosuggest_request(
            "data_comprehensive_diagnosis",
            serde_json::Value::Object(infer_comprehensive_diagnosis_input(text, task)),
            "根据综合诊断/经营体检意图自动补偿综合业务诊断 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "dashboard",
            "看板",
            "数据概览",
            "经营概览",
            "经营指标",
            "核心指标",
            "店铺大盘",
            "大盘",
        ],
    ) {
        return Some(autosuggest_request(
            "data_dashboard",
            serde_json::Value::Object(infer_dashboard_input(text, task)),
            "根据数据看板/经营概览意图自动补偿数据看板 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "competitor",
            "竞品",
            "对标",
            "竞对",
            "价格带对比",
            "竞品分析",
        ],
    ) {
        let numbers = numbers_in_text(text);
        let request = autosuggest_request(
            "data_competitor_analysis",
            serde_json::json!({
                "product_ref": selected_product_ref(task),
                "our_product": first_selected_product(task)
                    .map(|product| product.name)
                    .unwrap_or_else(|| infer_search_query_for_task(text, task)),
                "our_price": numbers.first().copied().unwrap_or(99.0),
            }),
            "根据竞品/对标意图自动补偿竞品分析 skill",
        );
        return Some(request);
    }

    if contains_any(
        text,
        &["channel roi", "渠道roi", "渠道 roi", "投放渠道", "渠道投产"],
    ) {
        let channels = infer_channel_roi_rows(text);
        return Some(autosuggest_request(
            "data_channel_roi",
            serde_json::json!({
                "days": infer_period_days(text),
                "channels": channels,
                "product_ref": selected_product_ref(task),
            }),
            if channels.is_empty() {
                "根据渠道投产/归因意图自动补偿渠道ROI skill"
            } else {
                "根据用户显式提供的渠道指标自动补偿渠道ROI skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "attribution",
            "归因模型",
            "多触点归因",
            "渠道归因",
            "投放归因",
            "归因分析",
        ],
    ) {
        let channels = infer_channel_roi_rows(text);
        return Some(autosuggest_request(
            "data_attribution_analysis",
            serde_json::json!({
                "days": infer_period_days(text),
                "attribution_model": infer_attribution_model(text),
                "channels": channels,
                "product_ref": selected_product_ref(task),
            }),
            if channels.is_empty() {
                "根据渠道归因/多触点归因意图自动补偿渠道归因 skill"
            } else {
                "根据用户显式提供的渠道指标自动补偿渠道归因 skill"
            },
        ));
    }

    if let Some(request) = autosuggest_search(text, task) {
        return Some(request);
    }

    if contains_any(
        text,
        &[
            "ltv",
            "LTV",
            "客户价值",
            "生命周期价值",
            "获客成本",
            "回收周期",
            "复购价值",
        ],
    ) {
        return Some(autosuggest_request(
            "data_ltv_calculator",
            serde_json::Value::Object(infer_ltv_calculator_input(text, task)),
            "根据LTV/客户价值/获客成本回收意图自动补偿 LTV 客户价值 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "cohort",
            "cohort analysis",
            "retention",
            "留存",
            "留存率",
            "复购率",
            "队列留存",
            "同期群",
            "留存队列",
            "留存分析",
            "复购留存",
            "新客留存",
            "用户留存",
            "客户留存",
            "流失节点",
        ],
    ) {
        return Some(autosuggest_request(
            "data_cohort_analysis",
            serde_json::Value::Object(infer_cohort_analysis_input(text, task)),
            "根据队列留存/同期群/复购留存意图自动补偿队列留存 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "price elasticity",
            "elasticity",
            "价格弹性",
            "价格敏感",
            "需求弹性",
            "调价测试",
            "涨价影响",
            "降价影响",
        ],
    ) {
        return Some(autosuggest_request(
            "data_price_elasticity",
            serde_json::Value::Object(infer_price_elasticity_input(text, task)),
            "根据价格弹性/价格敏感/调价影响意图自动补偿价格弹性 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "a/b",
            "ab test",
            "ab_test",
            "a b test",
            "AB测试",
            "A/B测试",
            "A B测试",
            "对照组",
            "控制组",
            "实验组",
            "测试组",
            "转化对比",
            "实验显著",
            "推全",
        ],
    ) {
        return Some(autosuggest_request(
            "data_ab_test_analyzer",
            serde_json::Value::Object(infer_ab_test_analyzer_input(text, task)),
            "根据A/B测试/对照组/实验组意图自动补偿 A/B 测试分析 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "multi period",
            "multi-period",
            "multi_period",
            "multi period trend",
            "multi-period trend",
            "多周期",
            "多周期趋势",
            "多周期环比",
            "多周期对比",
            "多周期分析",
            "环比趋势",
            "同比环比",
            "移动平均",
            "趋势动能",
        ],
    ) {
        return Some(autosuggest_request(
            "data_multi_period_trend",
            serde_json::Value::Object(infer_multi_period_trend_input(text, task)),
            "根据多周期趋势/环比/移动平均意图自动补偿多周期趋势 skill",
        ));
    }

    if let Some(request) = autosuggest_product_or_campaign_metrics(text, task) {
        return Some(request);
    }

    if let Some(request) = autosuggest_query_store_metrics(text, false) {
        return Some(request);
    }

    if contains_any(
        text,
        &["refund", "退款", "售后占比", "退货率", "帕累托", "退款分解"],
    ) {
        let items = infer_refund_items(text);
        return Some(autosuggest_request(
            "data_refund_decomposition",
            serde_json::json!({
                "days": infer_period_days(text),
                "top_n": 5,
                "product_ref": selected_product_ref(task),
                "items": items,
            }),
            if items.is_empty() {
                "根据退款分解/帕累托意图自动补偿退款分析 skill"
            } else {
                "根据用户显式提供的退款项自动补偿退款分析 skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "seasonal",
            "seasonality",
            "seasonal decompose",
            "季节性",
            "周期性",
            "周期分解",
            "季节分解",
            "周内波动",
            "周期波动",
            "周期趋势",
        ],
    ) {
        return Some(autosuggest_request(
            "data_seasonal_decompose",
            serde_json::Value::Object(infer_seasonal_decompose_input(text, task)),
            "根据季节性/周期性/周内波动意图自动补偿季节性分解 skill",
        ));
    }

    if contains_any(text, &["anomaly", "异常", "波动", "下滑", "暴涨", "暴跌"]) {
        return Some(autosuggest_request(
            "data_anomaly_diagnosis",
            serde_json::Value::Object(infer_anomaly_diagnosis_input(text, task)),
            "根据异常诊断意图自动补偿数据异常 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "demand forecast",
            "demand_forecast",
            "需求预测",
            "销量预测",
            "订单预测",
            "备货预测",
            "补货预测",
            "库存预测",
            "补货量",
            "备货量",
        ],
    ) {
        return Some(autosuggest_request(
            "data_demand_forecast",
            serde_json::Value::Object(infer_demand_forecast_input(text, task)),
            "根据需求预测/备货预测/补货预测意图自动补偿需求预测 skill",
        ));
    }

    if contains_any(text, &["trend", "forecast", "预测", "趋势"]) {
        return Some(autosuggest_request(
            "data_trend_forecast",
            serde_json::Value::Object(infer_trend_forecast_input(text, task)),
            "根据趋势/预测意图自动补偿趋势预测 skill",
        ));
    }

    if contains_any(
        text,
        &["customer", "segment", "rfm", "客户", "人群", "分层"],
    ) {
        return Some(autosuggest_request(
            "data_customer_segmentation",
            serde_json::Value::Object(infer_customer_segmentation_input(text, task)),
            "根据客户分层意图自动补偿 RFM skill",
        ));
    }

    let numbers = numbers_in_text(text);
    let mut input = serde_json::Map::new();
    if numbers.len() >= 5 {
        input.insert(
            "impressions".to_string(),
            serde_json::json!(numbers[0].round() as i64),
        );
        input.insert(
            "clicks".to_string(),
            serde_json::json!(numbers[1].round() as i64),
        );
        input.insert(
            "add_to_cart".to_string(),
            serde_json::json!(numbers[2].round() as i64),
        );
        input.insert(
            "orders".to_string(),
            serde_json::json!(numbers[3].round() as i64),
        );
        input.insert(
            "payments".to_string(),
            serde_json::json!(numbers[4].round() as i64),
        );
    }
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_store_metrics_platform(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    Some(autosuggest_request(
        "data_funnel_analysis",
        serde_json::Value::Object(input),
        "根据数据/转化意图自动补偿漏斗分析 skill",
    ))
}

fn autosuggest_data_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if is_data_price_search_chain_intent(text) {
        let primary = autosuggest_request(
            "data_price_elasticity",
            serde_json::Value::Object(infer_price_elasticity_input(text, task)),
            "根据价格弹性/价格敏感/调价影响意图自动补偿价格弹性 skill",
        );
        return vec![primary, autosuggest_data_competitor_search(text, task)];
    }

    if is_data_competitor_search_chain_intent(text) {
        let numbers = numbers_in_text(text);
        let primary = autosuggest_request(
            "data_competitor_analysis",
            serde_json::json!({
                "product_ref": selected_product_ref(task),
                "our_product": first_selected_product(task)
                    .map(|product| product.name)
                    .unwrap_or_else(|| infer_search_query_for_task(text, task)),
                "our_price": numbers.first().copied().unwrap_or(99.0),
            }),
            "根据竞品/对标意图自动补偿竞品分析 skill",
        );
        return vec![primary, autosuggest_data_competitor_search(text, task)];
    }

    if let Some(primary) = autosuggest_data_native_for_external_search(text, task) {
        if let Some(search) = autosuggest_data_external_search(text, task, &primary.skill_id) {
            return vec![primary, search];
        }
        return vec![primary];
    }

    let Some(primary) = autosuggest_data(text, task) else {
        return Vec::new();
    };

    if let Some(search) = autosuggest_data_external_search(text, task, &primary.skill_id) {
        return vec![primary, search];
    }

    vec![primary]
}

fn autosuggest_data_native_for_external_search(
    text: &str,
    task: &Task,
) -> Option<ParsedSkillRequest> {
    if !has_external_search_intent(text) && !is_industry_benchmark_search_intent(text) {
        return None;
    }

    if contains_any(
        text,
        &[
            "comprehensive diagnosis",
            "comprehensive_diagnosis",
            "综合诊断",
            "全面诊断",
            "业务诊断",
            "经营诊断",
            "经营体检",
            "业务体检",
            "一体化诊断",
            "整体诊断",
            "全链路诊断",
        ],
    ) {
        return Some(autosuggest_request(
            "data_comprehensive_diagnosis",
            serde_json::Value::Object(infer_comprehensive_diagnosis_input(text, task)),
            "根据综合诊断/经营体检意图自动补偿综合业务诊断 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "dashboard",
            "看板",
            "数据概览",
            "经营概览",
            "经营指标",
            "核心指标",
            "店铺大盘",
            "大盘",
        ],
    ) {
        return Some(autosuggest_request(
            "data_dashboard",
            serde_json::Value::Object(infer_dashboard_input(text, task)),
            "根据数据看板/经营概览意图自动补偿数据看板 skill",
        ));
    }

    if contains_any(
        text,
        &["channel roi", "渠道roi", "渠道 roi", "投放渠道", "渠道投产"],
    ) {
        let channels = infer_channel_roi_rows(text);
        return Some(autosuggest_request(
            "data_channel_roi",
            serde_json::json!({
                "days": infer_period_days(text),
                "channels": channels,
                "product_ref": selected_product_ref(task),
            }),
            if channels.is_empty() {
                "根据渠道投产/归因意图自动补偿渠道ROI skill"
            } else {
                "根据用户显式提供的渠道指标自动补偿渠道ROI skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "attribution",
            "归因模型",
            "多触点归因",
            "渠道归因",
            "投放归因",
            "归因分析",
        ],
    ) {
        let channels = infer_channel_roi_rows(text);
        return Some(autosuggest_request(
            "data_attribution_analysis",
            serde_json::json!({
                "days": infer_period_days(text),
                "attribution_model": infer_attribution_model(text),
                "channels": channels,
                "product_ref": selected_product_ref(task),
            }),
            if channels.is_empty() {
                "根据渠道归因/多触点归因意图自动补偿渠道归因 skill"
            } else {
                "根据用户显式提供的渠道指标自动补偿渠道归因 skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "multi period",
            "multi-period",
            "multi_period",
            "multi period trend",
            "multi-period trend",
            "多周期",
            "多周期趋势",
            "多周期环比",
            "多周期对比",
            "多周期分析",
            "环比趋势",
            "同比环比",
            "移动平均",
            "趋势动能",
        ],
    ) {
        return Some(autosuggest_request(
            "data_multi_period_trend",
            serde_json::Value::Object(infer_multi_period_trend_input(text, task)),
            "根据多周期趋势/环比/移动平均意图自动补偿多周期趋势 skill",
        ));
    }

    if contains_any(
        text,
        &["refund", "退款", "售后占比", "退货率", "帕累托", "退款分解"],
    ) {
        let items = infer_refund_items(text);
        return Some(autosuggest_request(
            "data_refund_decomposition",
            serde_json::json!({
                "days": infer_period_days(text),
                "top_n": 5,
                "product_ref": selected_product_ref(task),
                "items": items,
            }),
            if items.is_empty() {
                "根据退款分解/帕累托意图自动补偿退款分析 skill"
            } else {
                "根据用户显式提供的退款项自动补偿退款分析 skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "seasonal",
            "seasonality",
            "seasonal decompose",
            "季节性",
            "周期性",
            "周期分解",
            "季节分解",
            "周内波动",
            "周期波动",
            "周期趋势",
        ],
    ) {
        return Some(autosuggest_request(
            "data_seasonal_decompose",
            serde_json::Value::Object(infer_seasonal_decompose_input(text, task)),
            "根据季节性/周期性/周内波动意图自动补偿季节性分解 skill",
        ));
    }

    if contains_any(text, &["anomaly", "异常", "波动", "下滑", "暴涨", "暴跌"]) {
        return Some(autosuggest_request(
            "data_anomaly_diagnosis",
            serde_json::Value::Object(infer_anomaly_diagnosis_input(text, task)),
            "根据异常诊断意图自动补偿数据异常 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "demand forecast",
            "demand_forecast",
            "需求预测",
            "销量预测",
            "订单预测",
            "备货预测",
            "补货预测",
            "库存预测",
            "补货量",
            "备货量",
        ],
    ) {
        return Some(autosuggest_request(
            "data_demand_forecast",
            serde_json::Value::Object(infer_demand_forecast_input(text, task)),
            "根据需求预测/备货预测/补货预测意图自动补偿需求预测 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "trend forecast",
            "forecast",
            "趋势预测",
            "趋势分析",
            "预测",
            "预估",
        ],
    ) {
        return Some(autosuggest_request(
            "data_trend_forecast",
            serde_json::Value::Object(infer_trend_forecast_input(text, task)),
            "根据趋势/预测意图自动补偿趋势预测 skill",
        ));
    }

    None
}

fn is_data_competitor_search_chain_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "competitor",
            "竞品",
            "竞对",
            "竞争对手",
            "对标",
            "同类商品",
            "价格带对比",
            "竞品分析",
        ],
    ) && has_external_search_intent(text)
}

fn is_data_price_search_chain_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "price elasticity",
            "elasticity",
            "价格弹性",
            "价格敏感",
            "需求弹性",
            "调价测试",
            "涨价影响",
            "降价影响",
            "市场定价",
            "竞品价格区间",
            "价格区间",
        ],
    ) && (has_external_search_intent(text)
        || contains_any(text, &["市场定价", "竞品价格区间", "实时价格", "最新价格"]))
}

fn autosuggest_data_competitor_search(text: &str, task: &Task) -> ParsedSkillRequest {
    let input = search_competitor_input(text, task);
    autosuggest_request(
        "search_competitor",
        serde_json::Value::Object(input),
        "根据数据分析中的实时竞品/市场定价需求自动追加外部搜索 skill",
    )
}

fn autosuggest_data_external_search(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> Option<ParsedSkillRequest> {
    if !is_data_external_search_chain_intent(text, native_skill_id) {
        return None;
    }

    Some(match native_skill_id {
        "data_channel_roi" | "data_attribution_analysis" => autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(data_channel_benchmark_search_input(text, task)),
            "根据数据分析中的行业 ROI / 渠道效率基准需求自动追加行业基准搜索 skill",
        ),
        "data_refund_decomposition" => autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(data_refund_benchmark_search_input(text, task)),
            "根据退款分析中的行业退款率基准需求自动追加行业基准搜索 skill",
        ),
        "data_trend_forecast"
        | "data_demand_forecast"
        | "data_multi_period_trend"
        | "data_seasonal_decompose" => autosuggest_request(
            "search_trends",
            serde_json::Value::Object(data_trend_search_input(text, task, native_skill_id)),
            "根据数据趋势/需求预测中的实时市场趋势需求自动追加趋势搜索 skill",
        ),
        "data_anomaly_diagnosis" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(data_anomaly_market_search_input(text, task)),
            "根据异常诊断中的外部市场原因/公开资料需求自动追加市场搜索 skill",
        ),
        "data_comprehensive_diagnosis" | "data_dashboard" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(data_market_context_search_input(
                text,
                task,
                native_skill_id,
            )),
            "根据经营诊断/看板中的外部市场资料需求自动追加市场搜索 skill",
        ),
        _ => return None,
    })
}

fn is_data_external_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "data_channel_roi" | "data_attribution_analysis" => {
            is_industry_benchmark_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "行业roi",
                        "行业 ROI",
                        "行业ROAS",
                        "行业 roas",
                        "渠道基准",
                        "投放基准",
                        "投产基准",
                        "渠道效率基准",
                        "最新ROI",
                        "最新 ROI",
                        "实时ROI",
                        "实时 ROI",
                    ],
                )
        }
        "data_refund_decomposition" => {
            is_industry_benchmark_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "行业退款率",
                        "平均退款率",
                        "退款率基准",
                        "售后率基准",
                        "退货率基准",
                        "最新退款率",
                        "实时退款率",
                    ],
                )
        }
        "data_trend_forecast"
        | "data_demand_forecast"
        | "data_multi_period_trend"
        | "data_seasonal_decompose" => {
            has_external_search_intent(text)
                && contains_any(
                    text,
                    &[
                        "市场趋势",
                        "消费趋势",
                        "需求趋势",
                        "热卖趋势",
                        "品类趋势",
                        "行业趋势",
                        "季节趋势",
                        "爆款",
                        "热卖",
                        "外部趋势",
                        "趋势参考",
                    ],
                )
        }
        "data_anomaly_diagnosis" => {
            has_external_search_intent(text)
                && contains_any(
                    text,
                    &[
                        "外部原因",
                        "市场原因",
                        "行业原因",
                        "平台原因",
                        "公开资料",
                        "公开来源",
                        "市场动态",
                        "行业动态",
                        "新闻",
                    ],
                )
        }
        "data_comprehensive_diagnosis" | "data_dashboard" => {
            (has_external_search_intent(text) || is_industry_benchmark_search_intent(text))
                && contains_any(
                    text,
                    &[
                        "市场报告",
                        "市场行情",
                        "行业动态",
                        "行业报告",
                        "公开资料",
                        "公开来源",
                        "行业基准",
                        "行业均值",
                        "行业平均",
                        "经营基准",
                    ],
                )
        }
        _ => false,
    }
}

fn data_channel_benchmark_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_industry_benchmarks_input(text, task);
    input.insert("metric".to_string(), serde_json::json!("roi"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_refund_benchmark_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_industry_benchmarks_input(text, task);
    input.insert("metric".to_string(), serde_json::json!("refund"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_trend_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(data_trend_search_query(text, task, native_skill_id)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_data_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_anomaly_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} {} 数据异常 外部原因 行业动态 2026",
            infer_search_platform_for_task(text, task),
            data_search_subject(text, task),
            infer_metric(text)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("news"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_data_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_market_context_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(data_market_context_query(text, task, native_skill_id)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_data_search_days(text, 60)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_trend_search_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = infer_search_platform_for_task(text, task);
    let subject = data_search_subject(text, task);
    match native_skill_id {
        "data_demand_forecast" => {
            format!("{platform} {subject} 需求趋势 热卖趋势 备货预测 2026")
        }
        "data_seasonal_decompose" => {
            format!("{platform} {subject} 季节趋势 周期波动 消费趋势 2026")
        }
        "data_multi_period_trend" => {
            format!("{platform} {subject} 多周期趋势 增长趋势 市场趋势 2026")
        }
        _ => format!("{platform} {subject} 市场趋势 消费趋势 热卖 2026"),
    }
}

fn data_market_context_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = infer_search_platform_for_task(text, task);
    let subject = data_search_subject(text, task);
    match native_skill_id {
        "data_dashboard" => format!("{platform} {subject} 电商经营指标 行业动态 市场报告 2026"),
        _ => format!("{platform} {subject} 电商经营诊断 市场行情 行业报告 2026"),
    }
}

fn data_search_subject(text: &str, task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| infer_search_query_for_task(text, task))
}

fn infer_data_search_days(text: &str, default: i64) -> i64 {
    infer_explicit_period_days(text).unwrap_or(default)
}
