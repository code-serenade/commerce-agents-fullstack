use serde_json::{Value, json};

use super::super::support::{
    average, calc_scenario, currency, deferred_accounting, deterministic_risk_samples, num, pct,
    percent, percentile, signed_percent,
};

pub(in crate::skills::accounting) fn scenario_analysis(input: &Value) -> Value {
    let base_gmv = num(input, "base_gmv", 0.0);
    if base_gmv <= 0.0 {
        return deferred_accounting(
            "accounting_scenario_analysis",
            "请提供基准GMV（月销售额）。",
            input,
        );
    }
    let base_margin = num(input, "base_margin_pct", 30.0) / 100.0;
    let base_ad = num(input, "base_ad_pct", 8.0) / 100.0;
    let base_refund = num(input, "base_refund_pct", 5.0) / 100.0;
    let fixed_cost = num(input, "fixed_cost", 0.0);
    let opt_growth = num(input, "optimistic_growth", 30.0) / 100.0;
    let pes_growth = num(input, "pessimistic_growth", -20.0) / 100.0;

    let scenarios = vec![
        calc_scenario(
            base_gmv * (1.0 + opt_growth),
            format!("乐观 ({:+.0}% GMV)", opt_growth * 100.0),
            base_margin,
            base_ad,
            base_refund,
            fixed_cost,
        ),
        calc_scenario(
            base_gmv,
            "基准 (当前水平)".to_owned(),
            base_margin,
            base_ad,
            base_refund,
            fixed_cost,
        ),
        calc_scenario(
            base_gmv * (1.0 + pes_growth),
            format!("悲观 ({:+.0}% GMV)", pes_growth * 100.0),
            base_margin,
            base_ad,
            base_refund,
            fixed_cost,
        ),
    ];

    let price_changes = [-0.20, -0.10, 0.0, 0.10, 0.20];
    let traffic_changes = [-0.20, -0.10, 0.0, 0.10, 0.20];
    let sensitivity_matrix: Vec<_> = price_changes
        .iter()
        .map(|pc| {
            traffic_changes
                .iter()
                .map(|tc| {
                    let adj_gmv = base_gmv * (1.0 + pc) * (1.0 + tc);
                    let row = calc_scenario(
                        adj_gmv,
                        String::new(),
                        base_margin,
                        base_ad,
                        base_refund,
                        fixed_cost,
                    );
                    json!({
                        "价格变动": signed_percent(pc * 100.0, 0),
                        "流量变动": signed_percent(tc * 100.0, 0),
                        "净利润": row["税后净利润"],
                        "净利率": row["净利率"],
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let net_contribution_rate = (base_margin - base_ad - 0.04 - 0.06) * (1.0 - base_refund);
    let bep_gmv = if net_contribution_rate > 0.0 {
        fixed_cost / net_contribution_rate
    } else {
        0.0
    };

    let samples =
        deterministic_risk_samples(base_gmv, base_margin, base_ad, base_refund, fixed_cost);
    let mut sorted = samples.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let expected = average(&samples);
    let stddev = (samples.iter().map(|x| (x - expected).powi(2)).sum::<f64>() / n as f64).sqrt();
    let mc_summary = json!({
        "模拟次数": n,
        "P10（最差10%情景净利润）": percentile(&sorted, 10.0).round(),
        "P25（下四分位净利润）": percentile(&sorted, 25.0).round(),
        "P50（中位净利润）": percentile(&sorted, 50.0).round(),
        "P75（上四分位净利润）": percentile(&sorted, 75.0).round(),
        "P90（最优10%情景净利润）": percentile(&sorted, 90.0).round(),
        "亏损概率": percent(samples.iter().filter(|x| **x < 0.0).count() as f64 / n as f64 * 100.0, 1),
        "期望净利润": expected.round(),
        "净利润标准差": stddev.round(),
    });

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if input.get("base_margin_pct").is_none()
            || input.get("base_ad_pct").is_none()
            || input.get("base_refund_pct").is_none()
            || input.get("fixed_cost").is_none()
            || input.get("optimistic_growth").is_none()
            || input.get("pessimistic_growth").is_none() { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("情景财务分析完成，基准GMV{}，盈亏平衡GMV{}，亏损概率{}", currency(base_gmv), currency(bep_gmv), mc_summary["亏损概率"].as_str().unwrap_or("N/A")),
        "三情景P&L对比": scenarios,
        "盈亏平衡GMV": bep_gmv.round(),
        "敏感性矩阵（价格×流量）": sensitivity_matrix,
        "蒙特卡洛风险分析": mc_summary,
        "AI情景解读": "纯计算版本已完成三情景、敏感性和确定性风险模拟；未调用LLM。",
        "分析参数": {
            "基准毛利率": percent(base_margin * 100.0, 0),
            "广告费率": percent(base_ad * 100.0, 0),
            "退款率": percent(base_refund * 100.0, 0),
            "固定成本/月": fixed_cost.round(),
        },
    })
}
