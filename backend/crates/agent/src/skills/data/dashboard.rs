fn data_dashboard(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let days = i(input, "days", 30).clamp(1, 365);
    let platform = s(input, "platform", "all");
    let (gmv, has_gmv) = number_from_any(input, &["gmv", "revenue"], 128_000.0);
    let (previous_gmv, has_previous_gmv) =
        number_from_any(input, &["previous_gmv", "last_gmv"], gmv * 0.92);
    let (orders, has_orders) = number_from_any(input, &["orders", "order_count"], 1_280.0);
    let (previous_orders, has_previous_orders) =
        number_from_any(input, &["previous_orders", "last_orders"], orders * 0.95);
    let (visitors, has_visitors) = number_from_any(input, &["visitors", "uv"], 32_000.0);
    let (previous_visitors, has_previous_visitors) = number_from_any(
        input,
        &["previous_visitors", "last_visitors"],
        visitors * 0.96,
    );
    let (clicks, has_clicks) = number_from_any(input, &["clicks", "click_count"], visitors * 0.18);
    let (ad_spend, has_ad_spend) = number_from_any(input, &["ad_spend", "ad_cost"], gmv * 0.12);
    let (refund_amount, has_refund_amount) =
        number_from_any(input, &["refund_amount", "refunds"], gmv * 0.035);
    let (gross_profit, has_gross_profit) =
        number_from_any(input, &["gross_profit", "profit"], gmv * 0.28);
    let (target_gmv, has_target_gmv) =
        number_from_any(input, &["target_gmv", "goal_gmv"], gmv * 1.15);

    let has_user_metrics = [
        has_gmv,
        has_previous_gmv,
        has_orders,
        has_previous_orders,
        has_visitors,
        has_previous_visitors,
        has_clicks,
        has_ad_spend,
        has_refund_amount,
        has_gross_profit,
        has_target_gmv,
    ]
    .into_iter()
    .any(|item| item);

    let conversion_rate = pct(orders, visitors.max(1.0));
    let ctr = pct(clicks, visitors.max(1.0));
    let aov = gmv / orders.max(1.0);
    let ad_roi = gmv / ad_spend.max(0.01);
    let refund_rate = pct(refund_amount, gmv.max(1.0));
    let gross_margin = pct(gross_profit, gmv.max(1.0));
    let target_achievement = pct(gmv, target_gmv.max(1.0));
    let gmv_growth = pct_delta(gmv, previous_gmv);
    let order_growth = pct_delta(orders, previous_orders);
    let visitor_growth = pct_delta(visitors, previous_visitors);

    let mut alerts = Vec::new();
    if gmv_growth < -10.0 {
        alerts.push(json!({"level": "warning", "metric": "GMV", "message": "GMV环比下滑超过10%，优先排查流量、转化和客单价"}));
    }
    if conversion_rate < 2.0 {
        alerts.push(json!({"level": "warning", "metric": "转化率", "message": "访客到订单转化偏低，建议复核详情页、价格和首屏卖点"}));
    }
    if ad_roi < 2.0 {
        alerts.push(json!({"level": "warning", "metric": "广告ROI", "message": "广告ROI低于2，建议收缩低效计划并复盘素材"}));
    }
    if refund_rate > 6.0 {
        alerts.push(json!({"level": "warning", "metric": "退款率", "message": "退款率偏高，建议拆解售后原因和商品描述一致性"}));
    }
    if gross_margin < 20.0 {
        alerts.push(json!({"level": "warning", "metric": "毛利率", "message": "毛利率偏低，需复核进货成本、优惠力度和平台扣点"}));
    }
    if alerts.is_empty() {
        alerts.push(json!({"level": "info", "metric": "整体", "message": "核心指标未触发高优先级预警，建议继续跟踪趋势和渠道结构"}));
    }

    let score = dashboard_health_score(
        gmv_growth,
        conversion_rate,
        ad_roi,
        refund_rate,
        gross_margin,
        target_achievement,
    );
    let data_readiness = if has_user_metrics {
        "user_provided"
    } else {
        "sample_only"
    };

    json!({
        "status": "success",
        "has_data": has_user_metrics,
        "data_readiness": data_readiness,
        "adapter_status": if has_user_metrics { "manual_input" } else { "store_metrics_required" },
        "数据来源": if has_user_metrics { "用户提供指标" } else { "未接入 store_metrics adapter；以下为样例基线，仅用于展示看板结构和诊断口径" },
        "商品引用": empty_string_as_null(product_ref),
        "周期": format!("{days}天"),
        "平台": platform,
        "核心指标": [
            metric_card("GMV", round2(gmv), "CNY", gmv_growth, "看收入规模和环比动能"),
            metric_card("订单数", round2(orders), "orders", order_growth, "看成交规模"),
            metric_card("访客数", round2(visitors), "UV", visitor_growth, "看流量盘子"),
            metric_card("点击率", round2(ctr), "%", ctr - 15.0, "流量点击效率"),
            metric_card("转化率", round2(conversion_rate), "%", conversion_rate - 3.0, "访客到订单效率"),
            metric_card("客单价", round2(aov), "CNY", 0.0, "成交结构和价格带"),
            metric_card("广告ROI", round2(ad_roi), "x", ad_roi - 2.5, "投放产出效率"),
            metric_card("退款率", round2(refund_rate), "%", 4.0 - refund_rate, "售后和描述一致性"),
            metric_card("毛利率", round2(gross_margin), "%", gross_margin - 25.0, "利润安全垫"),
            metric_card("目标完成率", round2(target_achievement), "%", target_achievement - 100.0, "目标进度")
        ],
        "经营健康度": {
            "总分": score,
            "等级": if score >= 80 { "健康" } else if score >= 60 { "需关注" } else { "风险" },
            "评分口径": "GMV动能、转化、ROI、退款、毛利和目标完成率的确定性规则评分"
        },
        "看板分区": [
            {"section": "增长", "metrics": ["GMV", "订单数", "访客数"], "summary": growth_summary(gmv_growth, order_growth, visitor_growth)},
            {"section": "效率", "metrics": ["转化率", "客单价", "广告ROI"], "summary": efficiency_summary(conversion_rate, ad_roi, aov)},
            {"section": "风险", "metrics": ["退款率", "毛利率"], "summary": risk_summary(refund_rate, gross_margin)}
        ],
        "预警": alerts,
        "下一步动作": dashboard_actions(gmv_growth, conversion_rate, ad_roi, refund_rate, gross_margin),
        "说明": if has_user_metrics { "本看板基于本次 skill 入参计算，不代表外部平台实时拉取结果。" } else { "当前没有真实店铺指标入参，也没有 store_metrics adapter；前端应展示为缺数据/样例态，不能当作平台实时数据。" }
    })
}

fn store_metric_platform_rows(input: &Value, platform_filter: &str) -> Vec<Value> {
    input
        .get("platforms")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    platform_filter == "all"
                        || platform_filter.trim().is_empty()
                        || value_string(item, "platform", "")
                            .eq_ignore_ascii_case(platform_filter.trim())
                })
                .filter_map(|item| {
                    let platform = value_string(item, "platform", "unknown");
                    let (gmv, has_gmv) = number_from_any(item, &["gmv", "revenue"], 0.0);
                    let (orders, has_orders) =
                        number_from_any(item, &["orders", "order_count"], 0.0);
                    let (uv, has_uv) = number_from_any(item, &["visitors", "uv"], 0.0);
                    let (clicks, has_clicks) =
                        number_from_any(item, &["clicks", "click_count"], 0.0);
                    let (ad_spend, has_ad_spend) =
                        number_from_any(item, &["ad_spend", "ad_cost"], 0.0);
                    let (refund_amount, has_refund) =
                        number_from_any(item, &["refund_amount", "refunds"], 0.0);
                    let (gross_profit, has_profit) =
                        number_from_any(item, &["gross_profit", "profit"], 0.0);

                    if [
                        has_gmv,
                        has_orders,
                        has_uv,
                        has_clicks,
                        has_ad_spend,
                        has_refund,
                        has_profit,
                    ]
                    .into_iter()
                    .any(|value| value)
                    {
                        Some(json!({
                            "platform": platform,
                            "gmv": round2(gmv),
                            "orders": round2(orders),
                            "uv": round2(uv),
                            "clicks": round2(clicks),
                            "ad_spend": round2(ad_spend),
                            "refund_amount": round2(refund_amount),
                            "gross_profit": round2(gross_profit),
                            "conversion_rate": percent(pct(orders, uv.max(1.0)), 2)
                        }))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn store_metric_totals(input: &Value, platform_rows: &[Value]) -> Value {
    if !platform_rows.is_empty() {
        let sum = |key: &str| {
            platform_rows
                .iter()
                .filter_map(|row| value_pointer_number(row, &format!("/{key}")))
                .sum::<f64>()
        };
        return json!({
            "gmv": round2(sum("gmv")),
            "orders": round2(sum("orders")),
            "uv": round2(sum("uv")),
            "clicks": round2(sum("clicks")),
            "ad_spend": round2(sum("ad_spend")),
            "refund_amount": round2(sum("refund_amount")),
            "gross_profit": round2(sum("gross_profit"))
        });
    }

    let (gmv, _) = number_from_any(input, &["gmv", "revenue"], 0.0);
    let (orders, _) = number_from_any(input, &["orders", "order_count"], 0.0);
    let (uv, _) = number_from_any(input, &["visitors", "uv"], 0.0);
    let (clicks, _) = number_from_any(input, &["clicks", "click_count"], 0.0);
    let (ad_spend, _) = number_from_any(input, &["ad_spend", "ad_cost"], 0.0);
    let (refund_amount, _) = number_from_any(input, &["refund_amount", "refunds"], 0.0);
    let (gross_profit, _) = number_from_any(input, &["gross_profit", "profit"], 0.0);

    json!({
        "gmv": round2(gmv),
        "orders": round2(orders),
        "uv": round2(uv),
        "clicks": round2(clicks),
        "ad_spend": round2(ad_spend),
        "refund_amount": round2(refund_amount),
        "gross_profit": round2(gross_profit)
    })
}

fn store_metric_previous(input: &Value) -> Value {
    let (previous_gmv, has_previous_gmv) =
        number_from_any(input, &["previous_gmv", "last_gmv"], 0.0);
    let (previous_orders, has_previous_orders) =
        number_from_any(input, &["previous_orders", "last_orders"], 0.0);
    let (previous_visitors, has_previous_visitors) =
        number_from_any(input, &["previous_visitors", "last_visitors"], 0.0);

    json!({
        "gmv": if has_previous_gmv { json!(round2(previous_gmv)) } else { Value::Null },
        "orders": if has_previous_orders { json!(round2(previous_orders)) } else { Value::Null },
        "uv": if has_previous_visitors { json!(round2(previous_visitors)) } else { Value::Null }
    })
}

fn store_metric_week_over_week(gmv: f64, orders: f64, visitors: f64, previous: &Value) -> Value {
    let mut entries = Map::new();
    if let Some(previous_gmv) = value_pointer_number(previous, "/gmv").filter(|value| *value > 0.0)
    {
        entries.insert(
            "gmv".to_owned(),
            json!({
                "current": round2(gmv),
                "previous": round2(previous_gmv),
                "change_rate": signed_percent(pct_delta(gmv, previous_gmv), 2)
            }),
        );
    }
    if let Some(previous_orders) =
        value_pointer_number(previous, "/orders").filter(|value| *value > 0.0)
    {
        entries.insert(
            "orders".to_owned(),
            json!({
                "current": round2(orders),
                "previous": round2(previous_orders),
                "change_rate": signed_percent(pct_delta(orders, previous_orders), 2)
            }),
        );
    }
    if let Some(previous_visitors) =
        value_pointer_number(previous, "/uv").filter(|value| *value > 0.0)
    {
        entries.insert(
            "uv".to_owned(),
            json!({
                "current": round2(visitors),
                "previous": round2(previous_visitors),
                "change_rate": signed_percent(pct_delta(visitors, previous_visitors), 2)
            }),
        );
    }
    Value::Object(entries)
}

fn store_metric_anomalies(
    gmv: f64,
    conversion_rate: f64,
    ad_roi: Option<f64>,
    refund_rate: f64,
    gross_margin: f64,
) -> Vec<Value> {
    let mut anomalies = Vec::new();
    if gmv <= 0.0 {
        anomalies.push(json!({
            "metric": "gmv",
            "level": "warning",
            "message": "GMV为0或未提供，后续收入分析可信度不足"
        }));
    }
    if conversion_rate > 0.0 && conversion_rate < 1.0 {
        anomalies.push(json!({
            "metric": "conversion_rate",
            "level": "warning",
            "message": "转化率低于1%，建议优先复核商品详情页、价格和客服承接"
        }));
    }
    if matches!(ad_roi, Some(value) if value > 0.0 && value < 1.5) {
        anomalies.push(json!({
            "metric": "ad_roi",
            "level": "warning",
            "message": "广告ROI低于1.5，建议检查低效计划和素材疲劳"
        }));
    }
    if refund_rate > 8.0 {
        anomalies.push(json!({
            "metric": "refund_rate",
            "level": "warning",
            "message": "退款率高于8%，建议拆解售后原因"
        }));
    }
    if gross_margin > 0.0 && gross_margin < 15.0 {
        anomalies.push(json!({
            "metric": "gross_margin",
            "level": "warning",
            "message": "毛利率低于15%，需复核成本、优惠和平台扣点"
        }));
    }
    anomalies
}

fn product_metric_risks(
    gmv: f64,
    conversion_rate: f64,
    ad_roi: Option<f64>,
    refund_rate: f64,
    inventory: f64,
    stockout_count: f64,
) -> Vec<Value> {
    let mut risks = store_metric_anomalies(gmv, conversion_rate, ad_roi, refund_rate, 0.0);
    if inventory > 0.0 && inventory < 10.0 {
        risks.push(json!({
            "metric": "inventory",
            "level": "warning",
            "message": "库存低于10件，若仍有投放或活动承接，需检查补货和售罄风险"
        }));
    }
    if stockout_count > 0.0 {
        risks.push(json!({
            "metric": "stockout_count",
            "level": "warning",
            "message": "本轮输入存在缺货次数，建议评估缺货对转化和投放浪费的影响"
        }));
    }
    risks
}

fn product_metric_actions(
    conversion_rate: f64,
    ad_roi: Option<f64>,
    refund_rate: f64,
    inventory: f64,
    stockout_count: f64,
) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if conversion_rate > 0.0 && conversion_rate < 2.0 {
        actions.push("优先复核商品详情页卖点、价格带、评价区和客服承接，先提高商品转化率");
    }
    if matches!(ad_roi, Some(value) if value > 0.0 && value < 2.0) {
        actions.push("收缩低ROI商品投放计划，检查素材疲劳和关键词/人群匹配");
    }
    if refund_rate > 6.0 {
        actions.push("拆解退款原因，检查商品描述、规格、物流和售后口径是否一致");
    }
    if inventory > 0.0 && inventory < 10.0 {
        actions.push("低库存商品暂停大额加码，先确认补货周期和安全库存");
    }
    if stockout_count > 0.0 {
        actions.push("把缺货时段与广告消耗、访客和客服咨询量对齐，评估损失并补库存策略");
    }
    if actions.is_empty() {
        actions.push("维持当前商品节奏，下一轮重点观察转化率、ROI、退款率和库存周转");
    }
    actions
}

fn campaign_metric_risks(
    budget: f64,
    spend: f64,
    gmv: f64,
    conversion_rate: f64,
    roas: Option<f64>,
    refund_rate: f64,
    has_spend: bool,
) -> Vec<Value> {
    let mut risks = Vec::new();
    if has_spend && spend <= 0.0 {
        risks.push(json!({
            "metric": "spend",
            "level": "warning",
            "message": "活动花费为0，投放效率和ROI判断可信度不足"
        }));
    }
    if budget > 0.0 && spend > budget {
        risks.push(json!({
            "metric": "budget_usage",
            "level": "warning",
            "message": "活动花费已超过预算，建议检查预算控制和投放节奏"
        }));
    }
    if gmv <= 0.0 {
        risks.push(json!({
            "metric": "gmv",
            "level": "warning",
            "message": "活动GMV为0或未提供，无法判断活动产出"
        }));
    }
    if conversion_rate > 0.0 && conversion_rate < 1.0 {
        risks.push(json!({
            "metric": "conversion_rate",
            "level": "warning",
            "message": "活动转化率低于1%，需复核人群、价格、落地页和优惠承接"
        }));
    }
    if matches!(roas, Some(value) if value > 0.0 && value < 2.0) {
        risks.push(json!({
            "metric": "roas",
            "level": "warning",
            "message": "活动ROAS低于2，建议压缩低效计划或调整优惠机制"
        }));
    }
    if refund_rate > 8.0 {
        risks.push(json!({
            "metric": "refund_rate",
            "level": "warning",
            "message": "活动退款率偏高，可能存在冲动购买、描述不符或履约问题"
        }));
    }
    risks
}

fn campaign_metric_actions(
    budget_usage: f64,
    conversion_rate: f64,
    roas: Option<f64>,
    refund_rate: f64,
) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if budget_usage > 100.0 {
        actions.push("先冻结超预算活动单元，复核预算分配和出价策略");
    }
    if conversion_rate > 0.0 && conversion_rate < 1.5 {
        actions.push("优化活动落地页、利益点表达和客服承接，优先修正转化漏斗");
    }
    if matches!(roas, Some(value) if value > 0.0 && value < 2.0) {
        actions.push("按计划/素材/人群拆ROAS，保留高产出单元，暂停低效单元");
    }
    if refund_rate > 8.0 {
        actions.push("把活动订单按SKU和退款原因拆解，避免用高退款活动继续放量");
    }
    if actions.is_empty() {
        actions.push("活动指标未见明显高危信号，下一轮继续按预算消耗、ROAS和转化率复盘");
    }
    actions
}

fn normalize_metric_platform(input: &Value) -> String {
    let platform = s(input, "platform", "all");
    if platform.trim().is_empty() {
        "all".to_owned()
    } else {
        platform
    }
}

fn empty_string_as_null(value: String) -> Value {
    if value.trim().is_empty() {
        Value::Null
    } else {
        json!(value)
    }
}
