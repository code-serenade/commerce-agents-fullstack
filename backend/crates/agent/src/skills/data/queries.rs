fn query_store_metrics(input: &Value) -> Value {
    let days = i(input, "days", 30).clamp(1, 365);
    let platform_filter = {
        let value = s(input, "platform", "all");
        if value.trim().is_empty() {
            "all".to_owned()
        } else {
            value
        }
    };
    let include_anomalies = input
        .get("include_anomalies")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let metric_keys = [
        "gmv",
        "revenue",
        "previous_gmv",
        "last_gmv",
        "orders",
        "order_count",
        "previous_orders",
        "last_orders",
        "visitors",
        "uv",
        "previous_visitors",
        "last_visitors",
        "clicks",
        "click_count",
        "ad_spend",
        "ad_cost",
        "refund_amount",
        "refunds",
        "gross_profit",
        "profit",
    ];
    let platform_rows = store_metric_platform_rows(input, &platform_filter);
    let has_explicit_metrics = has_any_input(input, &metric_keys) || !platform_rows.is_empty();

    if !has_explicit_metrics {
        return json!({
            "status": "deferred",
            "skill": "query_store_metrics",
            "has_data": false,
            "adapter_status": "store_metrics_required",
            "data_readiness": "missing_user_input",
            "deferred_reason": "当前没有真实店铺指标输入，也没有 store_metrics adapter；不会伪造平台实时数据",
            "side_effects": "none",
            "received_params": input,
            "expected_inputs": [
                "gmv/revenue",
                "orders/order_count",
                "visitors/uv",
                "clicks/click_count",
                "ad_spend/ad_cost",
                "refund_amount/refunds",
                "gross_profit/profit",
                "platforms[]"
            ],
            "next_actions": [
                "在本轮输入中提供店铺经营指标",
                "后续通过 service 层接入已鉴权的 store_metrics adapter",
                "前端展示为缺数据状态，不要当作平台实时读取结果"
            ],
            "summary": "店铺指标查询等待真实数据输入"
        });
    }

    let totals = store_metric_totals(input, &platform_rows);
    let previous = store_metric_previous(input);
    let gmv = value_pointer_number(&totals, "/gmv").unwrap_or_default();
    let orders = value_pointer_number(&totals, "/orders").unwrap_or_default();
    let visitors = value_pointer_number(&totals, "/uv").unwrap_or_default();
    let clicks = value_pointer_number(&totals, "/clicks").unwrap_or_default();
    let ad_spend = value_pointer_number(&totals, "/ad_spend").unwrap_or_default();
    let refund_amount = value_pointer_number(&totals, "/refund_amount").unwrap_or_default();
    let gross_profit = value_pointer_number(&totals, "/gross_profit").unwrap_or_default();

    let conversion_rate = pct(orders, visitors.max(1.0));
    let ctr = pct(clicks, visitors.max(1.0));
    let aov = if orders > 0.0 { gmv / orders } else { 0.0 };
    let ad_roi = if ad_spend > 0.0 {
        Some(round2(gmv / ad_spend))
    } else {
        None
    };
    let refund_rate = pct(refund_amount, gmv.max(1.0));
    let gross_margin = pct(gross_profit, gmv.max(1.0));
    let week_over_week = store_metric_week_over_week(gmv, orders, visitors, &previous);
    let anomalies = if include_anomalies {
        store_metric_anomalies(gmv, conversion_rate, ad_roi, refund_rate, gross_margin)
    } else {
        Vec::new()
    };
    let anomaly_count = anomalies.len();

    json!({
        "status": "success",
        "skill": "query_store_metrics",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": "explicit_metrics",
        "数据来源": "用户提供指标",
        "days": days,
        "date_range": {
            "mode": "relative",
            "days": days
        },
        "platform": platform_filter,
        "platforms": platform_rows,
        "totals": totals,
        "derived_metrics": {
            "conversion_rate": percent(conversion_rate, 2),
            "ctr": percent(ctr, 2),
            "aov": round2(aov),
            "ad_roi": ad_roi,
            "refund_rate": percent(refund_rate, 2),
            "gross_margin": percent(gross_margin, 2)
        },
        "week_over_week": week_over_week,
        "anomalies": anomalies,
        "anomaly_count": anomaly_count,
        "summary": format!("店铺指标已从本轮显式输入汇总，GMV={}，订单={}", round2(gmv), round2(orders)),
        "side_effects": "none",
        "说明": "本结果只基于本轮明确提供的指标，不代表已读取淘宝、抖音或其它平台实时数据。"
    })
}

fn query_product_metrics(input: &Value) -> Value {
    let days = i(input, "days", 30).clamp(1, 365);
    let platform = normalize_metric_platform(input);
    let product_ref = s(input, "product_ref", "");
    let product_id = s(input, "product_id", "");
    let product_name = s(input, "product_name", "");
    let sku_id = s(input, "sku_id", "");
    let metric_keys = [
        "gmv",
        "revenue",
        "previous_gmv",
        "last_gmv",
        "orders",
        "order_count",
        "previous_orders",
        "last_orders",
        "visitors",
        "uv",
        "clicks",
        "click_count",
        "ad_spend",
        "ad_cost",
        "refund_amount",
        "refunds",
        "gross_profit",
        "profit",
        "inventory",
        "stock",
        "stockout_count",
    ];
    let has_explicit_metrics = has_any_input(input, &metric_keys);
    if !has_explicit_metrics {
        return json!({
            "status": "deferred",
            "skill": "query_product_metrics",
            "has_data": false,
            "adapter_status": "product_metrics_required",
            "data_readiness": "missing_user_input",
            "deferred_reason": "当前没有本轮显式商品指标输入，也没有 product_metrics adapter；不会默认读取任何预设商品或伪造平台实时商品数据",
            "side_effects": "none",
            "query": {
                "product_ref": empty_string_as_null(product_ref),
                "product_id": empty_string_as_null(product_id),
                "product_name": empty_string_as_null(product_name),
                "sku_id": empty_string_as_null(sku_id),
                "platform": platform,
                "days": days
            },
            "expected_inputs": [
                "product_ref/product_id/product_name/sku_id",
                "gmv/revenue",
                "orders/order_count",
                "visitors/uv",
                "clicks/click_count",
                "ad_spend/ad_cost",
                "refund_amount/refunds",
                "gross_profit/profit",
                "inventory/stock",
                "stockout_count"
            ],
            "next_actions": [
                "在本轮输入中提供商品或SKU经营指标",
                "或先通过前端引用商品并由后续 service adapter 注入真实商品指标",
                "前端展示为缺商品指标状态，不要当作已读取平台实时商品数据"
            ],
            "answerPolicy": {
                "no_default_product": true,
                "live_platform_lookup": false,
                "source": "explicit_product_metrics_required"
            },
            "summary": "商品指标查询等待真实数据输入"
        });
    }

    let (gmv, _) = number_from_any(input, &["gmv", "revenue"], 0.0);
    let (orders, _) = number_from_any(input, &["orders", "order_count"], 0.0);
    let (visitors, _) = number_from_any(input, &["visitors", "uv"], 0.0);
    let (clicks, _) = number_from_any(input, &["clicks", "click_count"], 0.0);
    let (ad_spend, _) = number_from_any(input, &["ad_spend", "ad_cost"], 0.0);
    let (refund_amount, _) = number_from_any(input, &["refund_amount", "refunds"], 0.0);
    let (gross_profit, _) = number_from_any(input, &["gross_profit", "profit"], 0.0);
    let (inventory, _) = number_from_any(input, &["inventory", "stock"], 0.0);
    let (stockout_count, _) = number_from_any(input, &["stockout_count"], 0.0);
    let previous = store_metric_previous(input);
    let conversion_rate = pct(orders, visitors.max(1.0));
    let ctr = pct(clicks, visitors.max(1.0));
    let aov = if orders > 0.0 { gmv / orders } else { 0.0 };
    let ad_roi = if ad_spend > 0.0 {
        Some(round2(gmv / ad_spend))
    } else {
        None
    };
    let refund_rate = pct(refund_amount, gmv.max(1.0));
    let gross_margin = pct(gross_profit, gmv.max(1.0));
    let sell_through_pressure = if inventory > 0.0 {
        round2(orders / inventory)
    } else {
        0.0
    };
    let risks = product_metric_risks(
        gmv,
        conversion_rate,
        ad_roi,
        refund_rate,
        inventory,
        stockout_count,
    );

    json!({
        "status": "success",
        "skill": "query_product_metrics",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": "explicit_product_metrics",
        "数据来源": "用户提供商品指标",
        "days": days,
        "date_range": {
            "mode": "relative",
            "days": days
        },
        "query": {
            "product_ref": empty_string_as_null(product_ref),
            "product_id": empty_string_as_null(product_id),
            "product_name": empty_string_as_null(product_name),
            "sku_id": empty_string_as_null(sku_id),
            "platform": platform
        },
        "totals": {
            "gmv": round2(gmv),
            "orders": round2(orders),
            "uv": round2(visitors),
            "clicks": round2(clicks),
            "ad_spend": round2(ad_spend),
            "refund_amount": round2(refund_amount),
            "gross_profit": round2(gross_profit),
            "inventory": round2(inventory),
            "stockout_count": round2(stockout_count)
        },
        "derived_metrics": {
            "conversion_rate": percent(conversion_rate, 2),
            "ctr": percent(ctr, 2),
            "aov": round2(aov),
            "ad_roi": ad_roi,
            "refund_rate": percent(refund_rate, 2),
            "gross_margin": percent(gross_margin, 2),
            "sell_through_pressure": sell_through_pressure
        },
        "week_over_week": store_metric_week_over_week(gmv, orders, visitors, &previous),
        "risks": risks,
        "next_actions": product_metric_actions(conversion_rate, ad_roi, refund_rate, inventory, stockout_count),
        "answerPolicy": {
            "no_default_product": true,
            "live_platform_lookup": false,
            "source": "user_explicit_product_metrics"
        },
        "summary": format!("商品指标已从本轮显式输入汇总，GMV={}，订单={}", round2(gmv), round2(orders)),
        "side_effects": "none",
        "说明": "本结果只基于本轮明确提供的商品指标，不代表已读取淘宝、抖音、千川或其它平台实时商品数据。"
    })
}

fn query_campaign_metrics(input: &Value) -> Value {
    let days = i(input, "days", 30).clamp(1, 365);
    let platform = normalize_metric_platform(input);
    let campaign_id = s(input, "campaign_id", "");
    let campaign_name = s(input, "campaign_name", "");
    let campaign_type = s(input, "campaign_type", "general");
    let metric_keys = [
        "budget",
        "spend",
        "ad_spend",
        "gmv",
        "revenue",
        "previous_gmv",
        "last_gmv",
        "orders",
        "order_count",
        "visitors",
        "uv",
        "clicks",
        "click_count",
        "impressions",
        "exposure",
        "coupon_cost",
        "refund_amount",
        "refunds",
    ];
    let has_explicit_metrics = has_any_input(input, &metric_keys);
    if !has_explicit_metrics {
        return json!({
            "status": "deferred",
            "skill": "query_campaign_metrics",
            "has_data": false,
            "adapter_status": "campaign_metrics_required",
            "data_readiness": "missing_user_input",
            "deferred_reason": "当前没有本轮显式活动指标输入，也没有 campaign_metrics adapter；不会伪造活动、投放或平台实时数据",
            "side_effects": "none",
            "query": {
                "campaign_id": empty_string_as_null(campaign_id),
                "campaign_name": empty_string_as_null(campaign_name),
                "campaign_type": campaign_type,
                "platform": platform,
                "days": days
            },
            "expected_inputs": [
                "campaign_id/campaign_name/campaign_type",
                "budget",
                "spend/ad_spend",
                "gmv/revenue",
                "orders/order_count",
                "visitors/uv",
                "clicks/click_count",
                "impressions/exposure",
                "coupon_cost",
                "refund_amount/refunds"
            ],
            "next_actions": [
                "在本轮输入中提供活动预算、花费、GMV、曝光、点击或订单等指标",
                "后续通过 service 层接入已鉴权的 campaign_metrics adapter",
                "前端展示为缺活动指标状态，不要当作平台实时活动复盘结果"
            ],
            "answerPolicy": {
                "live_platform_lookup": false,
                "source": "explicit_campaign_metrics_required"
            },
            "summary": "活动指标查询等待真实数据输入"
        });
    }

    let (budget, _) = number_from_any(input, &["budget"], 0.0);
    let (spend, has_spend) = number_from_any(input, &["spend", "ad_spend"], 0.0);
    let (gmv, _) = number_from_any(input, &["gmv", "revenue"], 0.0);
    let (orders, _) = number_from_any(input, &["orders", "order_count"], 0.0);
    let (visitors, _) = number_from_any(input, &["visitors", "uv"], 0.0);
    let (clicks, _) = number_from_any(input, &["clicks", "click_count"], 0.0);
    let (impressions, _) = number_from_any(input, &["impressions", "exposure"], 0.0);
    let (coupon_cost, _) = number_from_any(input, &["coupon_cost"], 0.0);
    let (refund_amount, _) = number_from_any(input, &["refund_amount", "refunds"], 0.0);
    let total_cost = spend + coupon_cost;
    let conversion_rate = pct(orders, visitors.max(1.0));
    let ctr = pct(clicks, impressions.max(1.0));
    let cpc = if clicks > 0.0 {
        Some(round2(spend / clicks))
    } else {
        None
    };
    let cpm = if impressions > 0.0 {
        Some(round2(spend / impressions * 1000.0))
    } else {
        None
    };
    let roas = if spend > 0.0 {
        Some(round2(gmv / spend))
    } else {
        None
    };
    let roi = if total_cost > 0.0 {
        Some(round2((gmv - total_cost) / total_cost))
    } else {
        None
    };
    let budget_usage = pct(spend, budget.max(1.0));
    let refund_rate = pct(refund_amount, gmv.max(1.0));
    let previous = store_metric_previous(input);
    let risks = campaign_metric_risks(
        budget,
        spend,
        gmv,
        conversion_rate,
        roas,
        refund_rate,
        has_spend,
    );

    json!({
        "status": "success",
        "skill": "query_campaign_metrics",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": "explicit_campaign_metrics",
        "数据来源": "用户提供活动指标",
        "days": days,
        "date_range": {
            "mode": "relative",
            "days": days
        },
        "query": {
            "campaign_id": empty_string_as_null(campaign_id),
            "campaign_name": empty_string_as_null(campaign_name),
            "campaign_type": campaign_type,
            "platform": platform
        },
        "totals": {
            "budget": round2(budget),
            "spend": round2(spend),
            "coupon_cost": round2(coupon_cost),
            "total_cost": round2(total_cost),
            "gmv": round2(gmv),
            "orders": round2(orders),
            "uv": round2(visitors),
            "clicks": round2(clicks),
            "impressions": round2(impressions),
            "refund_amount": round2(refund_amount)
        },
        "derived_metrics": {
            "budget_usage": percent(budget_usage, 2),
            "ctr": percent(ctr, 2),
            "conversion_rate": percent(conversion_rate, 2),
            "cpc": cpc,
            "cpm": cpm,
            "roas": roas,
            "roi": roi,
            "refund_rate": percent(refund_rate, 2)
        },
        "week_over_week": store_metric_week_over_week(gmv, orders, visitors, &previous),
        "risks": risks,
        "next_actions": campaign_metric_actions(budget_usage, conversion_rate, roas, refund_rate),
        "answerPolicy": {
            "live_platform_lookup": false,
            "source": "user_explicit_campaign_metrics"
        },
        "summary": format!("活动指标已从本轮显式输入汇总，GMV={}，花费={}", round2(gmv), round2(spend)),
        "side_effects": "none",
        "说明": "本结果只基于本轮明确提供的活动指标，不代表已读取淘宝、抖音、千川或其它平台实时活动数据。"
    })
}

