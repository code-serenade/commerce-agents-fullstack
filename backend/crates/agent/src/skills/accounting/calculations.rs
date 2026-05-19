use serde_json::{Value, json};

use super::support::{
    average, calc_scenario, currency, deferred_accounting, deterministic_risk_samples, leak_advice,
    num, nums, pct, percent, percentile, round2, signed_percent, string, variance, wf,
};

pub(super) fn cost_calc(input: &Value) -> Value {
    let purchase = num(input, "purchase_cost", 0.0);
    let shipping = num(input, "shipping_cost", 5.0);
    let packaging = num(input, "packaging_cost", 2.0);
    let fee_rate = num(input, "platform_fee_rate", 5.0);
    let ad_cost = num(input, "ad_cost_per_unit", 3.0);
    let price = num(input, "selling_price", 100.0);
    let platform_fee = round2(price * fee_rate / 100.0);
    let total_cost = round2(purchase + shipping + packaging + platform_fee + ad_cost);
    let profit = round2(price - total_cost);
    let margin = pct(profit, price);
    let uses_defaults = input.get("shipping_cost").is_none()
        || input.get("packaging_cost").is_none()
        || input.get("platform_fee_rate").is_none()
        || input.get("ad_cost_per_unit").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("成本核算完成，综合成本{}，单品利润{}，利润率{}", currency(total_cost), currency(profit), percent(margin, 2)),
        "售价": price,
        "成本明细": {
            "采购成本": purchase,
            "物流成本": shipping,
            "包装成本": packaging,
            "平台佣金": platform_fee,
            "推广成本": ad_cost,
        },
        "综合成本": total_cost,
        "单品利润": profit,
        "利润率": percent(margin, 2),
        "成本占比": {
            "采购": percent(pct(purchase, total_cost), 1),
            "物流": percent(pct(shipping, total_cost), 1),
            "包装": percent(pct(packaging, total_cost), 1),
            "平台": percent(pct(platform_fee, total_cost), 1),
            "推广": percent(pct(ad_cost, total_cost), 1),
        },
        "建议": if margin < 15.0 { "利润率偏低，建议优化采购或提价" } else { "利润率健康" },
    })
}

pub(super) fn profit_analysis(input: &Value) -> Value {
    let days = num(input, "days", 30.0).round();
    let revenue = num(input, "revenue", 0.0);
    if revenue <= 0.0 {
        return deferred_accounting(
            "accounting_profit_analysis",
            "纯计算版本需要手动传入 revenue 参数。",
            input,
        );
    }
    let refund = num(input, "refund_amount", 0.0);
    let cogs = if input.get("cogs").is_some() {
        num(input, "cogs", 0.0)
    } else {
        round2(revenue * 0.40)
    };
    let opex = if input.get("operating_expenses").is_some() {
        num(input, "operating_expenses", 0.0)
    } else {
        round2(revenue * 0.10)
    };
    let ad = num(input, "ad_spend", 0.0);
    let net_revenue = round2(revenue - refund);
    let gross_profit = round2(net_revenue - cogs);
    let gross_margin = pct(gross_profit, net_revenue);
    let operating_profit = round2(gross_profit - opex - ad);
    let operating_margin = pct(operating_profit, net_revenue);
    let uses_defaults = input.get("cogs").is_none() || input.get("operating_expenses").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("利润分析完成，净营收{}，营业利润{}，营业利润率{}", currency(net_revenue), currency(operating_profit), percent(operating_margin, 2)),
        "统计周期": format!("最近{}天", days),
        "数据来源": "用户提供",
        "营收分析": {
            "总营收": revenue,
            "退款": refund,
            "净营收": net_revenue,
            "退款率": percent(pct(refund, revenue), 2),
        },
        "利润分析": {
            "毛利润": gross_profit,
            "毛利率": percent(gross_margin, 2),
            "营业利润": operating_profit,
            "营业利润率": percent(operating_margin, 2),
        },
        "费用结构": {
            "商品成本": cogs,
            "运营费用": opex,
            "广告支出": ad,
            "费用率": percent(pct(cogs + opex + ad, net_revenue), 2),
        },
        "健康度": {
            "毛利率": if gross_margin > 40.0 { "健康" } else if gross_margin > 20.0 { "偏低" } else { "亏损风险" },
            "营业利润率": if operating_margin > 10.0 { "健康" } else if operating_margin > 0.0 { "偏低" } else { "亏损" },
            "广告占比": if pct(ad, net_revenue) < 15.0 { "合理" } else { "偏高" },
        }
    })
}

pub(super) fn roi_calc(input: &Value) -> Value {
    let period = string(input, "period", "最近30天");
    let mut investment = num(input, "investment", 0.0);
    let revenue = num(input, "revenue_generated", 0.0);
    if investment <= 0.0 && revenue <= 0.0 {
        return deferred_accounting(
            "accounting_roi_calc",
            "纯计算版本需要手动传入 investment 和 revenue_generated。",
            input,
        );
    }
    if investment <= 0.0 {
        investment = 1.0;
    }
    let cogs = if input.get("cost_of_goods").is_some() {
        num(input, "cost_of_goods", 0.0)
    } else {
        round2(revenue * 0.40)
    };
    let aov = num(input, "avg_order_value", 150.0).max(0.01);
    let roas = round2(revenue / investment);
    let profit_roi = round2((revenue - cogs - investment) / investment);
    let acos = pct(investment, revenue);
    let cpa = if revenue > 0.0 {
        round2(investment / (revenue / aov))
    } else {
        0.0
    };
    let net_profit = round2(revenue - cogs - investment);
    let uses_defaults =
        input.get("cost_of_goods").is_none() || input.get("avg_order_value").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("ROI计算完成，ROAS {}，净利润{}，利润ROI {}", roas, currency(net_profit), profit_roi),
        "统计周期": period,
        "数据来源": "用户提供",
        "投入": investment,
        "产出营收": revenue,
        "ROI指标": {
            "ROAS(广告回报率)": roas,
            "真实广告ROI": roas,
            "利润ROI": profit_roi,
            "ACOS(广告成本占比)": percent(acos, 2),
            "CPA(获客成本)": cpa,
        },
        "盈亏分析": {
            "毛利润": round2(revenue - cogs),
            "净利润": net_profit,
            "是否盈利": if net_profit > 0.0 { "是" } else { "否" },
        },
        "评价": {
            "ROAS": if roas > 5.0 { "优秀" } else if roas > 3.0 { "良好" } else if roas > 1.0 { "及格" } else { "亏损" },
            "建议": if roas > 5.0 { "加大投放" } else if roas > 3.0 { "维持投放" } else if roas > 1.0 { "优化素材" } else { "暂停投放排查" },
        }
    })
}

pub(super) fn budget_plan(input: &Value) -> Value {
    let month = string(input, "month", "下月");
    let growth_rate = num(input, "growth_rate", 20.0);
    let target_gmv = num(input, "target_gmv", 500_000.0).max(0.0);
    let target_margin = num(input, "target_margin", 15.0).max(0.0);
    let target_profit = round2(target_gmv * target_margin / 100.0);
    let items = [
        ("采购成本", 0.40),
        ("物流费用", 0.08),
        ("平台佣金", 0.05),
        ("广告投放", 0.12),
        ("人力成本", 0.10),
        ("包装耗材", 0.03),
        ("其他费用", 0.02),
    ];
    let budget_items: Vec<_> = items
        .iter()
        .map(|(name, ratio)| {
            json!({
                "科目": name,
                "预算金额": currency(target_gmv * ratio),
                "占GMV": percent(ratio * 100.0, 1),
            })
        })
        .collect();
    let total_cost_rate: f64 = items.iter().map(|(_, ratio)| ratio).sum();
    let total_budget = round2(target_gmv * total_cost_rate);
    let planned_profit = round2(target_gmv - total_budget);
    let uses_defaults = input.get("target_gmv").is_none()
        || input.get("target_margin").is_none()
        || input.get("growth_rate").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("预算编制完成，目标GMV{}，计划利润{}", currency(target_gmv), currency(planned_profit)),
        "数据来源": if input.get("target_gmv").is_some() { "用户指定目标" } else { "默认目标（纯计算版本未读取历史数据）" },
        "预算月份": month,
        "目标增长率": signed_percent(growth_rate, 1),
        "目标GMV": currency(target_gmv),
        "目标利润": currency(target_profit),
        "预算明细": budget_items,
        "预算汇总": {
            "总预算成本": currency(total_budget),
            "计划利润": currency(planned_profit),
            "计划利润率": percent(pct(planned_profit, target_gmv), 1),
            "目标达成": if planned_profit >= target_profit { "满足目标利润率" } else { "低于目标利润率，需压缩费用或提高毛利" },
        },
    })
}

pub(super) fn break_even_calc(input: &Value) -> Value {
    let price = num(input, "selling_price", 100.0);
    let unit_var = num(input, "unit_variable_cost", round2(price * 0.55));
    let fixed = num(input, "fixed_costs", round2(price * 200.0 * 0.13));
    let contribution_per_unit = price - unit_var;
    if contribution_per_unit <= 0.0 {
        return deferred_accounting(
            "accounting_break_even_calc",
            "单品变动成本高于售价，无法达到盈亏平衡；请降低成本或提高售价后重新计算。",
            input,
        );
    }

    let bep_units = (fixed / contribution_per_unit).round();
    let bep_revenue = round2(bep_units * price);
    let margin_of_safety_pct = if price > unit_var {
        (1.0 - 1.0 / (price / unit_var)) * 100.0
    } else {
        0.0
    };
    let price_variants = [0.8, 0.9, 1.0, 1.1, 1.2].map(|r| round2(price * r));
    let sales_targets = [0.5, 0.75, 1.0, 1.5, 2.0].map(|r| (bep_units * r).round());
    let sensitivity: Vec<_> = price_variants
        .iter()
        .map(|pv| {
            let mut row = serde_json::Map::new();
            row.insert("售价".to_owned(), json!(currency(*pv)));
            for sv in sales_targets {
                let profit = round2((pv - unit_var) * sv - fixed);
                row.insert(
                    format!("{}单", sv as i64),
                    json!(format!(
                        "{}{}",
                        currency(profit),
                        if profit >= 0.0 { "✓" } else { "✗" }
                    )),
                );
            }
            Value::Object(row)
        })
        .collect();
    let uses_defaults = input.get("selling_price").is_none()
        || input.get("unit_variable_cost").is_none()
        || input.get("fixed_costs").is_none();

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("盈亏平衡分析完成，需销售{}单，需实现营收{}，贡献利润率{}", bep_units as i64, currency(bep_revenue), percent(pct(contribution_per_unit, price), 1)),
        "数据来源": "用户提供/默认参数",
        "定价": price,
        "单品变动成本": unit_var,
        "月固定成本": fixed,
        "单位贡献利润": round2(contribution_per_unit),
        "贡献利润率": percent(pct(contribution_per_unit, price), 1),
        "盈亏平衡点": {
            "需销售订单数": bep_units as i64,
            "需实现营收": currency(bep_revenue),
            "安全边际": percent(margin_of_safety_pct, 1),
        },
        "敏感性分析(利润矩阵)": sensitivity,
        "说明": "✓=盈利 ✗=亏损",
    })
}

pub(super) fn compliance_check(input: &Value) -> Value {
    let revenue = num(input, "monthly_revenue", 0.0);
    if revenue <= 0.0 {
        return deferred_accounting(
            "accounting_compliance_check",
            "请传入 monthly_revenue；纯计算版本不会自动读取店铺营收。",
            input,
        );
    }
    let tax_type = string(input, "tax_type", "小规模");
    let invoice_rate = num(input, "invoice_rate", 80.0);
    let cash_ratio = num(input, "cash_ratio", 10.0);
    let mut risks = Vec::new();
    if tax_type == "小规模" && revenue * 12.0 > 5_000_000.0 {
        risks.push(json!({"风险": "年化营收超500万需转为一般纳税人", "等级": "高", "建议": "提前完成纳税人身份切换和税负测算"}));
    }
    if invoice_rate < 70.0 {
        risks.push(
            json!({"风险": "开票率偏低", "等级": "中", "建议": "规范开票流程，补齐交易凭证"}),
        );
    }
    if cash_ratio > 30.0 {
        risks.push(
            json!({"风险": "现金收款占比过高", "等级": "中", "建议": "提高线上支付占比并保留流水"}),
        );
    }
    let vat_rate = if tax_type == "一般纳税人" {
        0.13
    } else {
        0.03
    };
    let vat = round2(revenue * vat_rate);
    let surcharge = round2(vat * 0.12);
    let income_tax = round2(revenue * 0.05 * 0.25);
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if input.get("invoice_rate").is_none() || input.get("cash_ratio").is_none() { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("合规检查完成，预估税额合计{}，发现{}个风险点", currency(vat + surcharge + income_tax), risks.len()),
        "数据来源": "用户提供",
        "月营收": currency(revenue),
        "纳税类型": tax_type,
        "预估税额": {
            "增值税": currency(vat),
            "附加税": currency(surcharge),
            "企业所得税": currency(income_tax),
            "合计": currency(vat + surcharge + income_tax),
        },
        "合规检查": {
            "开票率": percent(invoice_rate, 1),
            "现金占比": percent(cash_ratio, 1),
            "风险点数": risks.len(),
        },
        "风险详情": if risks.is_empty() { vec![json!({"提示": "暂未发现合规风险"})] } else { risks },
        "建议": ["按月申报纳税", "保留交易凭证和平台流水", "每月完成收入、发票、回款三方核对"],
    })
}

pub(super) fn pl_statement(input: &Value) -> Value {
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

pub(super) fn budget_vs_actual(input: &Value) -> Value {
    let actual_gmv = num(input, "actual_gmv", 0.0);
    if actual_gmv <= 0.0 {
        return deferred_accounting(
            "accounting_budget_vs_actual",
            "纯计算版本需要手动传入 actual_gmv；budget_gmv 不传时会按 actual_gmv×120%生成。",
            input,
        );
    }
    let budget_gmv = num(input, "budget_gmv", round2(actual_gmv * 1.20));
    let actual_ad = num(input, "actual_ad_spend", 0.0);
    let budget_ad = num(input, "budget_ad_spend", round2(budget_gmv * 0.12));
    let budget_cogs_rate = num(input, "budget_cogs_rate", 40.0) / 100.0;
    let actual_cogs_rate = num(input, "actual_cogs_rate", 40.0) / 100.0;
    let budget_opex_rate = num(input, "budget_opex_rate", 10.0) / 100.0;
    let actual_opex_rate = num(input, "actual_opex_rate", 10.0) / 100.0;
    let actual_refund_rate = num(input, "actual_refund_rate", 3.0) / 100.0;

    let budget_net_rev = round2(budget_gmv * 0.97);
    let budget_cogs = round2(budget_gmv * budget_cogs_rate);
    let budget_opex = round2(budget_gmv * budget_opex_rate);
    let budget_gross_profit = round2(budget_net_rev - budget_cogs);
    let budget_op_profit = round2(budget_gross_profit - budget_opex - budget_ad);
    let budget_margin = pct(budget_op_profit, budget_gmv);

    let actual_net_rev = round2(actual_gmv * (1.0 - actual_refund_rate));
    let actual_cogs = round2(actual_net_rev * actual_cogs_rate);
    let actual_opex = round2(actual_gmv * actual_opex_rate);
    let actual_gross_profit = round2(actual_net_rev - actual_cogs);
    let actual_op_profit = round2(actual_gross_profit - actual_opex - actual_ad);
    let actual_margin = pct(actual_op_profit, actual_gmv);

    let variance_table = json!({
        "GMV": variance(budget_gmv, actual_gmv, true),
        "净营收": variance(budget_net_rev, actual_net_rev, true),
        "商品成本": variance(budget_cogs, actual_cogs, false),
        "广告投入": variance(budget_ad, actual_ad, false),
        "运营费用": variance(budget_opex, actual_opex, false),
        "毛利润": variance(budget_gross_profit, actual_gross_profit, true),
        "营业利润": variance(budget_op_profit, actual_op_profit, true),
        "利润率": {
            "预算": percent(budget_margin, 2),
            "实际": percent(actual_margin, 2),
            "差异": format!("{:+.1}ppt", actual_margin - budget_margin),
            "状态": if actual_margin >= budget_margin { "有利" } else { "不利" },
        },
    });

    let gmv_ach = actual_gmv / budget_gmv.max(1.0);
    let margin_ach = actual_margin / budget_margin.max(0.1);
    let ad_eff = if actual_ad > 0.0 && budget_ad > 0.0 {
        budget_ad / actual_ad
    } else {
        1.0
    };
    let exec_score =
        ((gmv_ach * 0.50 + margin_ach * 0.35 + ad_eff.min(1.5) * 0.15) * 100.0).min(100.0);
    let mut risks = Vec::new();
    if actual_gmv < budget_gmv * 0.90 {
        risks.push(format!(
            "GMV未达标（完成率{:.0}%，差{}元）",
            gmv_ach * 100.0,
            round2(budget_gmv - actual_gmv)
        ));
    }
    if budget_ad > 0.0 && actual_ad > budget_ad * 1.15 {
        risks.push(format!(
            "广告超支（超出{}元）",
            round2(actual_ad - budget_ad)
        ));
    }
    if actual_margin < budget_margin * 0.85 {
        risks.push(format!(
            "利润率低于预算（实{} vs 预算{}）",
            percent(actual_margin, 2),
            percent(budget_margin, 2)
        ));
    }

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if input.get("budget_gmv").is_none()
            || input.get("budget_ad_spend").is_none()
            || input.get("budget_cogs_rate").is_none()
            || input.get("actual_cogs_rate").is_none()
            || input.get("budget_opex_rate").is_none()
            || input.get("actual_opex_rate").is_none()
            || input.get("actual_refund_rate").is_none() { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("预算vs实际分析完成，GMV完成率{}，执行质量评分{:.1}/100", percent(gmv_ach * 100.0, 1), exec_score),
        "数据说明": if input.get("budget_gmv").is_some() { "用户提供预算" } else { "预算自动生成：actual_gmv×120%" },
        "执行质量评分": format!("{:.1}/100", exec_score),
        "GMV完成率": percent(gmv_ach * 100.0, 1),
        "预算vs实际对比": variance_table,
        "风险项": if risks.is_empty() { vec!["各科目执行符合预算，无重大风险".to_owned()] } else { risks },
    })
}

pub(super) fn cash_flow_forecast(input: &Value) -> Value {
    let days = num(input, "days", 30.0).max(1.0);
    let base_gmv = num(input, "base_gmv", 500_000.0).max(0.0);
    let base_ad = num(input, "base_ad_spend", round2(base_gmv * 0.10));
    let capex = num(input, "capex", 0.0);
    let loan_repayment = num(input, "loan_repayment", 0.0);
    let opening_cash = num(input, "opening_cash", 0.0);
    let mut growth = nums(input, "growth_assumptions");
    if growth.len() < 3 {
        growth.extend([5.0, 8.0, 12.0]);
        growth.truncate(3);
    }
    let factor = 30.0 / days;
    let monthly_gmv = base_gmv * factor;
    let monthly_ad = base_ad * factor;
    let labels = ["第1个月", "第2个月", "第3个月"];
    let mut running_cash = opening_cash;
    let mut forecast = Vec::new();
    let mut total_net_cf = 0.0;
    for (idx, growth_pct) in growth.iter().take(3).enumerate() {
        let multiplier = (1.0 + growth_pct / 100.0).powi((idx + 1) as i32);
        let gmv = round2(monthly_gmv * multiplier);
        let ad = round2(monthly_ad * multiplier);
        let refund = round2(gmv * 0.03);
        let cash_in = round2(gmv - refund);
        let cogs = round2(gmv * 0.40);
        let logistics = round2(gmv * 0.08);
        let platform = round2(gmv * 0.05);
        let opex = round2(cash_in * 0.10);
        let tax = round2(cash_in * 0.03);
        let operating_cf = round2(cash_in - cogs - logistics - platform - ad - opex - tax);
        let investing_cf = round2(-capex);
        let financing_cf = round2(-loan_repayment);
        let net_cf = round2(operating_cf + investing_cf + financing_cf);
        running_cash = round2(running_cash + net_cf);
        total_net_cf += net_cf;
        forecast.push(json!({
            "月份": labels[idx],
            "GMV预测": currency(gmv),
            "增长假设": signed_percent(*growth_pct, 1),
            "经营现金流入": currency(cash_in),
            "经营现金流出明细": {
                "商品成本": currency(cogs),
                "物流费": currency(logistics),
                "平台佣金": currency(platform),
                "广告投放": currency(ad),
                "运营费用": currency(opex),
                "税费": currency(tax),
            },
            "经营现金流净额": currency(operating_cf),
            "投资现金流": currency(investing_cf),
            "筹资现金流": currency(financing_cf),
            "月净现金流": currency(net_cf),
            "期末现金余额": currency(running_cash),
            "经营现金流状态": if operating_cf > 0.0 { "正常" } else { "预警" },
        }));
    }
    let risk_tips: Vec<String> = forecast
        .iter()
        .filter(|m| m["经营现金流状态"] == "预警")
        .map(|m| {
            format!(
                "{}经营现金流为负，需关注资金缺口",
                m["月份"].as_str().unwrap_or("")
            )
        })
        .collect();
    let uses_defaults = input.get("base_gmv").is_none()
        || input.get("base_ad_spend").is_none()
        || input.get("growth_assumptions").is_none()
        || input.get("capex").is_none()
        || input.get("loan_repayment").is_none()
        || input.get("opening_cash").is_none();
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if uses_defaults { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("3个月现金流预测完成，累计净现金流{}，期末现金余额{}", currency(total_net_cf), currency(running_cash)),
        "数据来源": if input.get("base_gmv").is_some() { "用户提供基期数据 + 增长假设" } else { "默认基期数据 + 增长假设" },
        "基期月GMV": currency(monthly_gmv),
        "期初现金余额": currency(opening_cash),
        "3个月现金流预测": forecast,
        "3个月累计净现金流": currency(total_net_cf),
        "预测说明": "基于固定成本率和增长假设的确定性滚动预测；未调用外部数据或LLM。",
        "风险提示": if risk_tips.is_empty() { vec!["未发现明显资金缺口".to_owned()] } else { risk_tips },
    })
}

pub(super) fn financial_narrative(input: &Value) -> Value {
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

pub(super) fn gmv_waterfall(input: &Value) -> Value {
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

pub(super) fn scenario_analysis(input: &Value) -> Value {
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
