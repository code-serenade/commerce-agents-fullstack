use serde_json::{Value, json};

use super::super::support::{
    currency, deferred_accounting, leak_advice, num, pct, percent, round2, string, wf,
};

pub(in crate::skills::business::accounting) fn pl_statement(input: &Value) -> Value {
    let gmv = num(input, "gmv", 0.0);
    if gmv <= 0.0 {
        return deferred_accounting(
            "accounting_pl_statement",
            "请传入 gmv；纯计算版本不会自动读取店铺数据。",
            input,
        );
    }
    let days = num(input, "days", 30.0).round();
    let ad_spend = num(input, "ad_spend", 0.0);
    let orders = num(input, "orders", 0.0);
    let refund_rate = num(input, "refund_rate", 3.0) / 100.0;
    let cogs_rate = num(input, "cogs_rate", 40.0) / 100.0;
    let variable_rate = num(input, "variable_cost_rate", 10.0) / 100.0;
    let tax_rate = num(input, "tax_rate", 25.0) / 100.0;
    let depreciation = num(input, "depreciation", 0.0);

    let refund = round2(gmv * refund_rate);
    let net_revenue = round2(gmv - refund);
    let cogs = round2(net_revenue * cogs_rate);
    let gross_profit = round2(net_revenue - cogs);
    let variable_costs = round2(net_revenue * variable_rate);
    let platform_fee = round2(gmv * 0.05);
    let contribution = round2(gross_profit - variable_costs - platform_fee - ad_spend);
    let fixed_costs = if input.get("fixed_costs").is_some() {
        num(input, "fixed_costs", 0.0)
    } else {
        round2(net_revenue * 0.03)
    };
    let ebitda = round2(contribution - fixed_costs);
    let ebit = round2(ebitda - depreciation);
    let tax = round2((ebit * tax_rate).max(0.0));
    let net_profit = round2(ebit - tax);
    let gross_margin = pct(gross_profit, net_revenue);
    let cm_rate = pct(contribution, net_revenue);
    let net_margin = pct(net_profit, net_revenue);
    let uses_defaults = input.get("refund_rate").is_none()
        || input.get("cogs_rate").is_none()
        || input.get("variable_cost_rate").is_none()
        || input.get("fixed_costs").is_none()
        || input.get("tax_rate").is_none()
        || input.get("depreciation").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("完整利润表已生成，净营收{}，税后净利润{}，净利率{}", currency(net_revenue), currency(net_profit), percent(net_margin, 2)),
        "统计周期": format!("最近{}天", days),
        "数据来源": "用户提供 + 参数估算",
        "利润表": {
            "① GMV（总销售额）": currency(gmv),
            "  (-) 退款": currency(refund),
            "② 净营收": currency(net_revenue),
            "  (-) 商品成本（COGS）": currency(cogs),
            "③ 毛利润": currency(gross_profit),
            "   毛利率": percent(gross_margin, 2),
            "  (-) 物流/包装（变动）": currency(variable_costs),
            "  (-) 平台佣金": currency(platform_fee),
            "  (-) 广告投放": currency(ad_spend),
            "④ 贡献利润（CM）": currency(contribution),
            "   贡献利润率": percent(cm_rate, 2),
            "  (-) 固定成本（人工/租金等）": currency(fixed_costs),
            "⑤ EBITDA": currency(ebitda),
            "  (-) 折旧摊销": currency(depreciation),
            "⑥ EBIT（税前利润）": currency(ebit),
            "  (-) 所得税": currency(tax),
            "⑦ 税后净利润": currency(net_profit),
            "   净利率": percent(net_margin, 2),
        },
        "关键运营指标": {
            "订单量": orders.round() as i64,
            "客单价(AOV)": if orders > 0.0 { json!(currency(gmv / orders)) } else { json!("N/A") },
            "退款率": percent(refund_rate * 100.0, 2),
            "广告ROI": if ad_spend > 0.0 { json!(round2(gmv / ad_spend)) } else { json!("无广告数据") },
        },
        "健康度评估": {
            "毛利率": if gross_margin > 50.0 { "优秀" } else if gross_margin > 30.0 { "正常" } else { "偏低" },
            "贡献利润率": if cm_rate > 25.0 { "优秀" } else if cm_rate > 10.0 { "正常" } else { "偏低" },
            "净利率": if net_margin > 15.0 { "优秀" } else if net_margin > 5.0 { "正常" } else if net_margin < 0.0 { "亏损" } else { "偏低" },
        },
    })
}

pub(in crate::skills::business::accounting) fn financial_narrative(input: &Value) -> Value {
    let days = num(input, "days", 30.0).round();
    let context = string(input, "context", "");
    let gmv = num(input, "gmv", 500_000.0).max(0.0);
    let ad_spend = num(input, "ad_spend", round2(gmv * 0.10));
    let orders = num(input, "orders", (gmv / 150.0).round());
    let net_revenue = round2(gmv * 0.97);
    let cogs = round2(net_revenue * 0.40);
    let gross_profit = round2(net_revenue - cogs);
    let contribution = round2(gross_profit - net_revenue * 0.10 - gmv * 0.05 - ad_spend);
    let net_profit = round2(contribution - net_revenue * 0.03);
    let gross_margin = pct(gross_profit, net_revenue);
    let cm_rate = pct(contribution, net_revenue);
    let net_margin = pct(net_profit, net_revenue);
    let ad_roi = if ad_spend > 0.0 {
        round2(gmv / ad_spend)
    } else {
        0.0
    };
    let rating = if net_margin >= 15.0 && ad_roi >= 4.0 {
        "A"
    } else if net_margin >= 5.0 && ad_roi >= 2.0 {
        "B"
    } else if net_margin >= 0.0 {
        "C"
    } else {
        "D"
    };
    let mut risks = Vec::new();
    if gross_margin < 30.0 {
        risks.push("毛利率偏低，需优化采购成本或价格带");
    }
    if ad_spend > 0.0 && ad_roi < 2.0 {
        risks.push("广告ROI偏低，投放效率拖累利润");
    }
    if net_margin < 5.0 {
        risks.push("净利率偏低，抗促销和退款波动能力弱");
    }
    let uses_defaults = input.get("gmv").is_none()
        || input.get("ad_spend").is_none()
        || input.get("orders").is_none();
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("财务健康诊断完成，评级{}，净利率{}，广告ROI{}", rating, percent(net_margin, 2), if ad_spend > 0.0 { ad_roi.to_string() } else { "无广告数据".to_owned() }),
        "统计周期": format!("近{}天", days),
        "数据来源": if input.get("gmv").is_some() { "用户提供 + 确定性诊断" } else { "默认样例 + 确定性诊断" },
        "背景": if context.is_empty() { "无额外背景" } else { context.as_str() },
        "关键财务指标": {
            "GMV": currency(gmv),
            "净营收": currency(net_revenue),
            "毛利率": percent(gross_margin, 2),
            "贡献利润率": percent(cm_rate, 2),
            "净利率": percent(net_margin, 2),
            "广告ROI": if ad_spend > 0.0 { json!(ad_roi) } else { json!("无广告数据") },
            "客单价": if orders > 0.0 { json!(currency(gmv / orders)) } else { json!("N/A") },
            "退款率": "3.00%",
        },
        "财务健康评级": rating,
        "诊断摘要": format!("当前评级为{rating}；毛利率{}，净利率{}，广告ROI{}。", percent(gross_margin, 1), percent(net_margin, 1), if ad_spend > 0.0 { ad_roi.to_string() } else { "无广告数据".to_owned() }),
        "主要风险": if risks.is_empty() { vec!["暂未发现重大财务风险"] } else { risks },
        "改善路径": [
            "优先跟踪GMV、毛利率、广告ROI、退款率四项日指标",
            "将广告预算向ROI前40%的计划集中，低效素材48小时内下线",
            "复盘高退款SKU并同步调整详情页承诺、客服话术和质检标准"
        ],
    })
}

pub(in crate::skills::business::accounting) fn gmv_waterfall(input: &Value) -> Value {
    let gmv = num(input, "gmv", 0.0);
    if gmv <= 0.0 {
        return deferred_accounting(
            "accounting_gmv_waterfall",
            "纯计算版本需要手动传入 gmv。",
            input,
        );
    }
    let ad_spend = num(input, "ad_spend", 0.0);
    let orders = num(input, "orders", 0.0);
    let refund_rate = num(input, "refund_rate", 3.0) / 100.0;
    let cogs_rate = num(input, "cogs_rate", 40.0) / 100.0;
    let logistics_rate = num(input, "logistics_rate", 8.0) / 100.0;
    let platform_fee_rate = num(input, "platform_fee_rate", 5.0) / 100.0;
    let tax_rate = num(input, "tax_rate", 3.0) / 100.0;

    let step1_gmv = round2(gmv);
    let step2_refund = round2(gmv * refund_rate);
    let step3_net_rev = round2(step1_gmv - step2_refund);
    let step4_cogs = round2(step3_net_rev * cogs_rate);
    let step5_gross_profit = round2(step3_net_rev - step4_cogs);
    let step6_logistics = round2(step3_net_rev * logistics_rate);
    let step7_platform_fee = round2(step1_gmv * platform_fee_rate);
    let step8_ad = round2(ad_spend);
    let step9_contribution =
        round2(step5_gross_profit - step6_logistics - step7_platform_fee - step8_ad);
    let step10_fixed = if input.get("fixed_opex").is_some() {
        num(input, "fixed_opex", 0.0)
    } else {
        round2(step3_net_rev * 0.03)
    };
    let step11_ebitda = round2(step9_contribution - step10_fixed);
    let step12_tax = round2((step11_ebitda * tax_rate).max(0.0));
    let step13_net_profit = round2(step11_ebitda - step12_tax);

    let waterfall = vec![
        wf("① GMV（总销售额）", step1_gmv, step1_gmv, "基准", step1_gmv),
        wf(
            "  (-) 退款/退货",
            -step2_refund,
            step1_gmv,
            "扣减",
            step1_gmv,
        ),
        wf("② 净营收", step3_net_rev, step1_gmv, "小计", step1_gmv),
        wf(
            "  (-) 商品成本(COGS)",
            -step4_cogs,
            step1_gmv,
            "扣减",
            step1_gmv,
        ),
        wf("③ 毛利润", step5_gross_profit, step1_gmv, "小计", step1_gmv),
        wf(
            "  (-) 物流/包装费",
            -step6_logistics,
            step1_gmv,
            "扣减",
            step1_gmv,
        ),
        wf(
            "  (-) 平台佣金",
            -step7_platform_fee,
            step1_gmv,
            "扣减",
            step1_gmv,
        ),
        wf("  (-) 广告投放费", -step8_ad, step1_gmv, "扣减", step1_gmv),
        wf(
            "④ 贡献利润(CM)",
            step9_contribution,
            step1_gmv,
            "小计",
            step1_gmv,
        ),
        wf(
            "  (-) 固定运营成本",
            -step10_fixed,
            step1_gmv,
            "扣减",
            step1_gmv,
        ),
        wf("⑤ EBITDA", step11_ebitda, step1_gmv, "小计", step1_gmv),
        wf("  (-) 税费", -step12_tax, step1_gmv, "扣减", step1_gmv),
        wf(
            "⑥ 税后净利润",
            step13_net_profit,
            step1_gmv,
            "终值",
            step1_gmv,
        ),
    ];

    let mut deductions = [
        ("退款", step2_refund),
        ("商品成本", step4_cogs),
        ("物流", step6_logistics),
        ("平台佣金", step7_platform_fee),
        ("广告", step8_ad),
        ("固定成本", step10_fixed),
    ];
    deductions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top_leak = deductions[0];
    let net_margin = pct(step13_net_profit, step1_gmv);
    let gross_margin = pct(step5_gross_profit, step1_gmv);
    let cm_margin = pct(step9_contribution, step1_gmv);
    let uses_defaults = input.get("refund_rate").is_none()
        || input.get("cogs_rate").is_none()
        || input.get("logistics_rate").is_none()
        || input.get("platform_fee_rate").is_none()
        || input.get("fixed_opex").is_none()
        || input.get("tax_rate").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("GMV利润瀑布分析完成，税后净利润{}，净利率{}，最大漏损项{}", currency(step13_net_profit), percent(net_margin, 2), top_leak.0),
        "数据来源": "用户提供 + 参数估算",
        "GMV利润瀑布": waterfall,
        "核心利润率": {
            "毛利率": percent(gross_margin, 2),
            "贡献利润率": percent(cm_margin, 2),
            "净利率": percent(net_margin, 2),
            "广告ROI": if ad_spend > 0.0 { json!(round2(gmv / ad_spend)) } else { json!("无广告数据") },
            "客单价": if orders > 0.0 { json!(currency(gmv / orders)) } else { json!("N/A") },
        },
        "健康度诊断": {
            "净利率": percent(net_margin, 2),
            "评级": if net_margin >= 15.0 { "优秀" } else if net_margin >= 5.0 { "正常" } else if net_margin < 0.0 { "亏损" } else { "偏低" },
            "最大漏损项": format!("{}（占GMV {}）", top_leak.0, percent(pct(top_leak.1, step1_gmv), 1)),
            "改善建议": leak_advice(top_leak.0, top_leak.1, step1_gmv),
        }
    })
}
