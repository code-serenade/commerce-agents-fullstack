use serde_json::{Value, json};

use super::{
    common::{
        average, empty_string_as_null, i, ints, mape, metric_display, moving_average, n, nums, pct,
        percent, round2, round4, s, ses_fitted, ses_level, signed_percent, stddev,
    },
    common_spec::deferred,
};

pub(super) fn refund_decomposition(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let items = input
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if items.is_empty() {
        return deferred(
            "data_refund_decomposition",
            "需要退款金额、退款率或渠道退款明细",
            input,
        );
    }
    let mut rows = items
        .iter()
        .map(|item| {
            let amount = n(
                item,
                "refund_amount",
                n(item, "gmv", 0.0) * n(item, "refund_rate", 0.03),
            );
            (amount, item)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    let total = rows.iter().map(|(amount, _)| *amount).sum::<f64>().max(1.0);
    let mut cumulative = 0.0;
    let top_n = i(input, "top_n", 5).max(1) as usize;
    let pareto = rows
        .iter()
        .take(top_n)
        .map(|(amount, item)| {
            let share = *amount / total * 100.0;
            cumulative += share;
            json!({"项目": s(item, "name", "未知"), "退款金额": round2(*amount), "占总退款比": percent(share, 1), "累计占比": percent(cumulative.min(100.0), 1), "是否80%内": cumulative <= 80.0})
        })
        .collect::<Vec<_>>();
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": "user_provided",
        "adapter_status": "manual_input",
        "summary": "退款拆解分析完成",
        "商品引用": empty_string_as_null(product_ref),
        "总退款估算": round2(total),
        "帕累托分析": pareto,
        "整改建议": ["优先处理累计80%内的问题项", "复核详情页描述、质检和物流包装"]
    })
}

pub(super) fn seasonal_decompose(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let series = nums(input, "values");
    let period = i(input, "period", 7).max(2) as usize;
    if series.len() < period * 2 {
        return json!({"has_data": false, "提示": format!("数据点不足（需≥{}个），当前仅{}个", period * 2, series.len())});
    }
    let ma = moving_average(&series, period);
    let trend_start = ma.first().copied().unwrap_or(series[0]);
    let trend_end = ma
        .last()
        .copied()
        .unwrap_or(*series.last().unwrap_or(&series[0]));
    let trend_growth = pct(trend_end - trend_start, trend_start);
    let mut indices = Vec::new();
    for p in 0 .. period {
        let vals = series
            .iter()
            .enumerate()
            .filter_map(|(idx, value)| (idx % period == p).then_some(*value))
            .collect::<Vec<_>>();
        indices.push(average(&vals) / average(&series).max(1e-9));
    }
    let strength = stddev(&indices) * 100.0;
    json!({
        "has_data": true,
        "商品引用": empty_string_as_null(product_ref),
        "指标": metric_display(&s(input, "metric", "gmv")),
        "季节周期": format!("{period}天"),
        "趋势分析": {"趋势起点": round2(trend_start), "趋势终点": round2(trend_end), "趋势增长率": signed_percent(trend_growth, 2)},
        "季节性分析": {"季节性强度": percent(strength, 1), "季节指数": indices.iter().map(|v| round4(*v)).collect::<Vec<_>>()},
        "运营建议": if trend_growth >= 0.0 { "趋势向上，峰值周期前加大备货和投放" } else { "趋势下行，优先排查流量、价格和竞品冲击" }
    })
}

pub(super) fn price_elasticity(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let points = input
        .get("price_points")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| {
            vec![
                json!({"price": 90.0, "quantity": 560.0}),
                json!({"price": 110.0, "quantity": 440.0}),
            ]
        });
    let pairs = points
        .iter()
        .filter_map(|point| {
            let price = n(point, "price", 0.0);
            let qty = n(point, "quantity", 0.0);
            (price > 0.0 && qty > 0.0).then_some((price, qty))
        })
        .collect::<Vec<_>>();
    if pairs.len() < 2 {
        return json!({"error": "至少需要2个有效 price_points"});
    }
    let mut elasticities = Vec::new();
    for pair in pairs.windows(2) {
        let (p1, q1) = pair[0];
        let (p2, q2) = pair[1];
        let pct_q = (q2 - q1) / ((q1 + q2) / 2.0);
        let pct_p = (p2 - p1) / ((p1 + p2) / 2.0);
        if pct_p.abs() > 1e-9 {
            elasticities.push(pct_q / pct_p);
        }
    }
    let e = average(&elasticities);
    let cost = n(input, "cost", pairs[0].0 * 0.45);
    let current = n(
        input,
        "current_price",
        pairs.last().map(|(p, _)| *p).unwrap_or(100.0),
    );
    let optimal = if e < -1.0 {
        cost * e / (e + 1.0)
    } else {
        current
    };
    json!({
        "商品引用": empty_string_as_null(product_ref),
        "价格弹性系数": round4(e),
        "敏感度分类": if e.abs() > 1.5 { "高弹性" } else if e.abs() > 0.8 { "中弹性" } else { "低弹性" },
        "当前售价": round2(current),
        "Lerner最优价参考": round2(optimal.max(cost * 1.05)),
        "建议": if e < -1.0 { "需求对价格敏感，降价促量需同步核算利润" } else { "需求弹性较低，可测试小幅提价" }
    })
}

pub(super) fn demand_forecast(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let values = nums(input, "values");
    if values.len() < 3 {
        return json!({"error": "数据不足", "hint": "请提供 values 至少3个数据点"});
    }
    let horizons = ints(input, "forecast_days", vec![7, 14, 30]);
    let alphas = [0.2, 0.4, 0.6, 0.8];
    let mut best_alpha = 0.2;
    let mut best_mape = f64::INFINITY;
    for alpha in alphas {
        let preds = ses_fitted(&values, alpha);
        let score = mape(&values[1 ..], &preds);
        if score < best_mape {
            best_mape = score;
            best_alpha = alpha;
        }
    }
    let level = ses_level(&values, best_alpha);
    let sigma = stddev(&values);
    let forecasts = horizons
        .into_iter()
        .map(|days| json!({"未来天数": days, "预测总量": round2(level * days as f64), "日均预测": round2(level), "置信区间": [round2((level - 1.96 * sigma).max(0.0)), round2(level + 1.96 * sigma)]}))
        .collect::<Vec<_>>();
    json!({
        "商品引用": empty_string_as_null(product_ref),
        "指标": metric_display(&s(input, "metric", "orders")),
        "历史数据点数": values.len(),
        "最优alpha": best_alpha,
        "MAPE": percent(best_mape, 2),
        "预测结果": forecasts,
        "备货建议": format!("按日均{}的预测水平备货，并预留约{}安全缓冲", round2(level), round2(1.65 * sigma))
    })
}
