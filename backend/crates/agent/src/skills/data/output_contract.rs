fn normalize_data_output(skill_name: &str, input: &Value, output: &mut Value) {
    let Some(map) = output.as_object_mut() else {
        return;
    };
    if map.contains_key("status") {
        return;
    }

    if data_output_needs_user_input(map) {
        let reason = data_deferred_reason(map);
        map.insert("status".to_owned(), json!("deferred"));
        map.insert("skill".to_owned(), json!(skill_name));
        map.insert("has_data".to_owned(), json!(false));
        map.insert("adapter_status".to_owned(), json!("user_input_required"));
        map.insert("data_readiness".to_owned(), json!("missing_user_input"));
        map.insert("deferred_reason".to_owned(), json!(reason));
        map.insert("summary".to_owned(), json!(data_summary(skill_name)));
        return;
    }

    map.insert("status".to_owned(), json!("success"));
    map.insert("has_data".to_owned(), json!(true));
    map.insert("adapter_status".to_owned(), json!("manual_input"));
    map.insert(
        "data_readiness".to_owned(),
        json!(data_readiness(skill_name, input)),
    );
    map.insert("summary".to_owned(), json!(data_summary(skill_name)));
}

fn data_output_needs_user_input(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("error") || matches!(map.get("has_data").and_then(Value::as_bool), Some(false))
}

fn data_deferred_reason(map: &serde_json::Map<String, Value>) -> String {
    ["deferred_reason", "error", "hint", "提示"]
        .iter()
        .find_map(|key| {
            map.get(*key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "需要补充用户输入数据".to_owned())
}

fn data_readiness(skill_name: &str, input: &Value) -> &'static str {
    match skill_name {
        "data_customer_segmentation"
            if !has_any_input(
                input,
                &[
                    "total_customers",
                    "avg_order_value",
                    "avg_orders_per_customer",
                    "recency_days",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "data_competitor_analysis" if input_array_missing_or_empty(input, "competitors") => {
            "calculated_from_defaults"
        }
        "data_ltv_calculator"
            if !has_any_input(
                input,
                &[
                    "cac",
                    "avg_order_value",
                    "purchase_frequency",
                    "gross_margin",
                    "monthly_retention",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "data_cohort_analysis" if !has_any_input(input, &["new_customers"]) => {
            "calculated_from_defaults"
        }
        "data_price_elasticity" if input_array_missing_or_empty(input, "price_points") => {
            "calculated_from_defaults"
        }
        _ => "user_provided",
    }
}

fn data_summary(skill_name: &str) -> &'static str {
    match skill_name {
        "query_store_metrics" => "店铺指标查询结果已生成",
        "query_product_metrics" => "商品指标查询结果已生成",
        "query_campaign_metrics" => "活动指标查询结果已生成",
        "data_anomaly_diagnosis" => "异常诊断报告已生成",
        "data_trend_forecast" => "趋势预测报告已生成",
        "data_customer_segmentation" => "客户分层报告已生成",
        "data_competitor_analysis" => "竞品分析报告已生成",
        "data_multi_period_trend" => "多周期趋势分析报告已生成",
        "data_attribution_analysis" => "渠道归因分析报告已生成",
        "data_ltv_calculator" => "LTV客户价值报告已生成",
        "data_cohort_analysis" => "队列留存分析报告已生成",
        "data_ab_test_analyzer" => "A/B测试分析报告已生成",
        "data_seasonal_decompose" => "季节性趋势分解报告已生成",
        "data_price_elasticity" => "价格弹性分析报告已生成",
        "data_demand_forecast" => "需求预测报告已生成",
        "data_comprehensive_diagnosis" => "综合业务诊断报告已生成",
        _ => "数据分析报告已生成",
    }
}
