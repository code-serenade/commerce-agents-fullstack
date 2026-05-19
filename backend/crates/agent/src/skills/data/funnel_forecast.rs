use serde_json::{Value, json};

use super::{
    common::{
        average, avg_growth, empty_string_as_null, i, linear_fit, metric_display, n, nums, pct,
        percent, round2, round4, s, stddev,
    },
    common_spec::deferred,
};

pub(super) fn funnel_analysis(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let impressions = i(input, "impressions", 0).max(i(input, "clicks", 0));
    let clicks = i(input, "clicks", 0);
    let orders = i(input, "orders", 0);
    let add_to_cart = i(input, "add_to_cart", (orders as f64 * 2.5).round() as i64);
    let payments = i(input, "payments", (orders as f64 * 0.95).round() as i64);
    if impressions <= 0 && clicks <= 0 {
        return deferred(
            "data_funnel_analysis",
            "需要曝光、点击、加购、下单或付款等漏斗数据",
            input,
        );
    }

    let stages = [
        (
            "曝光→点击",
            impressions,
            clicks,
            5.0,
            "优化主图和标题关键词，提升点击率",
        ),
        (
            "点击→加购",
            clicks,
            add_to_cart,
            30.0,
            "改善详情页、增强价格竞争力",
        ),
        (
            "加购→下单",
            add_to_cart,
            orders,
            35.0,
            "设置限时优惠和库存紧迫感",
        ),
        (
            "下单→付款",
            orders,
            payments,
            90.0,
            "简化支付流程，增加支付方式",
        ),
    ];
    let mut bottleneck = "无明显瓶颈";
    let mut worst_gap = f64::MIN;
    let funnel = stages
        .iter()
        .filter(|(_, upper, _, _, _)| *upper > 0)
        .map(|(name, upper, lower, benchmark, _)| {
            let rate = pct(*lower as f64, *upper as f64);
            let gap = benchmark - rate;
            if gap > worst_gap {
                worst_gap = gap;
                bottleneck = name;
            }
            json!({
                "阶段": name,
                "转化率": percent(rate, 2),
                "基准值": percent(*benchmark, 1),
                "差距": percent(gap, 2),
                "状态": if gap <= 0.0 { "正常" } else if gap < 5.0 { "需关注" } else { "异常" }
            })
        })
        .collect::<Vec<_>>();
    let suggestion = stages
        .iter()
        .find(|(name, _, _, _, _)| *name == bottleneck)
        .map(|(_, _, _, _, suggestion)| *suggestion)
        .unwrap_or("持续监控各环节");

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": "user_provided",
        "adapter_status": "manual_input",
        "summary": format!("漏斗分析完成，瓶颈环节：{bottleneck}"),
        "数据来源": "用户提供",
        "商品引用": empty_string_as_null(product_ref),
        "漏斗分析": funnel,
        "整体转化率": percent(pct(payments as f64, impressions.max(1) as f64), 3),
        "瓶颈环节": bottleneck,
        "优化建议": suggestion
    })
}

pub(super) fn anomaly_diagnosis(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let values = nums(input, "values");
    if values.len() < 3 {
        return json!({"error": "无真实数据且未提供足够时序数据", "required": "values至少3个数字"});
    }
    let threshold = n(input, "threshold", 2.0).abs().max(0.1);
    let mean = average(&values);
    let std = stddev(&values).max(1e-9);
    let anomalies = values
        .iter()
        .enumerate()
        .filter_map(|(idx, value)| {
            let z = (*value - mean) / std;
            (z.abs() >= threshold).then(|| {
                json!({"第几天": idx + 1, "数值": round2(*value), "Z-Score": round2(z), "方向": if z > 0.0 { "激增" } else { "骤降" }})
            })
        })
        .collect::<Vec<_>>();
    json!({
        "数据来源": "用户提供",
        "商品引用": empty_string_as_null(product_ref),
        "指标": s(input, "metric_name", "未知指标"),
        "样本数": values.len(),
        "均值": round2(mean),
        "标准差": round2(std),
        "异常点数": anomalies.len(),
        "异常详情": anomalies,
        "建议": if anomalies.is_empty() { "指标运行正常" } else { "立即排查异常原因" }
    })
}

pub(super) fn trend_forecast(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let values = nums(input, "values");
    if values.len() < 2 {
        return json!({"error": "数据不足，无法预测", "hint": "请提供 values 至少2个数据点"});
    }
    let periods = i(input, "forecast_periods", 7).max(1) as usize;
    let (slope, intercept) = linear_fit(&values);
    let forecasts = (0..periods)
        .map(|idx| json!({"期数": format!("第{}天", idx + 1), "预测值": round2((intercept + slope * (values.len() + idx) as f64).max(0.0))}))
        .collect::<Vec<_>>();
    let avg_growth = avg_growth(&values);
    let trend = if slope > average(&values).abs() * 0.01 {
        "上升"
    } else if slope < -average(&values).abs() * 0.01 {
        "下降"
    } else {
        "平稳"
    };
    json!({
        "数据来源": "用户提供",
        "商品引用": empty_string_as_null(product_ref),
        "指标": metric_display(&s(input, "metric_name", "gmv")),
        "历史数据点数": values.len(),
        "最新值": round2(*values.last().unwrap_or(&0.0)),
        "趋势方向": trend,
        "平均日增长率": percent(avg_growth, 2),
        "斜率": round4(slope),
        "预测结果": forecasts,
        "置信说明": "基于线性回归，反映近期趋势，不含季节性因素"
    })
}
