fn infer_store_metrics_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_store_metrics_platform(text)),
    );
    input.insert("include_anomalies".to_string(), serde_json::json!(true));

    insert_metric_value(
        &mut input,
        "previous_gmv",
        text,
        &["previous_gmv", "last_gmv", "上期GMV", "上周GMV", "前期GMV"],
    );
    insert_metric_value(
        &mut input,
        "gmv",
        text,
        &[
            "本期GMV",
            "当前GMV",
            "GMV",
            "gmv",
            "销售额",
            "成交额",
            "营收",
        ],
    );
    insert_metric_value(
        &mut input,
        "previous_orders",
        text,
        &[
            "previous_orders",
            "last_orders",
            "上期订单",
            "上周订单",
            "前期订单",
        ],
    );
    insert_metric_value(
        &mut input,
        "orders",
        text,
        &[
            "本期订单",
            "当前订单",
            "订单数",
            "订单",
            "成交订单",
            "支付订单",
        ],
    );
    insert_metric_value(
        &mut input,
        "previous_visitors",
        text,
        &[
            "previous_visitors",
            "last_visitors",
            "上期UV",
            "上周UV",
            "前期UV",
        ],
    );
    insert_metric_value(
        &mut input,
        "visitors",
        text,
        &["本期UV", "当前UV", "UV", "uv", "访客数", "访客"],
    );
    insert_metric_value(
        &mut input,
        "clicks",
        text,
        &["点击数", "点击", "clicks", "click_count"],
    );
    insert_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告费",
            "广告花费",
            "投放费用",
            "ad_spend",
            "ad spend",
            "ad_cost",
        ],
    );
    insert_metric_value(
        &mut input,
        "refund_amount",
        text,
        &["退款金额", "退款额", "退款", "refund_amount", "refunds"],
    );
    insert_metric_value(
        &mut input,
        "gross_profit",
        text,
        &["毛利额", "毛利", "gross_profit", "profit"],
    );

    input
}

fn infer_dashboard_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_store_metrics_input(text);
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    insert_metric_value(
        &mut input,
        "target_gmv",
        text,
        &[
            "目标GMV",
            "目标销售额",
            "目标成交额",
            "target_gmv",
            "goal_gmv",
        ],
    );
    input
}

fn infer_comprehensive_diagnosis_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_dashboard_input(text, task);
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_diagnosis_focus(text)),
    );

    let channels = infer_channel_roi_rows(text);
    if !channels.is_empty() {
        input.insert("channels".to_string(), serde_json::json!(channels));
    }

    let items = infer_refund_items(text);
    if !items.is_empty() {
        input.insert("items".to_string(), serde_json::json!(items));
    }

    let values = numbers_in_text(text);
    if values.len() >= 3 {
        input.insert("values".to_string(), serde_json::json!(values.clone()));
    }
    if values.len() >= 5 {
        input
            .entry("impressions".to_string())
            .or_insert_with(|| serde_json::json!(values[0].round().max(0.0) as i64));
        input
            .entry("clicks".to_string())
            .or_insert_with(|| serde_json::json!(values[1].round().max(0.0) as i64));
        input
            .entry("add_to_cart".to_string())
            .or_insert_with(|| serde_json::json!(values[2].round().max(0.0) as i64));
        input
            .entry("orders".to_string())
            .or_insert_with(|| serde_json::json!(values[3].round().max(0.0) as i64));
        input
            .entry("payments".to_string())
            .or_insert_with(|| serde_json::json!(values[4].round().max(0.0) as i64));
    }

    input
}

fn infer_diagnosis_focus(text: &str) -> &'static str {
    if contains_any(
        text,
        &[
            "重点看增长",
            "重点分析增长",
            "聚焦增长",
            "增长为主",
            "增长优先",
            "focus growth",
        ],
    ) {
        "growth"
    } else if contains_any(text, &["profit", "利润", "毛利", "盈利", "ROI", "roi"]) {
        "profit"
    } else if contains_any(text, &["retention", "留存", "复购", "客户", "会员"]) {
        "retention"
    } else if contains_any(text, &["risk", "风险", "异常", "退款", "下滑"]) {
        "risk"
    } else if contains_any(text, &["growth", "增长", "放量", "规模", "GMV", "gmv"]) {
        "growth"
    } else {
        "all"
    }
}

fn infer_product_metrics_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_common_metrics_input(text);
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if let Some(product) = first_selected_product(task) {
        input.insert("product_name".to_string(), serde_json::json!(product.name));
    } else if !is_category_level_task(&task.description) && !is_category_level_task(text) {
        let product_name = infer_product_name(text);
        if !product_name.trim().is_empty() {
            input.insert("product_name".to_string(), serde_json::json!(product_name));
        }
    }
    if let Some(product_id) = extract_labeled_token(text, &["商品ID", "product_id", "product id"])
    {
        input.insert("product_id".to_string(), serde_json::json!(product_id));
    }
    if let Some(sku_id) = extract_labeled_token(text, &["SKU", "sku_id", "sku id", "规格ID"]) {
        input.insert("sku_id".to_string(), serde_json::json!(sku_id));
    }
    insert_metric_value(
        &mut input,
        "inventory",
        text,
        &["库存", "剩余库存", "inventory", "stock"],
    );
    insert_metric_value(
        &mut input,
        "stockout_count",
        text,
        &["缺货次数", "断货次数", "stockout_count", "stockout"],
    );
    input
}

fn infer_campaign_metrics_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_common_metrics_input(text);
    let campaign_name = infer_campaign_name(text);
    if !campaign_name.trim().is_empty() {
        input.insert(
            "campaign_name".to_string(),
            serde_json::json!(campaign_name),
        );
    }
    if let Some(campaign_id) =
        extract_labeled_token(text, &["活动ID", "campaign_id", "campaign id"])
    {
        input.insert("campaign_id".to_string(), serde_json::json!(campaign_id));
    }
    input.insert(
        "campaign_type".to_string(),
        serde_json::json!(infer_campaign_type(text)),
    );
    insert_metric_value(&mut input, "budget", text, &["预算", "budget", "活动预算"]);
    insert_metric_value(
        &mut input,
        "spend",
        text,
        &["活动花费", "实际花费", "已花费", "消耗", "spend", "cost"],
    );
    insert_metric_value(
        &mut input,
        "impressions",
        text,
        &[
            "曝光",
            "曝光量",
            "展现",
            "展现量",
            "impressions",
            "exposure",
        ],
    );
    insert_metric_value(
        &mut input,
        "coupon_cost",
        text,
        &["优惠券成本", "券成本", "让利成本", "coupon_cost"],
    );
    input
}

fn infer_common_metrics_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_store_metrics_platform(text)),
    );
    insert_metric_value(
        &mut input,
        "previous_gmv",
        text,
        &["previous_gmv", "last_gmv", "上期GMV", "上周GMV", "前期GMV"],
    );
    insert_metric_value(
        &mut input,
        "gmv",
        text,
        &[
            "本期GMV",
            "当前GMV",
            "活动GMV",
            "商品GMV",
            "GMV",
            "gmv",
            "销售额",
            "成交额",
            "营收",
        ],
    );
    insert_metric_value(
        &mut input,
        "orders",
        text,
        &["订单数", "订单", "销量", "成交单", "orders", "order_count"],
    );
    insert_metric_value(
        &mut input,
        "visitors",
        text,
        &["访客数", "访客", "UV", "uv", "visitors"],
    );
    insert_metric_value(
        &mut input,
        "clicks",
        text,
        &["点击数", "点击", "clicks", "click_count"],
    );
    insert_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告费",
            "广告花费",
            "投放费用",
            "投放花费",
            "ad_spend",
            "ad spend",
            "ad_cost",
        ],
    );
    insert_metric_value(
        &mut input,
        "refund_amount",
        text,
        &["退款金额", "退款额", "退款", "refund_amount", "refunds"],
    );
    insert_metric_value(
        &mut input,
        "gross_profit",
        text,
        &["毛利额", "毛利", "gross_profit", "profit"],
    );
    input
}

fn insert_metric_value(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    text: &str,
    aliases: &[&str],
) {
    if let Some(value) = extract_metric_value(text, aliases) {
        input.insert(key.to_string(), serde_json::json!(value));
    }
}

fn infer_trend_forecast_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "metric_name".to_string(),
        serde_json::json!(infer_metric(text)),
    );
    input.insert(
        "forecast_periods".to_string(),
        serde_json::json!(infer_forecast_periods(text)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_anomaly_diagnosis_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "metric_name".to_string(),
        serde_json::json!(infer_metric(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_customer_segmentation_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if let Some(total_customers) = extract_metric_value(
        text,
        &[
            "客户总数",
            "总客户数",
            "客户数",
            "用户数",
            "total_customers",
        ],
    ) {
        input.insert(
            "total_customers".to_string(),
            serde_json::json!(total_customers.round().max(1.0) as i64),
        );
    }
    insert_metric_value(
        &mut input,
        "avg_order_value",
        text,
        &["客单价", "平均客单价", "AOV", "aov", "avg_order_value"],
    );
    insert_metric_value(
        &mut input,
        "avg_orders_per_customer",
        text,
        &[
            "人均订单数",
            "人均购买次数",
            "平均订单数",
            "avg_orders_per_customer",
        ],
    );
    insert_metric_value(
        &mut input,
        "recency_days",
        text,
        &["平均购买间隔", "购买间隔", "最近购买间隔", "recency_days"],
    );
    input
}

fn infer_ab_test_analyzer_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "test_name".to_string(),
        serde_json::json!(infer_ab_test_name(text)),
    );
    input.insert(
        "metric_name".to_string(),
        serde_json::json!(infer_ab_metric(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }

    if let Some(segment) = ab_group_segment(text, AB_CONTROL_LABELS, AB_TEST_LABELS) {
        insert_ab_group_metrics(&mut input, "control", &segment);
    }
    if let Some(segment) = ab_group_segment(text, AB_TEST_LABELS, AB_CONTROL_LABELS) {
        insert_ab_group_metrics(&mut input, "test", &segment);
    }
    insert_ab_direct_metrics(&mut input, text);

    let required_keys = [
        "control_visitors",
        "control_conversions",
        "test_visitors",
        "test_conversions",
    ];
    if required_keys.iter().any(|key| !input.contains_key(*key)) {
        let numbers = numbers_in_text(text);
        if numbers.len() >= 4 {
            for (key, value) in required_keys.iter().zip(numbers.into_iter().take(4)) {
                input
                    .entry((*key).to_string())
                    .or_insert_with(|| serde_json::json!(value.round().max(0.0) as i64));
            }
        }
    }

    input
}

const AB_CONTROL_LABELS: &[&str] = &[
    "对照组",
    "控制组",
    "control group",
    "control",
    "A组",
    "A 组",
    "A版",
    "A 版",
];
const AB_TEST_LABELS: &[&str] = &[
    "实验组",
    "测试组",
    "test group",
    "variant",
    "B组",
    "B 组",
    "B版",
    "B 版",
];

fn ab_group_segment(text: &str, labels: &[&str], other_labels: &[&str]) -> Option<String> {
    let combined = labels
        .iter()
        .chain(other_labels.iter())
        .copied()
        .collect::<Vec<_>>();
    labels
        .iter()
        .find_map(|label| segment_after_label(text, label, &combined))
        .filter(|segment| !segment.trim().is_empty())
}

fn insert_ab_group_metrics(
    input: &mut serde_json::Map<String, serde_json::Value>,
    prefix: &str,
    segment: &str,
) {
    if let Some(value) = extract_metric_value(
        segment,
        &[
            "访客数",
            "访客",
            "UV",
            "uv",
            "样本量",
            "样本",
            "人数",
            "visitors",
            "visitor",
            "users",
            "traffic",
        ],
    ) {
        input.insert(
            format!("{prefix}_visitors"),
            serde_json::json!(value.round().max(0.0) as i64),
        );
    }
    let conversions = extract_metric_value(
        segment,
        &[
            "转化数",
            "转化人数",
            "支付人数",
            "成交人数",
            "成交数",
            "订单数",
            "订单",
            "conversions",
            "conversion_count",
            "orders",
        ],
    )
    .or_else(|| {
        if contains_any(segment, &["转化率", "conversion rate", "cvr"]) {
            None
        } else {
            extract_metric_value(segment, &["转化", "成交", "支付"])
        }
    });
    if let Some(value) = conversions {
        input.insert(
            format!("{prefix}_conversions"),
            serde_json::json!(value.round().max(0.0) as i64),
        );
    }
}

fn insert_ab_direct_metrics(input: &mut serde_json::Map<String, serde_json::Value>, text: &str) {
    insert_ab_integer(
        input,
        "control_visitors",
        text,
        &[
            "control_visitors",
            "control visitors",
            "对照组访客数",
            "对照组访客",
            "控制组访客数",
            "控制组访客",
            "A组访客数",
            "A组访客",
            "A版访客",
        ],
    );
    insert_ab_integer(
        input,
        "control_conversions",
        text,
        &[
            "control_conversions",
            "control conversions",
            "对照组转化数",
            "对照组转化",
            "控制组转化数",
            "控制组转化",
            "A组转化数",
            "A组转化",
            "A版转化",
        ],
    );
    insert_ab_integer(
        input,
        "test_visitors",
        text,
        &[
            "test_visitors",
            "test visitors",
            "实验组访客数",
            "实验组访客",
            "测试组访客数",
            "测试组访客",
            "B组访客数",
            "B组访客",
            "B版访客",
        ],
    );
    insert_ab_integer(
        input,
        "test_conversions",
        text,
        &[
            "test_conversions",
            "test conversions",
            "实验组转化数",
            "实验组转化",
            "测试组转化数",
            "测试组转化",
            "B组转化数",
            "B组转化",
            "B版转化",
        ],
    );
}

fn insert_ab_integer(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    text: &str,
    aliases: &[&str],
) {
    if let Some(value) = extract_metric_value(text, aliases) {
        input.insert(
            key.to_string(),
            serde_json::json!(value.round().max(0.0) as i64),
        );
    }
}

fn infer_ab_test_name(text: &str) -> &'static str {
    if contains_any(text, &["首屏", "详情首屏", "详情页首屏", "above the fold"]) {
        "详情页首屏A/B测试"
    } else if contains_any(text, &["主图", "封面", "image"]) {
        "主图A/B测试"
    } else if contains_any(text, &["标题", "title"]) {
        "标题A/B测试"
    } else if contains_any(text, &["详情页", "详情", "detail"]) {
        "详情页A/B测试"
    } else if contains_any(text, &["直播", "live"]) {
        "直播间A/B测试"
    } else {
        "A/B测试"
    }
}

fn infer_ab_metric(text: &str) -> &'static str {
    if contains_any(text, &["点击", "ctr", "CTR", "click"]) {
        "点击率"
    } else if contains_any(text, &["支付", "成交", "订单", "purchase", "order"]) {
        "支付转化率"
    } else {
        "转化率"
    }
}

fn infer_forecast_periods(text: &str) -> i64 {
    if let Some(days) = extract_metric_value(text, &["未来", "预测", "forecast_periods"]) {
        days.round().clamp(1.0, 90.0) as i64
    } else {
        7
    }
}

fn infer_ltv_calculator_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    insert_metric_value(
        &mut input,
        "cac",
        text,
        &["CAC", "cac", "获客成本", "获客成本CAC"],
    );
    insert_metric_value(
        &mut input,
        "avg_order_value",
        text,
        &["客单价", "AOV", "aov", "avg_order_value"],
    );
    insert_metric_value(
        &mut input,
        "purchase_frequency",
        text,
        &["购买频次", "月购买频次", "复购频次", "purchase_frequency"],
    );
    insert_metric_value(
        &mut input,
        "gross_margin",
        text,
        &["毛利率", "gross_margin"],
    );
    insert_metric_value(
        &mut input,
        "monthly_retention",
        text,
        &["月留存率", "留存率", "monthly_retention"],
    );
    input
}

fn infer_cohort_analysis_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "cohort_unit".to_string(),
        serde_json::json!(infer_cohort_unit(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if let Some(new_customers) = extract_metric_value(
        text,
        &["首期新客", "新客数", "新增客户", "新客", "new_customers"],
    ) {
        input.insert(
            "new_customers".to_string(),
            serde_json::json!(new_customers.round().max(1.0) as i64),
        );
    }
    input
}

fn infer_seasonal_decompose_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert("metric".to_string(), serde_json::json!(infer_metric(text)));
    input.insert(
        "period".to_string(),
        serde_json::json!(infer_seasonal_period(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_multi_period_trend_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert("metric".to_string(), serde_json::json!(infer_metric(text)));
    input.insert(
        "periods".to_string(),
        serde_json::json!(infer_multi_periods(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_multi_periods(text: &str) -> Vec<i64> {
    if contains_any(text, &["周内", "周度", "每周", "weekly", "week"]) {
        vec![7, 14, 28]
    } else {
        vec![7, 14, 30]
    }
}

