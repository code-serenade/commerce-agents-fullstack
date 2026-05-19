use serde_json::{Value, json};

use super::{
    common::{
        average, empty_string_as_null, i, ints, json_ok, linear_fit, metric_display,
        moving_average, n, nums, pct, percent, required_string, round2, round4, s, signed_percent,
    },
    common_spec::deferred,
};
use crate::skills::SkillError;

pub(super) fn customer_segmentation(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let total = i(input, "total_customers", 1000).max(1);
    let aov = n(input, "avg_order_value", 120.0);
    let freq = n(input, "avg_orders_per_customer", 1.8);
    let recency = n(input, "recency_days", 30.0);
    let high_value = ((total as f64) * (0.12 + (freq / 20.0).min(0.08))).round() as i64;
    let loyal = ((total as f64) * 0.18).round() as i64;
    let attention = ((total as f64) * if recency <= 30.0 { 0.30 } else { 0.24 }).round() as i64;
    let lost = (total - high_value - loyal - attention).max(0);
    json!({
        "数据来源": "用户提供/默认估算",
        "商品引用": empty_string_as_null(product_ref),
        "客户总数": total,
        "RFM输入": {"平均客单价": round2(aov), "人均订单数": round2(freq), "平均购买间隔": round2(recency)},
        "分群": [
            {"客群标签": "Champions（冠军）", "客户数": high_value, "占比": percent(pct(high_value as f64, total as f64), 1), "策略": "专属折扣奖励，新品优先体验"},
            {"客群标签": "Loyal（忠诚客户）", "客户数": loyal, "占比": percent(pct(loyal as f64, total as f64), 1), "策略": "会员积分升级，推荐关联商品"},
            {"客群标签": "Need Attention（需关注）", "客户数": attention, "占比": percent(pct(attention as f64, total as f64), 1), "策略": "限时活动唤醒，个性化push"},
            {"客群标签": "Lost（已流失）", "客户数": lost, "占比": percent(pct(lost as f64, total as f64), 1), "策略": "低成本召回测试，无响应则停止触达"}
        ],
        "加权RFM说明": "R×0.15 + F×0.28 + M×0.57，当前为聚合输入估算"
    })
}

pub(super) fn competitor_analysis(input: &Value) -> Result<Value, SkillError> {
    let product_ref = s(input, "product_ref", "");
    let product = required_string(input, "our_product")?;
    let our_price = n(input, "our_price", 100.0).max(0.01);
    let competitors = input
        .get("competitors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| {
            vec![
                json!({"name": "竞品A", "price": our_price * 0.89, "monthly_sales": 5000, "rating": 4.7}),
                json!({"name": "竞品B", "price": our_price * 1.10, "monthly_sales": 3000, "rating": 4.5}),
            ]
        });
    let prices = competitors
        .iter()
        .map(|item| n(item, "price", our_price))
        .collect::<Vec<_>>();
    let avg_price = if prices.is_empty() {
        our_price
    } else {
        average(&prices)
    };
    let matrix = competitors
        .iter()
        .map(|item| {
            let price = n(item, "price", our_price).max(0.01);
            let diff = pct(our_price - price, price);
            json!({
                "竞品": s(item, "name", "未知"),
                "价格": round2(price),
                "月销量": i(item, "monthly_sales", 0),
                "评分": n(item, "rating", 0.0),
                "价格差异": signed_percent(diff, 1),
                "我方优势": if diff < 0.0 { "价格更低" } else { "需要用卖点或服务支撑溢价" }
            })
        })
        .collect::<Vec<_>>();
    json_ok(json!({
        "商品引用": empty_string_as_null(product_ref),
        "我方商品": product,
        "我方价格": round2(our_price),
        "竞品矩阵": matrix,
        "价格定位": if our_price < avg_price * 0.9 { "低价位" } else if our_price < avg_price * 1.1 { "中价位" } else { "高价位" },
        "竞品均价": round2(avg_price),
        "策略建议": ["明确差异化卖点", "监控最低价竞品", "用赠品/服务降低直接比价"]
    }))
}

pub(super) fn multi_period_trend(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let values = nums(input, "values");
    if values.len() < 4 {
        return json!({"error": "数据不足", "hint": "请提供 values 至少4个数据点"});
    }
    let periods = ints(input, "periods", vec![7, 14, 30]);
    let ma_window = i(input, "ma_window", 7).max(1) as usize;
    let comparisons = periods
        .into_iter()
        .filter(|period| (*period as usize) * 2 <= values.len())
        .map(|period| {
            let p = period as usize;
            let recent = average(&values[values.len() - p..]);
            let previous = average(&values[values.len() - p * 2..values.len() - p]);
            json!({"周期": format!("{period}天"), "近期均值": round2(recent), "前期均值": round2(previous), "环比": signed_percent(pct(recent - previous, previous), 2)})
        })
        .collect::<Vec<_>>();
    let ma = moving_average(&values, ma_window);
    let (slope, _) = linear_fit(&values);
    json!({
        "商品引用": empty_string_as_null(product_ref),
        "指标": metric_display(&s(input, "metric", "gmv")),
        "数据点数": values.len(),
        "最新值": round2(*values.last().unwrap_or(&0.0)),
        "移动平均(最近5天)": ma.into_iter().rev().take(5).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>(),
        "多周期环比": comparisons,
        "趋势动能": if slope > 0.0 { "上行" } else if slope < 0.0 { "下行" } else { "平稳" },
        "斜率": round4(slope)
    })
}

pub(super) fn channel_roi(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let channels = input
        .get("channels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if channels.is_empty() {
        return deferred(
            "data_channel_roi",
            "需要真实渠道GMV、广告花费、订单和UV数据",
            input,
        );
    }
    let mut rows = channels
        .iter()
        .map(|channel| {
            let gmv = n(channel, "gmv", 0.0);
            let ad = n(channel, "ad_spend", 0.0);
            let orders = i(channel, "orders", 0).max(1) as f64;
            let uv = i(channel, "uv", 0).max(1) as f64;
            let roi = if ad > 0.0 { gmv / ad } else { 0.0 };
            json!({
                "渠道": s(channel, "name", "未知渠道"),
                "GMV贡献": round2(gmv),
                "广告花费": round2(ad),
                "广告ROI": round2(roi),
                "获客成本(CPO)": round2(ad / orders),
                "点击成本(CPC)": round4(ad / uv)
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| n(b, "广告ROI", 0.0).total_cmp(&n(a, "广告ROI", 0.0)));
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": "user_provided",
        "adapter_status": "manual_input",
        "summary": "渠道ROI分析完成",
        "商品引用": empty_string_as_null(product_ref),
        "渠道ROI排名": rows,
        "优化建议": ["加码高ROI渠道", "ROI低于2的渠道优先优化创意和人群"]
    })
}
