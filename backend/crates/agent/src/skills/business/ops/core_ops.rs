use serde_json::{Value, json};

use super::common::{
    average, commission_rate, currency, listing_keywords, num, nums, pct, percent, promo_phase,
    round1, round2, string, strings, z_score,
};

pub(super) fn product_opportunity_score(input: &Value) -> Value {
    let product_name = string(input, "product_name", "");
    let category = string(input, "category", "通用商品");
    let platform = string(input, "platform", "淘宝");
    let product_ref = string(input, "product_ref", "");
    let target_audience = string(input, "target_audience", "待验证人群");
    let mut selling_points = strings(input, "selling_points");
    if selling_points.is_empty() {
        selling_points = strings(input, "features");
    }
    let competitor_prices = nums(input, "competitor_prices");
    let risk_signals = strings(input, "risk_signals");
    let budget = num(input, "budget", 0.0).max(0.0);
    let cost = num(input, "cost", 0.0).max(0.0);
    let price = num(input, "price", 0.0).max(0.0);
    let current_monthly_sales = num(input, "current_monthly_sales", 0.0).max(0.0);
    let search_demand_score = num(input, "search_demand_score", 0.0).clamp(0.0, 100.0);
    let review_pain_score = num(input, "review_pain_score", 0.0).clamp(0.0, 100.0);
    let supply_score = num(input, "supply_score", 0.0).clamp(0.0, 100.0);

    if product_name.is_empty()
        && product_ref.is_empty()
        && selling_points.is_empty()
        && category == "通用商品"
    {
        return json!({
            "has_data": false,
            "提示": "请至少提供 product_name、product_ref、category 或 selling_points；机会评分不会自动读取商品库或外部市场数据。",
            "示例参数": {
                "product_name": "低糖冷萃咖啡",
                "category": "饮品",
                "price": 19.9,
                "cost": 8.0,
                "competitor_prices": [16.9, 19.9, 22.9],
                "selling_points": ["低糖", "即饮", "校园便利"],
                "budget": 3000
            }
        });
    }

    let display_product = if !product_name.is_empty() {
        product_name.clone()
    } else if !product_ref.is_empty() {
        product_ref.clone()
    } else {
        category.clone()
    };
    let gross_margin_score = if cost > 0.0 && price > 0.0 {
        let margin = pct(price - cost - price * commission_rate(&platform), price);
        if margin >= 45.0 {
            90.0
        } else if margin >= 30.0 {
            78.0
        } else if margin >= 18.0 {
            62.0
        } else if margin >= 8.0 {
            45.0
        } else {
            28.0
        }
    } else {
        55.0
    };
    let competitor_score = competitor_opportunity_score(price, &competitor_prices);
    let demand_score = if search_demand_score > 0.0 {
        search_demand_score
    } else {
        heuristic_demand_score(&category, &selling_points, current_monthly_sales)
    };
    let differentiation_score = differentiation_score(&selling_points, review_pain_score);
    let feasibility_score = feasibility_score(budget, supply_score, &risk_signals);
    let risk_score = risk_penalty_score(&risk_signals);
    let total_score = round1(
        demand_score * 0.24
            + gross_margin_score * 0.20
            + differentiation_score * 0.18
            + competitor_score * 0.16
            + feasibility_score * 0.14
            + risk_score * 0.08,
    )
    .clamp(0.0, 100.0);

    let decision = if total_score >= 78.0 {
        "优先测试"
    } else if total_score >= 62.0 {
        "小预算验证"
    } else if total_score >= 45.0 {
        "补数后再判断"
    } else {
        "暂缓"
    };
    let avg_comp = if competitor_prices.is_empty() {
        None
    } else {
        Some(round2(average(&competitor_prices)))
    };
    let suggested_test_budget = if budget > 0.0 {
        budget.clamp(500.0, 5_000.0)
    } else if total_score >= 78.0 {
        3_000.0
    } else if total_score >= 62.0 {
        1_500.0
    } else {
        500.0
    };

    json!({
        "has_data": true,
        "数据来源": "用户输入 + 确定性机会评分模型",
        "商品": display_product,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "类目": category,
        "平台": platform,
        "目标人群": target_audience,
        "机会总分": total_score,
        "机会等级": opportunity_grade(total_score),
        "建议决策": decision,
        "分项评分": [
            {"维度": "需求信号", "分数": round1(demand_score), "依据": demand_reason(search_demand_score, current_monthly_sales)},
            {"维度": "毛利空间", "分数": round1(gross_margin_score), "依据": margin_reason(cost, price, &platform)},
            {"维度": "差异化", "分数": round1(differentiation_score), "依据": differentiation_reason(&selling_points, review_pain_score)},
            {"维度": "竞争位置", "分数": round1(competitor_score), "依据": competitor_reason(price, avg_comp)},
            {"维度": "验证可行性", "分数": round1(feasibility_score), "依据": feasibility_reason(budget, supply_score)},
            {"维度": "风险控制", "分数": round1(risk_score), "依据": risk_reason(&risk_signals)}
        ],
        "关键机会": opportunity_points(&selling_points, demand_score, gross_margin_score, competitor_score),
        "主要风险": risk_points(&risk_signals, cost, price, &competitor_prices),
        "MVP验证动作": [
            {"步骤": "确认卖点假设", "动作": "用3条核心卖点做标题/主图/短视频脚本A-B测试", "验收指标": "CTR或点击率高于基线10%以上"},
            {"步骤": "小预算测品", "动作": format!("先用{}以内预算跑2-3天轻量测试", currency(suggested_test_budget)), "验收指标": "加购率、询单率、成交转化至少有一项达标"},
            {"步骤": "复盘去留", "动作": "按毛利、转化、退款/差评风险决定放量、改款或暂停", "验收指标": "预计单品毛利覆盖获客成本且无高频履约风险"}
        ],
        "需要补充的数据": missing_opportunity_inputs(cost, price, &competitor_prices, search_demand_score, review_pain_score, &risk_signals),
        "后续适配 Skill": [
            "product_market_fit_diagnosis",
            "experiment_mvp_plan",
            "ops_smart_pricing",
            "search_market_info",
            "search_product_reviews"
        ],
        "治理说明": "该评分只基于本轮显式输入和确定性规则；没有联网搜索、没有读取实时平台后台，也不会把假设数据写成市场事实。"
    })
}

pub(super) fn product_market_fit_diagnosis(input: &Value) -> Value {
    let product_name = string(input, "product_name", "");
    let category = string(input, "category", "通用商品");
    let platform = string(input, "platform", "淘宝");
    let product_ref = string(input, "product_ref", "");
    let target_audience = string(input, "target_audience", "待验证人群");
    let mut selling_points = strings(input, "selling_points");
    if selling_points.is_empty() {
        selling_points = strings(input, "features");
    }
    let audience_segments = strings(input, "audience_segments");
    let mut channels = strings(input, "channels");
    if channels.is_empty() {
        channels = strings(input, "channel_candidates");
    }
    let use_scenarios = strings(input, "use_scenarios");
    let pain_points = strings(input, "pain_points");
    let risk_signals = strings(input, "risk_signals");
    let competitor_prices = nums(input, "competitor_prices");
    let price = num(input, "price", 0.0).max(0.0);
    let opportunity_score = num(input, "opportunity_score", 0.0).clamp(0.0, 100.0);
    let review_pain_score = num(input, "review_pain_score", 0.0).clamp(0.0, 100.0);

    if product_name.is_empty()
        && product_ref.is_empty()
        && selling_points.is_empty()
        && category == "通用商品"
        && target_audience == "待验证人群"
    {
        return json!({
            "has_data": false,
            "提示": "请至少提供商品、类目、目标人群、卖点或使用场景；PMF诊断不会自动读取商品库、用户画像或外部市场数据。",
            "示例参数": {
                "product_name": "低糖冷萃咖啡",
                "category": "饮品",
                "target_audience": "校园学生和通勤人群",
                "selling_points": ["低糖", "即饮", "便携"],
                "channels": ["校园社群", "抖音短视频", "小红书种草"],
                "use_scenarios": ["早八提神", "通勤路上", "健身后控糖"]
            }
        });
    }

    let display_product = if !product_name.is_empty() {
        product_name.clone()
    } else if !product_ref.is_empty() {
        product_ref.clone()
    } else {
        category.clone()
    };
    let resolved_audiences = resolved_audience_segments(&target_audience, &audience_segments);
    let resolved_channels = resolved_channel_candidates(&category, &platform, &channels);
    let audience_score = audience_fit_score(
        &category,
        &target_audience,
        &resolved_audiences,
        &selling_points,
        &pain_points,
        review_pain_score,
    );
    let value_score = differentiation_score(&selling_points, review_pain_score);
    let scenario_score = scenario_fit_score(&category, &use_scenarios, &selling_points);
    let channel_score =
        channel_fit_score(&category, &platform, &resolved_channels, &selling_points);
    let price_score = price_fit_score(price, &competitor_prices, opportunity_score);
    let trust_score = trust_fit_score(&risk_signals, &pain_points);
    let total_score = round1(
        audience_score * 0.24
            + value_score * 0.20
            + channel_score * 0.18
            + scenario_score * 0.14
            + price_score * 0.12
            + trust_score * 0.12,
    )
    .clamp(0.0, 100.0);

    json!({
        "has_data": true,
        "数据来源": "用户输入 + 确定性PMF诊断模型",
        "商品": display_product,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "类目": category,
        "平台": platform,
        "目标人群": target_audience,
        "PMF匹配总分": total_score,
        "PMF阶段判断": market_fit_stage(total_score),
        "建议动作": market_fit_action(total_score),
        "分项诊断": [
            {"维度": "商品-人群匹配", "分数": round1(audience_score), "判断": audience_fit_reason(&resolved_audiences, &pain_points, review_pain_score)},
            {"维度": "价值主张匹配", "分数": round1(value_score), "判断": value_fit_reason(&selling_points)},
            {"维度": "渠道匹配", "分数": round1(channel_score), "判断": channel_fit_reason(&resolved_channels, &category)},
            {"维度": "使用场景匹配", "分数": round1(scenario_score), "判断": scenario_fit_reason(&use_scenarios, &selling_points)},
            {"维度": "价格带匹配", "分数": round1(price_score), "判断": price_fit_reason(price, &competitor_prices, opportunity_score)},
            {"维度": "信任和风险匹配", "分数": round1(trust_score), "判断": trust_fit_reason(&risk_signals, &pain_points)}
        ],
        "优先人群": audience_priority(&resolved_audiences, &pain_points),
        "优先渠道": channel_priority(&resolved_channels, &category),
        "卖点-人群-渠道矩阵": market_fit_matrix(&selling_points, &resolved_audiences, &resolved_channels),
        "不匹配风险": market_fit_risks(&target_audience, &selling_points, &resolved_channels, &use_scenarios, price, &competitor_prices, &risk_signals),
        "验证问题": market_fit_validation_questions(&resolved_audiences, &resolved_channels, &selling_points),
        "MVP验证建议": [
            {"验证对象": "人群", "动作": "用同一卖点分别面向2-3个候选人群投放或社群测试", "通过标准": "至少一个人群点击率/询单率明显高于其它人群"},
            {"验证对象": "渠道", "动作": "同一商品素材在内容种草、搜索承接和私域/社群中做小样本测试", "通过标准": "找到一个获客成本可控且能产生成交线索的主渠道"},
            {"验证对象": "卖点", "动作": "把核心卖点拆成标题、主图、短视频开头或客服话术版本", "通过标准": "高点击卖点能继续带来加购、咨询或成交，而不是只带来浏览"}
        ],
        "需要补充的数据": missing_market_fit_inputs(&target_audience, &selling_points, &channels, &use_scenarios, price, &competitor_prices),
        "后续适配 Skill": [
            "experiment_mvp_plan",
            "product_opportunity_score",
            "ops_channel_strategy",
            "creative_title_ctr_scorer",
            "search_product_reviews"
        ],
        "治理说明": "该PMF诊断只基于本轮显式输入和确定性规则；没有联网搜索、没有读取实时用户画像或平台后台，也不会把假设人群写成已验证事实。"
    })
}

pub(super) fn experiment_mvp_plan(input: &Value) -> Value {
    let product_name = string(input, "product_name", "");
    let category = string(input, "category", "通用商品");
    let platform = string(input, "platform", "淘宝");
    let product_ref = string(input, "product_ref", "");
    let target_audience = string(input, "target_audience", "待验证人群");
    let mut selling_points = strings(input, "selling_points");
    if selling_points.is_empty() {
        selling_points = strings(input, "features");
    }
    let audience_segments = strings(input, "audience_segments");
    let channels = strings(input, "channels");
    let use_scenarios = strings(input, "use_scenarios");
    let constraints = strings(input, "constraints");
    let success_metrics = strings(input, "success_metrics");
    let budget = num(input, "budget", 3_000.0).clamp(300.0, 50_000.0);
    let days = num(input, "days", 14.0).round().clamp(7.0, 30.0) as i64;
    let price = num(input, "price", 0.0).max(0.0);
    let cost = num(input, "cost", 0.0).max(0.0);
    let opportunity_score = num(input, "opportunity_score", 0.0).clamp(0.0, 100.0);
    let market_fit_score = num(input, "market_fit_score", 0.0).clamp(0.0, 100.0);

    if product_name.is_empty()
        && product_ref.is_empty()
        && selling_points.is_empty()
        && category == "通用商品"
        && target_audience == "待验证人群"
    {
        return json!({
            "has_data": false,
            "提示": "请至少提供商品、类目、目标人群或卖点；MVP实验计划不会自动读取商品库、投放后台或外部市场数据。",
            "示例参数": {
                "product_name": "低糖冷萃咖啡",
                "category": "饮品",
                "target_audience": "校园学生和通勤人群",
                "selling_points": ["低糖", "即饮", "便携"],
                "budget": 3000,
                "days": 14
            }
        });
    }

    let display_product = if !product_name.is_empty() {
        product_name.clone()
    } else if !product_ref.is_empty() {
        product_ref.clone()
    } else {
        category.clone()
    };
    let resolved_channels = resolved_channel_candidates(&category, &platform, &channels);
    let resolved_audiences = resolved_audience_segments(&target_audience, &audience_segments);
    let primary_metric = primary_experiment_metric(&success_metrics, price, cost);
    let sample_goal = sample_goal_for_budget(budget, price);
    let stop_loss = experiment_stop_loss(budget, price, cost);
    let scale_line = experiment_scale_line(budget, opportunity_score, market_fit_score);

    json!({
        "has_data": true,
        "数据来源": "用户输入 + 确定性MVP实验计划模型",
        "商品": display_product,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "类目": category,
        "平台": platform,
        "目标人群": target_audience,
        "实验周期": format!("{}天", days),
        "总预算": currency(budget),
        "实验目标": format!("用{}验证商品卖点、人群、渠道和价格带是否具备继续投入价值", format_duration(days)),
        "核心假设": experiment_hypotheses(&selling_points, &resolved_audiences, &resolved_channels, &use_scenarios),
        "实验分组": experiment_groups(&selling_points, &resolved_audiences, &resolved_channels, budget),
        "预算拆分": experiment_budget_split(budget),
        "日程计划": experiment_schedule(days),
        "指标体系": {
            "一级指标": primary_metric,
            "过程指标": ["曝光/触达", "点击率", "咨询率", "加购率", "成交转化率", "退款/差评信号"],
            "样本目标": sample_goal,
            "成功线": scale_line,
            "止损线": stop_loss
        },
        "数据记录表头": [
            "日期",
            "实验组",
            "人群",
            "渠道",
            "卖点版本",
            "预算",
            "曝光/触达",
            "点击",
            "咨询/加购",
            "成交",
            "GMV",
            "退款/差评",
            "结论"
        ],
        "复盘判断": experiment_review_rules(price, cost),
        "风险控制": experiment_risk_controls(&constraints, budget),
        "需要补充的数据": missing_experiment_inputs(&selling_points, &audience_segments, &channels, price, cost, &success_metrics),
        "后续适配 Skill": [
            "experiment_review_report",
            "product_opportunity_score",
            "product_market_fit_diagnosis",
            "ops_channel_strategy",
            "creative_title_ctr_scorer"
        ],
        "治理说明": "该MVP实验计划只基于本轮显式输入和确定性规则；不读取真实投放后台、不创建平台任务、不声称实验已经发生。"
    })
}

pub(super) fn experiment_review_report(input: &Value) -> Value {
    let product_name = string(input, "product_name", "");
    let category = string(input, "category", "通用商品");
    let platform = string(input, "platform", "淘宝");
    let product_ref = string(input, "product_ref", "");
    let experiment_name = string(input, "experiment_name", "MVP实验");
    let experiment_goal = string(
        input,
        "experiment_goal",
        "验证商品、人群、渠道和卖点是否值得继续投入",
    );
    let success_metrics = strings(input, "success_metrics");
    let constraints = strings(input, "constraints");
    let budget = num(input, "budget", 0.0).max(0.0);
    let days = num(input, "days", 0.0).round().max(0.0) as i64;
    let price = num(input, "price", 0.0).max(0.0);
    let cost = num(input, "cost", 0.0).max(0.0);
    let groups = parse_experiment_review_groups(input, price, cost);

    if groups.is_empty() {
        return json!({
            "has_data": false,
            "提示": "请提供 experiment_groups 实验结果数组；实验复盘不会自动读取投放后台、订单后台或外部数据。",
            "示例参数": {
                "product_name": "低糖冷萃咖啡",
                "experiment_name": "14天MVP测品",
                "price": 19.9,
                "cost": 8,
                "experiment_groups": [
                    {"name": "G1", "audience": "早八学生", "channel": "校园社群", "selling_point": "低糖提神", "spend": 1000, "impressions": 18000, "clicks": 620, "consultations": 54, "add_to_cart": 38, "orders": 18, "gmv": 358.2, "refunds": 1, "negative_reviews": 0},
                    {"name": "G2", "audience": "通勤白领", "channel": "抖音短视频", "selling_point": "便携即饮", "spend": 1100, "impressions": 22000, "clicks": 520, "consultations": 31, "add_to_cart": 22, "orders": 9, "gmv": 179.1, "refunds": 0, "negative_reviews": 1}
                ]
            }
        });
    }

    let display_product = if !product_name.is_empty() {
        product_name.clone()
    } else if !product_ref.is_empty() {
        product_ref.clone()
    } else {
        category.clone()
    };
    let sorted_groups = ranked_experiment_review_groups(&groups);
    let winner = sorted_groups
        .first()
        .expect("groups is not empty after early return");
    let second = sorted_groups.get(1);
    let decision = experiment_review_decision(winner, second, &groups);
    let overview = experiment_review_overview(&groups, budget, days);
    let performance = sorted_groups
        .iter()
        .map(experiment_review_group_row)
        .collect::<Vec<_>>();
    let baseline = experiment_review_baseline(&input["baseline"], price, cost);

    json!({
        "has_data": true,
        "数据来源": "用户输入实验结果 + 确定性实验复盘模型",
        "商品": display_product,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "类目": category,
        "平台": platform,
        "实验名称": experiment_name,
        "实验目标": experiment_goal,
        "复盘结论": decision.label,
        "推荐动作": decision.action,
        "赢家组合": experiment_review_winner(winner),
        "实验概览": overview,
        "基线对照": baseline,
        "分组表现": performance,
        "核心指标": experiment_review_core_metrics(winner, &groups),
        "达标判断": experiment_review_thresholds(winner, second, &success_metrics),
        "放量/重测/停止依据": decision.reasons,
        "风险信号": experiment_review_risks(&groups, &constraints),
        "下一步动作": experiment_review_next_actions(decision.kind, winner),
        "需要补充的数据": missing_experiment_review_inputs(price, cost, budget, days, &success_metrics, &groups),
        "后续适配 Skill": [
            "experiment_mvp_plan",
            "ops_smart_pricing",
            "ops_channel_strategy",
            "creative_title_ctr_scorer",
            "data_ab_test_analyzer"
        ],
        "治理说明": "该实验复盘只基于本轮显式传入的 experiment_groups、baseline、售价、成本和约束；不联网、不读取真实投放后台、订单后台或评价后台，也不会把缺失数据补写成已发生事实。"
    })
}

pub(super) fn pricing_strategy(input: &Value) -> Value {
    let cost = num(input, "cost", 0.0);
    let competitors = nums(input, "competitor_prices");
    let platform = string(input, "platform", "淘宝");
    let product_ref = string(input, "product_ref", "");
    let avg_comp = if competitors.is_empty() {
        cost * 3.0
    } else {
        average(&competitors)
    };
    let min_comp = competitors
        .iter()
        .copied()
        .reduce(f64::min)
        .unwrap_or(cost * 2.0);
    let max_comp = competitors
        .iter()
        .copied()
        .reduce(f64::max)
        .unwrap_or(cost * 4.0);
    let commission_rate = commission_rate(&platform);
    let recommended = round2((avg_comp * 0.95).max(cost * 1.3));
    let commission = round2(recommended * commission_rate);
    let profit = round2(recommended - cost - commission);
    let margin = pct(profit, recommended);

    json!({
        "成本": cost,
        "平台": platform,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "建议零售价": recommended,
        "平台佣金": commission,
        "单品利润": profit,
        "利润率": percent(margin, 2),
        "竞品分析": {
            "竞品均价": round2(avg_comp),
            "竞品最低价": round2(min_comp),
            "竞品最高价": round2(max_comp),
            "价格竞争力": if recommended < avg_comp { "高" } else { "中" },
        }
    })
}

fn competitor_opportunity_score(price: f64, competitors: &[f64]) -> f64 {
    if competitors.is_empty() || price <= 0.0 {
        return 58.0;
    }
    let avg = average(competitors);
    let gap = pct(price - avg, avg);
    if gap <= -15.0 {
        82.0
    } else if gap <= 5.0 {
        76.0
    } else if gap <= 20.0 {
        62.0
    } else {
        45.0
    }
}

fn heuristic_demand_score(
    category: &str,
    selling_points: &[String],
    current_monthly_sales: f64,
) -> f64 {
    let mut score: f64 = if current_monthly_sales >= 1_000.0 {
        76.0
    } else if current_monthly_sales >= 300.0 {
        68.0
    } else if current_monthly_sales > 0.0 {
        58.0
    } else {
        55.0
    };
    let text = format!("{} {}", category, selling_points.join(" "));
    for keyword in [
        "刚需", "高频", "复购", "低糖", "便携", "学生", "通勤", "礼盒",
    ] {
        if text.contains(keyword) {
            score += 4.0;
        }
    }
    score.clamp(35.0, 88.0)
}

fn differentiation_score(selling_points: &[String], review_pain_score: f64) -> f64 {
    let point_score = match selling_points.len() {
        0 => 45.0,
        1 => 58.0,
        2 => 68.0,
        3..=4 => 78.0,
        _ => 82.0,
    };
    if review_pain_score > 0.0 {
        round1(point_score * 0.65 + review_pain_score * 0.35)
    } else {
        point_score
    }
}

fn feasibility_score(budget: f64, supply_score: f64, risk_signals: &[String]) -> f64 {
    let budget_score = if budget >= 5_000.0 {
        78.0
    } else if budget >= 1_500.0 {
        70.0
    } else if budget > 0.0 {
        58.0
    } else {
        52.0
    };
    let risk_penalty = (risk_signals.len() as f64 * 5.0).min(20.0);
    let base = if supply_score > 0.0 {
        budget_score * 0.55 + supply_score * 0.45
    } else {
        budget_score
    };
    (base - risk_penalty).clamp(25.0, 90.0)
}

fn risk_penalty_score(risk_signals: &[String]) -> f64 {
    match risk_signals.len() {
        0 => 82.0,
        1 => 72.0,
        2 => 62.0,
        3 => 50.0,
        _ => 38.0,
    }
}

fn opportunity_grade(score: f64) -> &'static str {
    if score >= 85.0 {
        "A"
    } else if score >= 70.0 {
        "B"
    } else if score >= 55.0 {
        "C"
    } else {
        "D"
    }
}

fn demand_reason(search_demand_score: f64, current_monthly_sales: f64) -> String {
    if search_demand_score > 0.0 {
        format!("使用显式需求评分 {}", round1(search_demand_score))
    } else if current_monthly_sales > 0.0 {
        format!(
            "使用当前月销量 {} 作为需求代理",
            current_monthly_sales.round()
        )
    } else {
        "未提供搜索热度或销量，按类目和卖点做保守估计".to_string()
    }
}

fn margin_reason(cost: f64, price: f64, platform: &str) -> String {
    if cost > 0.0 && price > 0.0 {
        let margin = pct(price - cost - price * commission_rate(platform), price);
        format!(
            "售价{}、成本{}、{}佣金后毛利率约{}",
            currency(price),
            currency(cost),
            platform,
            percent(margin, 1)
        )
    } else {
        "未同时提供成本和售价，按中性毛利空间处理".to_string()
    }
}

fn differentiation_reason(selling_points: &[String], review_pain_score: f64) -> String {
    if !selling_points.is_empty() && review_pain_score > 0.0 {
        format!(
            "有{}个卖点，并提供评价痛点评分{}",
            selling_points.len(),
            round1(review_pain_score)
        )
    } else if !selling_points.is_empty() {
        format!(
            "有{}个显式卖点：{}",
            selling_points.len(),
            selling_points.join("、")
        )
    } else {
        "缺少明确卖点，差异化偏弱".to_string()
    }
}

fn competitor_reason(price: f64, avg_comp: Option<f64>) -> String {
    match (price > 0.0, avg_comp) {
        (true, Some(avg)) => format!("售价{} vs 竞品均价{}", currency(price), currency(avg)),
        (_, Some(avg)) => format!("有竞品均价{}，但缺少本品售价", currency(avg)),
        _ => "未提供竞品价格，按中性竞争位置处理".to_string(),
    }
}

fn feasibility_reason(budget: f64, supply_score: f64) -> String {
    match (budget > 0.0, supply_score > 0.0) {
        (true, true) => format!(
            "预算{}，供应可行性评分{}",
            currency(budget),
            round1(supply_score)
        ),
        (true, false) => format!("预算{}，但缺少供应链评分", currency(budget)),
        (false, true) => format!("供应可行性评分{}，但缺少测试预算", round1(supply_score)),
        _ => "缺少预算和供应链信息，按保守可行性处理".to_string(),
    }
}

fn risk_reason(risk_signals: &[String]) -> String {
    if risk_signals.is_empty() {
        "未提供明显风险信号".to_string()
    } else {
        format!("识别到风险信号：{}", risk_signals.join("、"))
    }
}

fn opportunity_points(
    selling_points: &[String],
    demand_score: f64,
    gross_margin_score: f64,
    competitor_score: f64,
) -> Vec<Value> {
    let mut points = Vec::new();
    if demand_score >= 70.0 {
        points.push(json!("需求信号较强，适合进入小预算测试"));
    }
    if gross_margin_score >= 70.0 {
        points.push(json!("毛利空间较好，有覆盖获客成本的可能"));
    }
    if competitor_score >= 70.0 {
        points.push(json!("价格相对竞品有一定竞争力"));
    }
    for point in selling_points.iter().take(3) {
        points.push(json!(format!("可放大卖点：{point}")));
    }
    if points.is_empty() {
        points.push(json!(
            "当前机会点不明确，需要先补充需求、竞品、成本或卖点数据"
        ));
    }
    points
}

fn risk_points(
    risk_signals: &[String],
    cost: f64,
    price: f64,
    competitor_prices: &[f64],
) -> Vec<Value> {
    let mut risks = risk_signals
        .iter()
        .take(5)
        .map(|item| json!(item))
        .collect::<Vec<_>>();
    if cost <= 0.0 || price <= 0.0 {
        risks.push(json!("缺少成本或售价，暂不能可靠判断利润空间"));
    }
    if competitor_prices.is_empty() {
        risks.push(json!("缺少竞品价格，无法判断价格带竞争压力"));
    }
    if risks.is_empty() {
        risks.push(json!(
            "未发现显式高风险，但仍需通过小预算测试验证转化和履约"
        ));
    }
    risks
}

fn missing_opportunity_inputs(
    cost: f64,
    price: f64,
    competitor_prices: &[f64],
    search_demand_score: f64,
    review_pain_score: f64,
    risk_signals: &[String],
) -> Vec<Value> {
    let mut missing = Vec::new();
    if cost <= 0.0 {
        missing.push(json!("单位成本/含包装履约成本"));
    }
    if price <= 0.0 {
        missing.push(json!("计划售价或价格带"));
    }
    if competitor_prices.is_empty() {
        missing.push(json!("3-5个直接竞品价格"));
    }
    if search_demand_score <= 0.0 {
        missing.push(json!("搜索热度、类目增长或平台趋势证据"));
    }
    if review_pain_score <= 0.0 {
        missing.push(json!("差评/评价痛点强度"));
    }
    if risk_signals.is_empty() {
        missing.push(json!("供应、保质期、售后、合规等风险线索"));
    }
    missing
}

fn resolved_audience_segments(target_audience: &str, audience_segments: &[String]) -> Vec<String> {
    if !audience_segments.is_empty() {
        return audience_segments.iter().take(5).cloned().collect();
    }
    target_audience
        .split(['、', ',', '，', '/', '|'])
        .map(str::trim)
        .filter(|item| !item.is_empty() && *item != "待验证人群")
        .take(5)
        .map(str::to_owned)
        .collect()
}

fn resolved_channel_candidates(category: &str, platform: &str, channels: &[String]) -> Vec<String> {
    if !channels.is_empty() {
        return channels.iter().take(5).cloned().collect();
    }
    if category.contains("饮品") || category.contains("食品") {
        vec![
            "短视频种草".to_string(),
            "校园/社群私域".to_string(),
            "平台搜索承接".to_string(),
        ]
    } else if category.contains("美妆") || category.contains("服饰") {
        vec![
            "小红书种草".to_string(),
            "短视频测评".to_string(),
            format!("{platform}搜索承接"),
        ]
    } else if category.contains("数码") || category.contains("家电") {
        vec![
            "测评内容".to_string(),
            "搜索承接".to_string(),
            "直播讲解".to_string(),
        ]
    } else {
        vec![
            "内容种草".to_string(),
            "搜索承接".to_string(),
            "私域转化".to_string(),
        ]
    }
}

fn audience_fit_score(
    category: &str,
    target_audience: &str,
    audience_segments: &[String],
    selling_points: &[String],
    pain_points: &[String],
    review_pain_score: f64,
) -> f64 {
    let mut score: f64 = if target_audience == "待验证人群" && audience_segments.is_empty() {
        45.0
    } else if audience_segments.len() >= 2 {
        76.0
    } else {
        64.0
    };
    let text = format!(
        "{} {} {} {}",
        category,
        target_audience,
        selling_points.join(" "),
        pain_points.join(" ")
    );
    for keyword in [
        "学生",
        "宝妈",
        "通勤",
        "新手",
        "敏感肌",
        "健身",
        "送礼",
        "企业",
    ] {
        if text.contains(keyword) {
            score += 3.5;
        }
    }
    if !pain_points.is_empty() {
        score += 6.0;
    }
    if review_pain_score >= 70.0 {
        score += 6.0;
    } else if review_pain_score >= 50.0 {
        score += 3.0;
    }
    score.clamp(30.0, 92.0)
}

fn scenario_fit_score(category: &str, use_scenarios: &[String], selling_points: &[String]) -> f64 {
    let mut score: f64 = match use_scenarios.len() {
        0 => 46.0,
        1 => 62.0,
        2 => 72.0,
        _ => 80.0,
    };
    let text = format!("{} {}", category, selling_points.join(" "));
    for keyword in [
        "便携", "即饮", "家用", "通勤", "低糖", "礼盒", "应急", "入门",
    ] {
        if text.contains(keyword) {
            score += 3.0;
        }
    }
    score.clamp(28.0, 90.0)
}

fn channel_fit_score(
    category: &str,
    platform: &str,
    channels: &[String],
    selling_points: &[String],
) -> f64 {
    let mut score: f64 = if channels.is_empty() {
        45.0
    } else {
        62.0 + channels.len().min(4) as f64 * 5.0
    };
    let text = format!(
        "{} {} {} {}",
        category,
        platform,
        channels.join(" "),
        selling_points.join(" ")
    );
    if text.contains("抖音") || text.contains("短视频") || text.contains("直播") {
        score += 5.0;
    }
    if text.contains("小红书") || text.contains("种草") || text.contains("测评") {
        score += 5.0;
    }
    if text.contains("搜索") || text.contains("淘宝") || text.contains("京东") {
        score += 4.0;
    }
    if text.contains("私域") || text.contains("社群") || text.contains("校园") {
        score += 4.0;
    }
    score.clamp(30.0, 92.0)
}

fn price_fit_score(price: f64, competitor_prices: &[f64], opportunity_score: f64) -> f64 {
    let base = if price > 0.0 && !competitor_prices.is_empty() {
        competitor_opportunity_score(price, competitor_prices)
    } else if opportunity_score > 0.0 {
        opportunity_score * 0.65 + 20.0
    } else {
        55.0
    };
    base.clamp(30.0, 90.0)
}

fn trust_fit_score(risk_signals: &[String], pain_points: &[String]) -> f64 {
    let mut score: f64 = if risk_signals.is_empty() {
        78.0
    } else {
        74.0 - risk_signals.len().min(5) as f64 * 8.0
    };
    let text = format!("{} {}", risk_signals.join(" "), pain_points.join(" "));
    for keyword in ["发货慢", "破损", "过敏", "保质期", "虚假", "投诉", "退款"] {
        if text.contains(keyword) {
            score -= 5.0;
        }
    }
    score.clamp(20.0, 88.0)
}

fn market_fit_stage(score: f64) -> &'static str {
    if score >= 82.0 {
        "PMF信号较强"
    } else if score >= 68.0 {
        "可进入定向验证"
    } else if score >= 52.0 {
        "匹配假设不足"
    } else {
        "暂未形成PMF"
    }
}

fn market_fit_action(score: f64) -> &'static str {
    if score >= 82.0 {
        "围绕优先人群和主渠道做小规模放量验证"
    } else if score >= 68.0 {
        "先验证人群、卖点和渠道三件事，再决定是否放量"
    } else if score >= 52.0 {
        "补充人群痛点、渠道证据和价格带数据后再测"
    } else {
        "先重做定位或换人群，不建议直接投放"
    }
}

fn audience_fit_reason(
    audience_segments: &[String],
    pain_points: &[String],
    review_pain_score: f64,
) -> String {
    if !audience_segments.is_empty() && !pain_points.is_empty() {
        format!(
            "已给出{}个人群和{}个痛点，可形成初步匹配假设",
            audience_segments.len(),
            pain_points.len()
        )
    } else if !audience_segments.is_empty() {
        format!(
            "已给出{}个候选人群，但痛点证据不足",
            audience_segments.len()
        )
    } else if review_pain_score > 0.0 {
        format!(
            "有人群痛点评分{}，但缺少清晰人群分层",
            round1(review_pain_score)
        )
    } else {
        "缺少清晰人群和痛点，匹配判断偏弱".to_string()
    }
}

fn value_fit_reason(selling_points: &[String]) -> String {
    if selling_points.len() >= 3 {
        format!(
            "卖点较完整：{}",
            selling_points
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join("、")
        )
    } else if !selling_points.is_empty() {
        format!(
            "已有卖点：{}，但还需要转成用户利益表达",
            selling_points.join("、")
        )
    } else {
        "缺少显式卖点，价值主张不清晰".to_string()
    }
}

fn channel_fit_reason(channels: &[String], category: &str) -> String {
    if channels.is_empty() {
        format!("{category} 尚未给出渠道，按通用渠道保守判断")
    } else {
        format!("候选渠道：{}", channels.join("、"))
    }
}

fn scenario_fit_reason(use_scenarios: &[String], selling_points: &[String]) -> String {
    if !use_scenarios.is_empty() {
        format!("已给出使用场景：{}", use_scenarios.join("、"))
    } else if !selling_points.is_empty() {
        "未单独提供场景，暂从卖点推断使用场景".to_string()
    } else {
        "缺少使用场景，难以判断用户何时购买和复购".to_string()
    }
}

fn price_fit_reason(price: f64, competitor_prices: &[f64], opportunity_score: f64) -> String {
    if price > 0.0 && !competitor_prices.is_empty() {
        format!(
            "售价{}，竞品均价{}",
            currency(price),
            currency(round2(average(competitor_prices)))
        )
    } else if opportunity_score > 0.0 {
        format!(
            "沿用机会评分{}作为价格带代理判断",
            round1(opportunity_score)
        )
    } else {
        "缺少售价和竞品价格，价格带匹配需要补数".to_string()
    }
}

fn trust_fit_reason(risk_signals: &[String], pain_points: &[String]) -> String {
    if risk_signals.is_empty() && pain_points.is_empty() {
        "未提供显式风险或痛点，信任门槛仍需测试验证".to_string()
    } else if risk_signals.is_empty() {
        format!(
            "有痛点输入：{}，但缺少履约/售后风险线索",
            pain_points.join("、")
        )
    } else {
        format!("风险信号：{}", risk_signals.join("、"))
    }
}

fn audience_priority(audience_segments: &[String], pain_points: &[String]) -> Vec<Value> {
    let mut items = if audience_segments.is_empty() {
        vec![
            json!({"人群": "待验证核心人群", "优先级": "P1", "原因": "当前缺少人群分层，需要先做访谈或小样本测试"}),
        ]
    } else {
        audience_segments
            .iter()
            .take(4)
            .enumerate()
            .map(|(idx, audience)| {
                json!({
                    "人群": audience,
                    "优先级": format!("P{}", idx + 1),
                    "原因": pain_points.get(idx).cloned().unwrap_or_else(|| "需要验证该人群是否有明确购买动机".to_string())
                })
            })
            .collect::<Vec<_>>()
    };
    if items.is_empty() {
        items.push(json!({"人群": "待验证核心人群", "优先级": "P1", "原因": "缺少可排序人群"}));
    }
    items
}

fn channel_priority(channels: &[String], category: &str) -> Vec<Value> {
    channels
        .iter()
        .take(4)
        .enumerate()
        .map(|(idx, channel)| {
            json!({
                "渠道": channel,
                "优先级": format!("P{}", idx + 1),
                "验证重点": channel_validation_focus(channel, category)
            })
        })
        .collect()
}

fn channel_validation_focus(channel: &str, category: &str) -> &'static str {
    if channel.contains("搜索") {
        "验证关键词承接、价格带和详情页转化"
    } else if channel.contains("短视频") || channel.contains("抖音") || channel.contains("直播")
    {
        "验证前三秒卖点、评论问题和加购转化"
    } else if channel.contains("小红书") || channel.contains("种草") || channel.contains("测评")
    {
        "验证痛点叙事、真实体验和收藏/私信"
    } else if channel.contains("私域") || channel.contains("社群") || channel.contains("校园")
    {
        "验证熟人信任、团购转化和复购意愿"
    } else if category.contains("数码") {
        "验证参数解释和信任背书"
    } else {
        "验证点击、咨询、加购和成交链路"
    }
}

fn market_fit_matrix(
    selling_points: &[String],
    audience_segments: &[String],
    channels: &[String],
) -> Vec<Value> {
    let point_fallback = vec!["核心卖点待验证".to_string()];
    let audience_fallback = vec!["待验证人群".to_string()];
    let channel_fallback = vec!["待验证渠道".to_string()];
    let points = if selling_points.is_empty() {
        &point_fallback
    } else {
        selling_points
    };
    let audiences = if audience_segments.is_empty() {
        &audience_fallback
    } else {
        audience_segments
    };
    let channels = if channels.is_empty() {
        &channel_fallback
    } else {
        channels
    };
    points
        .iter()
        .take(3)
        .enumerate()
        .map(|(idx, point)| {
            json!({
                "卖点": point,
                "匹配人群": audiences.get(idx % audiences.len()).cloned().unwrap_or_default(),
                "首选渠道": channels.get(idx % channels.len()).cloned().unwrap_or_default(),
                "测试表达": format!("面向{}，用「{}」作为首屏或开头卖点", audiences.get(idx % audiences.len()).cloned().unwrap_or_default(), point)
            })
        })
        .collect()
}

fn market_fit_risks(
    target_audience: &str,
    selling_points: &[String],
    channels: &[String],
    use_scenarios: &[String],
    price: f64,
    competitor_prices: &[f64],
    risk_signals: &[String],
) -> Vec<Value> {
    let mut risks = risk_signals
        .iter()
        .take(4)
        .map(|risk| json!(format!("显式风险：{risk}")))
        .collect::<Vec<_>>();
    if target_audience == "待验证人群" {
        risks.push(json!("目标人群不清晰，容易出现素材点击高但成交弱"));
    }
    if selling_points.is_empty() {
        risks.push(json!("卖点不清晰，难以区分商品价值和普通功能"));
    }
    if channels.is_empty() {
        risks.push(json!(
            "缺少渠道假设，无法判断内容种草、搜索承接或私域转化哪条链路更适合"
        ));
    }
    if use_scenarios.is_empty() {
        risks.push(json!("缺少使用场景，用户购买时机和复购理由不明确"));
    }
    if price <= 0.0 || competitor_prices.is_empty() {
        risks.push(json!("缺少售价或竞品价，价格带匹配仍需补数"));
    }
    if risks.is_empty() {
        risks.push(json!(
            "当前匹配风险较少，但仍需用真实点击、咨询、加购和成交数据验证"
        ));
    }
    risks
}

fn market_fit_validation_questions(
    audience_segments: &[String],
    channels: &[String],
    selling_points: &[String],
) -> Vec<Value> {
    vec![
        json!(format!(
            "哪一个人群对该商品最先产生购买动机？候选：{}",
            list_or_placeholder(audience_segments, "待验证人群")
        )),
        json!(format!(
            "哪一个渠道能用最低成本拿到有效咨询或加购？候选：{}",
            list_or_placeholder(channels, "待验证渠道")
        )),
        json!(format!(
            "哪一个卖点既能带来点击，又能解释价格和购买理由？候选：{}",
            list_or_placeholder(selling_points, "待验证卖点")
        )),
        json!("用户不购买的首要原因是价格、信任、场景不强，还是竞品替代？"),
    ]
}

fn list_or_placeholder(items: &[String], placeholder: &str) -> String {
    if items.is_empty() {
        placeholder.to_string()
    } else {
        items.iter().take(5).cloned().collect::<Vec<_>>().join("、")
    }
}

fn missing_market_fit_inputs(
    target_audience: &str,
    selling_points: &[String],
    channels: &[String],
    use_scenarios: &[String],
    price: f64,
    competitor_prices: &[f64],
) -> Vec<Value> {
    let mut missing = Vec::new();
    if target_audience == "待验证人群" {
        missing.push(json!("目标人群或候选人群分层"));
    }
    if selling_points.is_empty() {
        missing.push(json!("商品卖点和用户利益表达"));
    }
    if channels.is_empty() {
        missing.push(json!("候选渠道或当前主要渠道"));
    }
    if use_scenarios.is_empty() {
        missing.push(json!("使用场景和购买触发时机"));
    }
    if price <= 0.0 {
        missing.push(json!("计划售价或价格带"));
    }
    if competitor_prices.is_empty() {
        missing.push(json!("直接竞品价格或替代品价格"));
    }
    missing
}

fn format_duration(days: i64) -> String {
    if days <= 14 {
        format!("{days}天小预算")
    } else {
        format!("{days}天分阶段")
    }
}

fn primary_experiment_metric(success_metrics: &[String], price: f64, cost: f64) -> String {
    if let Some(metric) = success_metrics
        .first()
        .filter(|value| !value.trim().is_empty())
    {
        return metric.clone();
    }
    if price > 0.0 && cost > 0.0 {
        "单品毛利覆盖获客成本且成交转化率达标".to_string()
    } else {
        "有效咨询/加购率达标，并能解释用户购买动机".to_string()
    }
}

fn sample_goal_for_budget(budget: f64, price: f64) -> String {
    let clicks = (budget / 1.5).round().clamp(100.0, 10_000.0) as i64;
    if price > 0.0 {
        let orders = (budget / price).round().clamp(5.0, 500.0) as i64;
        format!("至少获得{clicks}次点击/触达反馈，并争取{orders}单以内的小样本成交验证")
    } else {
        format!("至少获得{clicks}次点击/触达反馈，并记录咨询、加购和拒绝原因")
    }
}

fn experiment_stop_loss(budget: f64, price: f64, cost: f64) -> Value {
    let daily_loss = round2((budget * 0.25).max(100.0));
    let min_margin = if price > 0.0 && cost > 0.0 {
        percent(pct(price - cost, price), 1)
    } else {
        "毛利未知".to_string()
    };
    json!({
        "预算止损": format!("任一实验组连续2天消耗超过{}且无有效咨询/加购，暂停该组", currency(daily_loss)),
        "转化止损": "点击或触达足量后仍无咨询/加购，先改卖点或人群，不继续加预算",
        "利润止损": format!("若成交后毛利口径低于预期或履约风险高，即使有销量也不放量；当前毛利参考：{min_margin}"),
        "体验止损": "出现集中退款、差评、发货或质量问题时，先暂停测试并排查履约"
    })
}

fn experiment_scale_line(budget: f64, opportunity_score: f64, market_fit_score: f64) -> Value {
    let confidence = if opportunity_score >= 75.0 || market_fit_score >= 75.0 {
        "较高"
    } else if opportunity_score >= 60.0 || market_fit_score >= 60.0 {
        "中等"
    } else {
        "保守"
    };
    json!({
        "放量条件": [
            "至少一个人群-渠道-卖点组合连续2天表现优于其它组合",
            "咨询/加购/成交中至少一个关键指标达到预设成功线",
            "没有集中退款、差评、履约或合规风险"
        ],
        "建议放量方式": format!("信心等级为{confidence}；通过后单日预算最多扩大到当前日均预算的1.5-2倍"),
        "预算参考": currency((budget / 7.0).max(100.0))
    })
}

fn experiment_hypotheses(
    selling_points: &[String],
    audiences: &[String],
    channels: &[String],
    scenarios: &[String],
) -> Vec<Value> {
    vec![
        json!(format!(
            "目标人群假设：{}中至少有一个人群存在明确购买动机",
            list_or_placeholder(audiences, "待验证人群")
        )),
        json!(format!(
            "卖点假设：{}能够带来点击、咨询或加购",
            list_or_placeholder(selling_points, "待验证卖点")
        )),
        json!(format!(
            "渠道假设：{}中至少有一个渠道可以用可控成本触达有效人群",
            list_or_placeholder(channels, "待验证渠道")
        )),
        json!(format!(
            "场景假设：{}能解释用户为什么现在购买",
            list_or_placeholder(scenarios, "待验证使用场景")
        )),
    ]
}

fn experiment_groups(
    selling_points: &[String],
    audiences: &[String],
    channels: &[String],
    budget: f64,
) -> Vec<Value> {
    let point_fallback = vec!["核心卖点A".to_string(), "核心卖点B".to_string()];
    let audience_fallback = vec!["核心人群A".to_string(), "核心人群B".to_string()];
    let channel_fallback = vec!["内容种草".to_string(), "搜索/私域承接".to_string()];
    let points = if selling_points.is_empty() {
        &point_fallback
    } else {
        selling_points
    };
    let audiences = if audiences.is_empty() {
        &audience_fallback
    } else {
        audiences
    };
    let channels = if channels.is_empty() {
        &channel_fallback
    } else {
        channels
    };
    (0..3)
        .map(|idx| {
            json!({
                "实验组": format!("G{}", idx + 1),
                "人群": audiences.get(idx % audiences.len()).cloned().unwrap_or_default(),
                "渠道": channels.get(idx % channels.len()).cloned().unwrap_or_default(),
                "卖点版本": points.get(idx % points.len()).cloned().unwrap_or_default(),
                "预算": currency(round2(budget * experiment_group_weight(idx))),
                "目的": match idx {
                    0 => "验证最直接购买动机",
                    1 => "验证替代人群或替代卖点",
                    _ => "保留对照组，避免单一结论误判",
                }
            })
        })
        .collect()
}

fn experiment_group_weight(index: usize) -> f64 {
    match index {
        0 => 0.40,
        1 => 0.35,
        _ => 0.25,
    }
}

fn experiment_budget_split(budget: f64) -> Vec<Value> {
    vec![
        json!({"用途": "内容/素材制作", "占比": "20%", "预算": currency(round2(budget * 0.20)), "说明": "标题、主图、短视频脚本或社群海报"}),
        json!({"用途": "触达/投放/样品", "占比": "55%", "预算": currency(round2(budget * 0.55)), "说明": "用于小预算投放、达人样品、社群试卖或搜索承接"}),
        json!({"用途": "转化承接", "占比": "15%", "预算": currency(round2(budget * 0.15)), "说明": "优惠券、客服话术、详情页首屏或落地页调整"}),
        json!({"用途": "复盘和机动", "占比": "10%", "预算": currency(round2(budget * 0.10)), "说明": "补测表现较好的组合或处理异常风险"}),
    ]
}

fn experiment_schedule(days: i64) -> Vec<Value> {
    if days <= 10 {
        vec![
            json!({"阶段": "准备", "时间": "第1天", "动作": ["确定人群-卖点-渠道组合", "准备素材和记录表"], "产出": "实验分组表"}),
            json!({"阶段": "测试", "时间": "第2-6天", "动作": ["上线2-3个实验组", "每天记录点击、咨询、加购、成交和拒绝原因"], "产出": "过程数据"}),
            json!({"阶段": "复盘", "时间": "第7-10天", "动作": ["保留赢家组合", "停止明显无效组合", "判断是否二次测试"], "产出": "继续/调整/暂停结论"}),
        ]
    } else {
        vec![
            json!({"阶段": "准备", "时间": "第1-2天", "动作": ["确认实验假设", "准备素材、落地页、客服话术和记录表"], "产出": "实验看板"}),
            json!({"阶段": "第一轮测试", "时间": "第3-7天", "动作": ["三组小预算并行测试", "记录过程指标和用户反馈"], "产出": "第一轮赢家/输家组合"}),
            json!({"阶段": "第二轮优化", "时间": "第8-11天", "动作": ["放大赢家组合1.5倍预算", "替换低效卖点或人群"], "产出": "优化后数据"}),
            json!({"阶段": "复盘决策", "时间": "第12-14天", "动作": ["核算毛利和获客成本", "整理退款/差评/履约风险", "决定放量、改款或暂停"], "产出": "MVP实验复盘"}),
        ]
    }
}

fn experiment_review_rules(price: f64, cost: f64) -> Vec<Value> {
    let margin_rule = if price > 0.0 && cost > 0.0 {
        format!(
            "单笔毛利约{}，获客成本必须低于该毛利并保留履约缓冲",
            currency(round2(price - cost))
        )
    } else {
        "必须补齐售价、成本和履约费用后再判断能否放量".to_string()
    };
    vec![
        json!({"结论": "继续放量", "条件": ["赢家组合明确", "关键指标达标", "无集中风险", margin_rule]}),
        json!({"结论": "调整再测", "条件": ["点击高但咨询/加购弱", "人群反馈分散", "卖点能吸引但价格或信任不足"]}),
        json!({"结论": "暂停", "条件": ["多个组合均无有效反馈", "毛利无法覆盖获客成本", "出现明显履约/售后/合规风险"]}),
    ]
}

fn experiment_risk_controls(constraints: &[String], budget: f64) -> Vec<Value> {
    let mut controls = constraints
        .iter()
        .take(4)
        .map(|item| json!({"约束": item, "控制动作": "上线前写入实验检查清单，每日复盘是否触发"}))
        .collect::<Vec<_>>();
    controls.push(json!({
        "约束": "预算过度消耗",
        "控制动作": format!("单日消耗超过{}且没有有效反馈时暂停", currency(round2((budget / 5.0).max(100.0))))
    }));
    controls.push(json!({
        "约束": "结论过早放大",
        "控制动作": "至少观察2天连续数据，不因单日偶然成交直接放量"
    }));
    controls
}

fn missing_experiment_inputs(
    selling_points: &[String],
    audience_segments: &[String],
    channels: &[String],
    price: f64,
    cost: f64,
    success_metrics: &[String],
) -> Vec<Value> {
    let mut missing = Vec::new();
    if selling_points.is_empty() {
        missing.push(json!("待验证卖点"));
    }
    if audience_segments.is_empty() {
        missing.push(json!("候选人群分层"));
    }
    if channels.is_empty() {
        missing.push(json!("候选渠道"));
    }
    if price <= 0.0 {
        missing.push(json!("计划售价"));
    }
    if cost <= 0.0 {
        missing.push(json!("成本和履约费用"));
    }
    if success_metrics.is_empty() {
        missing.push(json!("成功指标和止损指标"));
    }
    missing
}

#[derive(Clone, Debug)]
struct ExperimentReviewGroup {
    name: String,
    audience: String,
    channel: String,
    selling_point: String,
    spend: f64,
    impressions: f64,
    clicks: f64,
    consultations: f64,
    add_to_cart: f64,
    orders: f64,
    gmv: f64,
    refunds: f64,
    negative_reviews: f64,
    gross_profit_after_spend: f64,
    score: f64,
}

struct ExperimentReviewDecision {
    kind: &'static str,
    label: &'static str,
    action: String,
    reasons: Vec<Value>,
}

fn parse_experiment_review_groups(
    input: &Value,
    price: f64,
    cost: f64,
) -> Vec<ExperimentReviewGroup> {
    input["experiment_groups"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| parse_experiment_review_group(item, index, price, cost))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_experiment_review_group(
    value: &Value,
    index: usize,
    price: f64,
    cost: f64,
) -> Option<ExperimentReviewGroup> {
    if !value.is_object() {
        return None;
    }
    let name = review_string(
        value,
        &["name", "group", "experiment_group", "实验组", "组名"],
        &format!("G{}", index + 1),
    );
    let audience = review_string(
        value,
        &["audience", "target_audience", "segment", "人群", "目标人群"],
        "未标注人群",
    );
    let channel = review_string(value, &["channel", "source", "渠道", "来源"], "未标注渠道");
    let selling_point = review_string(
        value,
        &["selling_point", "value_prop", "message", "卖点", "卖点版本"],
        "未标注卖点",
    );
    let spend = review_num(
        value,
        &[
            "spend", "ad_spend", "budget", "cost", "消耗", "花费", "预算",
        ],
        0.0,
    )
    .max(0.0);
    let impressions = review_num(
        value,
        &[
            "impressions",
            "exposure",
            "reach",
            "views",
            "曝光",
            "触达",
            "展示",
        ],
        0.0,
    )
    .max(0.0);
    let clicks = review_num(value, &["clicks", "click", "点击"], 0.0).max(0.0);
    let consultations = review_num(
        value,
        &[
            "consultations",
            "inquiries",
            "consults",
            "leads",
            "咨询",
            "询单",
            "线索",
        ],
        0.0,
    )
    .max(0.0);
    let add_to_cart = review_num(
        value,
        &["add_to_cart", "carts", "cart", "加购", "加入购物车"],
        0.0,
    )
    .max(0.0);
    let orders = review_num(
        value,
        &["orders", "sales", "conversions", "成交", "订单", "支付订单"],
        0.0,
    )
    .max(0.0);
    let mut gmv = review_num(
        value,
        &["gmv", "revenue", "sales_amount", "营收", "销售额", "GMV"],
        0.0,
    )
    .max(0.0);
    if gmv <= 0.0 && price > 0.0 && orders > 0.0 {
        gmv = orders * price;
    }
    let refunds = review_num(
        value,
        &["refunds", "refund_count", "return_count", "退款", "退货"],
        0.0,
    )
    .max(0.0);
    let negative_reviews = review_num(
        value,
        &[
            "negative_reviews",
            "bad_reviews",
            "complaints",
            "差评",
            "投诉",
        ],
        0.0,
    )
    .max(0.0);

    let signal = spend
        + impressions
        + clicks
        + consultations
        + add_to_cart
        + orders
        + gmv
        + refunds
        + negative_reviews;
    if signal <= 0.0 {
        return None;
    }

    let gross_profit_after_spend = if cost > 0.0 && orders > 0.0 {
        gmv - cost * orders - spend
    } else if gmv > 0.0 && spend > 0.0 {
        gmv - spend
    } else if spend > 0.0 && orders <= 0.0 && gmv <= 0.0 {
        -spend
    } else {
        0.0
    };

    let mut group = ExperimentReviewGroup {
        name,
        audience,
        channel,
        selling_point,
        spend,
        impressions,
        clicks,
        consultations,
        add_to_cart,
        orders,
        gmv,
        refunds,
        negative_reviews,
        gross_profit_after_spend,
        score: 0.0,
    };
    group.score = experiment_review_score(&group);
    Some(group)
}

fn review_string(value: &Value, keys: &[&str], default: &str) -> String {
    keys.iter()
        .find_map(|key| {
            value[*key]
                .as_str()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| default.to_string())
}

fn review_num(value: &Value, keys: &[&str], default: f64) -> f64 {
    keys.iter()
        .find_map(|key| value_to_review_num(&value[*key]))
        .unwrap_or(default)
}

fn value_to_review_num(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|number| number as f64))
        .or_else(|| value.as_u64().map(|number| number as f64))
        .or_else(|| {
            value.as_str().and_then(|text| {
                let normalized = text
                    .trim()
                    .replace([',', '，', '¥', '￥', '%'], "")
                    .replace("元", "");
                normalized.parse::<f64>().ok()
            })
        })
}

fn ranked_experiment_review_groups(groups: &[ExperimentReviewGroup]) -> Vec<ExperimentReviewGroup> {
    let mut ranked = groups.to_vec();
    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked
}

fn experiment_review_score(group: &ExperimentReviewGroup) -> f64 {
    let conversion_rate = pct(group.orders, group.clicks);
    let engagement_rate = pct(group.consultations + group.add_to_cart, group.clicks);
    let risk_rate = experiment_review_risk_rate(group);

    let volume_score = if group.clicks >= 500.0 || group.orders >= 20.0 {
        82.0
    } else if group.clicks >= 100.0 || group.orders >= 5.0 {
        68.0
    } else if group.clicks > 0.0 || group.orders > 0.0 {
        48.0
    } else {
        28.0
    };
    let conversion_score = if conversion_rate >= 4.0 {
        88.0
    } else if conversion_rate >= 2.0 {
        74.0
    } else if conversion_rate >= 1.0 {
        58.0
    } else if group.orders > 0.0 {
        45.0
    } else {
        25.0
    };
    let engagement_score = if engagement_rate >= 15.0 {
        84.0
    } else if engagement_rate >= 8.0 {
        72.0
    } else if engagement_rate >= 4.0 {
        56.0
    } else if engagement_rate > 0.0 {
        42.0
    } else {
        28.0
    };
    let profit_score = if group.gross_profit_after_spend > group.spend * 0.20 {
        88.0
    } else if group.gross_profit_after_spend > 0.0 {
        76.0
    } else if group.gmv > 0.0 && group.spend > 0.0 && pct(group.gmv, group.spend) >= 100.0 {
        66.0
    } else if group.gross_profit_after_spend >= -group.spend * 0.20 {
        52.0
    } else {
        34.0
    };
    let risk_score = if risk_rate <= 3.0 {
        84.0
    } else if risk_rate <= 8.0 {
        66.0
    } else if risk_rate <= 15.0 {
        46.0
    } else {
        26.0
    };

    round1(
        conversion_score * 0.26
            + profit_score * 0.22
            + engagement_score * 0.18
            + volume_score * 0.18
            + risk_score * 0.16,
    )
}

fn experiment_review_risk_rate(group: &ExperimentReviewGroup) -> f64 {
    pct(
        group.refunds + group.negative_reviews,
        group.orders.max(1.0),
    )
}

fn experiment_review_decision(
    winner: &ExperimentReviewGroup,
    second: Option<&ExperimentReviewGroup>,
    groups: &[ExperimentReviewGroup],
) -> ExperimentReviewDecision {
    let score_gap = second
        .map(|group| winner.score - group.score)
        .unwrap_or(12.0);
    let sufficient_sample = winner.clicks >= 100.0 || winner.orders >= 5.0;
    let high_risk = experiment_review_risk_rate(winner) > 15.0 || winner.negative_reviews >= 3.0;
    let no_effective_signal = groups
        .iter()
        .all(|group| group.orders <= 0.0 && group.consultations + group.add_to_cart < 3.0);
    let profit_positive = winner.gross_profit_after_spend > 0.0
        || (winner.gmv > 0.0 && winner.spend > 0.0 && winner.gmv / winner.spend >= 1.2);
    let conversion_signal =
        winner.orders > 0.0 || winner.consultations + winner.add_to_cart >= 10.0;

    if no_effective_signal {
        return ExperimentReviewDecision {
            kind: "stop",
            label: "暂停/停止",
            action: "多组都没有形成有效咨询、加购或成交信号，建议停止当前组合，回到人群、\
                     卖点或渠道假设重做。"
                .to_string(),
            reasons: vec![
                json!("所有实验组有效反馈不足"),
                json!("继续消耗预算难以提高结论质量"),
                json!("应先复核商品卖点、价格、信任背书和触达渠道"),
            ],
        };
    }
    if high_risk {
        return ExperimentReviewDecision {
            kind: "stop",
            label: "暂停/排查风险",
            action: "赢家组合存在退款、差评或投诉风险，先暂停放量，完成履约、\
                     质量和售后排查后再决定是否重测。"
                .to_string(),
            reasons: vec![
                json!(format!(
                    "赢家风险率为{}，超过可直接放量阈值",
                    percent(experiment_review_risk_rate(winner), 1)
                )),
                json!("风险未排除前，继续放量会放大售后和口碑问题"),
            ],
        };
    }
    if sufficient_sample && score_gap >= 8.0 && profit_positive {
        return ExperimentReviewDecision {
            kind: "scale",
            label: "可小步放量",
            action: "保留当前赢家组合，单日预算先按 1.5 \
                     倍以内小步放大，并保留对照组观察转化、毛利和风险是否稳定。"
                .to_string(),
            reasons: vec![
                json!(format!(
                    "赢家综合分 {}，与第二名差距 {} 分",
                    round1(winner.score),
                    round1(score_gap)
                )),
                json!(format!(
                    "样本达到基础判断线：点击 {}，成交 {}",
                    round1(winner.clicks),
                    round1(winner.orders)
                )),
                json!(format!(
                    "毛利扣消耗后参考为 {}",
                    currency(winner.gross_profit_after_spend)
                )),
                json!("未发现高风险退款或差评信号"),
            ],
        };
    }
    if !sufficient_sample {
        return ExperimentReviewDecision {
            kind: "inconclusive",
            label: "样本不足，暂不决策",
            action: "当前点击或成交样本不足，不建议直接放量；先延长测试或集中预算到高反馈组合，\
                     补齐判断样本。"
                .to_string(),
            reasons: vec![
                json!(format!(
                    "赢家点击 {}、成交 {}，尚未达到基础判断线",
                    round1(winner.clicks),
                    round1(winner.orders)
                )),
                json!("样本不足时，单组偶然成交或单日波动容易误导决策"),
            ],
        };
    }
    if conversion_signal {
        return ExperimentReviewDecision {
            kind: "adjust_retest",
            label: "调整后重测",
            action: "保留有信号的赢家方向，但先调整卖点表达、价格承接、详情页或客服话术，\
                     再做第二轮小预算重测。"
                .to_string(),
            reasons: vec![
                json!("已有点击、咨询、加购或成交信号，但利润、差距或稳定性暂未达到放量线"),
                json!(format!(
                    "当前赢家毛利扣消耗后参考为 {}",
                    currency(winner.gross_profit_after_spend)
                )),
                json!(format!(
                    "赢家与第二名差距 {} 分，仍需复测确认",
                    round1(score_gap)
                )),
            ],
        };
    }

    ExperimentReviewDecision {
        kind: "stop",
        label: "暂停/停止",
        action: "当前组合没有形成足够转化信号，建议停止消耗，回到机会评分和PMF诊断重构实验。"
            .to_string(),
        reasons: vec![
            json!("样本已有一定规模但转化信号弱"),
            json!("继续放量前需要重新定义人群、卖点或渠道"),
        ],
    }
}

fn experiment_review_overview(groups: &[ExperimentReviewGroup], budget: f64, days: i64) -> Value {
    let total_spend = groups.iter().map(|group| group.spend).sum::<f64>();
    let total_impressions = groups.iter().map(|group| group.impressions).sum::<f64>();
    let total_clicks = groups.iter().map(|group| group.clicks).sum::<f64>();
    let total_orders = groups.iter().map(|group| group.orders).sum::<f64>();
    let total_gmv = groups.iter().map(|group| group.gmv).sum::<f64>();
    json!({
        "实验组数": groups.len(),
        "实验周期": if days > 0 { format!("{days}天") } else { "未提供".to_string() },
        "计划预算": if budget > 0.0 { currency(budget) } else { "未提供".to_string() },
        "实际消耗": currency(total_spend),
        "预算使用率": if budget > 0.0 { percent(pct(total_spend, budget), 1) } else { "未提供".to_string() },
        "总曝光/触达": round1(total_impressions),
        "总点击": round1(total_clicks),
        "总成交": round1(total_orders),
        "总GMV": currency(total_gmv),
        "整体点击率": percent(pct(total_clicks, total_impressions), 2),
        "整体成交转化率": percent(pct(total_orders, total_clicks), 2),
        "整体ROAS": round2(if total_spend > 0.0 { total_gmv / total_spend } else { 0.0 }),
    })
}

fn experiment_review_baseline(value: &Value, price: f64, cost: f64) -> Value {
    parse_experiment_review_group(value, 0, price, cost)
        .map(|group| {
            json!({
                "说明": "使用用户提供的 baseline / 对照组数据",
                "点击率": percent(pct(group.clicks, group.impressions), 2),
                "咨询加购率": percent(pct(group.consultations + group.add_to_cart, group.clicks), 2),
                "成交转化率": percent(pct(group.orders, group.clicks), 2),
                "ROAS": round2(if group.spend > 0.0 { group.gmv / group.spend } else { 0.0 }),
                "毛利扣消耗": currency(group.gross_profit_after_spend),
                "风险率": percent(experiment_review_risk_rate(&group), 1)
            })
        })
        .unwrap_or_else(|| json!("未提供基线或对照组，当前仅做实验组之间的相对比较"))
}

fn experiment_review_group_row(group: &ExperimentReviewGroup) -> Value {
    json!({
        "实验组": group.name,
        "人群": group.audience,
        "渠道": group.channel,
        "卖点版本": group.selling_point,
        "综合分": round1(group.score),
        "消耗": currency(group.spend),
        "曝光/触达": round1(group.impressions),
        "点击": round1(group.clicks),
        "点击率": percent(pct(group.clicks, group.impressions), 2),
        "咨询": round1(group.consultations),
        "加购": round1(group.add_to_cart),
        "咨询加购率": percent(pct(group.consultations + group.add_to_cart, group.clicks), 2),
        "成交": round1(group.orders),
        "成交转化率": percent(pct(group.orders, group.clicks), 2),
        "GMV": currency(group.gmv),
        "ROAS": round2(if group.spend > 0.0 { group.gmv / group.spend } else { 0.0 }),
        "获客成本": if group.orders > 0.0 { currency(group.spend / group.orders) } else { "无成交".to_string() },
        "毛利扣消耗": currency(group.gross_profit_after_spend),
        "退款": round1(group.refunds),
        "差评/投诉": round1(group.negative_reviews),
        "风险率": percent(experiment_review_risk_rate(group), 1)
    })
}

fn experiment_review_winner(group: &ExperimentReviewGroup) -> Value {
    json!({
        "实验组": group.name,
        "人群": group.audience,
        "渠道": group.channel,
        "卖点版本": group.selling_point,
        "综合分": round1(group.score),
        "判断": "当前实验结果中的相对最优组合"
    })
}

fn experiment_review_core_metrics(
    winner: &ExperimentReviewGroup,
    groups: &[ExperimentReviewGroup],
) -> Value {
    let scores = groups.iter().map(|group| group.score).collect::<Vec<_>>();
    json!({
        "赢家综合分": round1(winner.score),
        "全组平均分": round1(average(&scores)),
        "赢家点击率": percent(pct(winner.clicks, winner.impressions), 2),
        "赢家咨询加购率": percent(pct(winner.consultations + winner.add_to_cart, winner.clicks), 2),
        "赢家成交转化率": percent(pct(winner.orders, winner.clicks), 2),
        "赢家ROAS": round2(if winner.spend > 0.0 { winner.gmv / winner.spend } else { 0.0 }),
        "赢家获客成本": if winner.orders > 0.0 { currency(winner.spend / winner.orders) } else { "无成交".to_string() },
        "赢家毛利扣消耗": currency(winner.gross_profit_after_spend),
        "赢家退款差评率": percent(experiment_review_risk_rate(winner), 1)
    })
}

fn experiment_review_thresholds(
    winner: &ExperimentReviewGroup,
    second: Option<&ExperimentReviewGroup>,
    success_metrics: &[String],
) -> Vec<Value> {
    let score_gap = second
        .map(|group| winner.score - group.score)
        .unwrap_or(winner.score);
    let custom_metrics = if success_metrics.is_empty() {
        json!("未提供自定义成功指标，使用点击、咨询/加购、成交、毛利和风险的默认复盘口径")
    } else {
        json!(success_metrics)
    };
    vec![
        json!({"指标": "样本量", "结果": if winner.clicks >= 100.0 || winner.orders >= 5.0 { "达标" } else { "不足" }, "依据": format!("点击 {}，成交 {}", round1(winner.clicks), round1(winner.orders))}),
        json!({"指标": "组间差异", "结果": if score_gap >= 8.0 { "明显" } else { "不明显" }, "依据": format!("赢家与第二名综合分差距 {}", round1(score_gap))}),
        json!({"指标": "转化", "结果": if winner.orders > 0.0 { "有成交信号" } else if winner.consultations + winner.add_to_cart > 0.0 { "有前置转化信号" } else { "弱" }, "依据": format!("咨询+加购 {}，成交 {}", round1(winner.consultations + winner.add_to_cart), round1(winner.orders))}),
        json!({"指标": "利润", "结果": if winner.gross_profit_after_spend > 0.0 { "正向" } else { "未达正向" }, "依据": format!("毛利扣消耗 {}", currency(winner.gross_profit_after_spend))}),
        json!({"指标": "风险", "结果": if experiment_review_risk_rate(winner) <= 8.0 { "可控" } else { "偏高" }, "依据": format!("退款+差评/成交 = {}", percent(experiment_review_risk_rate(winner), 1))}),
        json!({"指标": "用户自定义成功指标", "结果": custom_metrics}),
    ]
}

fn experiment_review_risks(groups: &[ExperimentReviewGroup], constraints: &[String]) -> Vec<Value> {
    let mut risks = Vec::new();
    for group in groups {
        if experiment_review_risk_rate(group) > 8.0 {
            risks.push(json!({
                "实验组": group.name,
                "风险": "退款/差评/投诉偏高",
                "处理": "先排查履约、品质、客服承接和用户预期，不直接放量"
            }));
        }
        if group.spend > 0.0 && group.orders <= 0.0 && group.consultations + group.add_to_cart < 3.0
        {
            risks.push(json!({
                "实验组": group.name,
                "风险": "有消耗但无有效转化",
                "处理": "暂停该组，重做卖点、人群或渠道假设"
            }));
        }
        if group.gross_profit_after_spend < -group.spend * 0.5 && group.spend > 0.0 {
            risks.push(json!({
                "实验组": group.name,
                "风险": "毛利难以覆盖消耗",
                "处理": "复核价格、成本、获客成本和优惠策略"
            }));
        }
    }
    for constraint in constraints.iter().take(4) {
        risks.push(json!({
            "约束": constraint,
            "处理": "放量或重测前需要纳入检查清单"
        }));
    }
    if risks.is_empty() {
        risks.push(json!(
            "未发现集中退款、差评、无效消耗或显式约束风险；仍需继续观察放量后的稳定性"
        ));
    }
    risks
}

fn experiment_review_next_actions(kind: &str, winner: &ExperimentReviewGroup) -> Vec<Value> {
    match kind {
        "scale" => vec![
            json!({"动作": "小步放量", "说明": format!("保留 {} / {} / {} 组合，单日预算先扩大到当前的1.5倍以内", winner.audience, winner.channel, winner.selling_point)}),
            json!({"动作": "保留对照", "说明": "至少保留一个低预算对照组，避免放量后误判渠道或素材自然波动"}),
            json!({"动作": "复核利润", "说明": "同步调用定价或财务口径核算，确认毛利、优惠、履约和售后成本"}),
        ],
        "adjust_retest" => vec![
            json!({"动作": "保留赢家方向", "说明": format!("继续使用 {} / {} 方向，但重写卖点表达或转化承接", winner.audience, winner.channel)}),
            json!({"动作": "重测价格和信任", "说明": "补充价格锚点、评价背书、详情页首屏或客服话术后再跑一轮"}),
            json!({"动作": "缩小变量", "说明": "第二轮一次只改卖点、价格或渠道中的一个变量"}),
        ],
        "stop" => vec![
            json!({"动作": "停止当前组合", "说明": "停止继续消耗无效或高风险组合预算"}),
            json!({"动作": "回到假设", "说明": "重新做商品机会评分、PMF诊断或用户痛点资料搜索"}),
            json!({"动作": "整理失败原因", "说明": "记录失败来自人群不准、卖点弱、价格高、信任不足还是渠道不匹配"}),
        ],
        _ => vec![
            json!({"动作": "补样本", "说明": "延长2-3天或把预算集中到反馈最高组合，补齐点击和成交样本"}),
            json!({"动作": "补数据", "说明": "补齐各组消耗、曝光、点击、咨询、加购、成交、GMV、退款和差评"}),
            json!({"动作": "暂不放量", "说明": "样本不足前不做大预算放量或永久停止判断"}),
        ],
    }
}

fn missing_experiment_review_inputs(
    price: f64,
    cost: f64,
    budget: f64,
    days: i64,
    success_metrics: &[String],
    groups: &[ExperimentReviewGroup],
) -> Vec<Value> {
    let mut missing = Vec::new();
    if price <= 0.0 {
        missing.push(json!("售价 price"));
    }
    if cost <= 0.0 {
        missing.push(json!("单位成本和履约成本 cost"));
    }
    if budget <= 0.0 {
        missing.push(json!("实验计划预算 budget"));
    }
    if days <= 0 {
        missing.push(json!("实验周期 days"));
    }
    if success_metrics.is_empty() {
        missing.push(json!("预设成功指标 success_metrics"));
    }
    if groups.iter().all(|group| group.spend <= 0.0) {
        missing.push(json!("各实验组实际消耗 spend"));
    }
    if groups.iter().all(|group| group.impressions <= 0.0) {
        missing.push(json!("各实验组曝光/触达 impressions"));
    }
    if groups.iter().all(|group| group.clicks <= 0.0) {
        missing.push(json!("各实验组点击 clicks"));
    }
    if groups.iter().all(|group| group.orders <= 0.0) {
        missing.push(json!("各实验组成交 orders"));
    }
    if groups.iter().all(|group| group.gmv <= 0.0) {
        missing.push(json!("各实验组GMV或营收 gmv"));
    }
    missing
}

pub(super) fn promo_planning(input: &Value) -> Value {
    let product = string(input, "product", "通用商品");
    let product_ref = string(input, "product_ref", "");
    let budget = num(input, "budget", 10_000.0).max(0.0);
    let platform = string(input, "platform", "淘宝");
    let phases = vec![
        promo_phase(
            "预热期",
            3,
            budget,
            0.15,
            "加购+收藏",
            &["短视频种草", "优惠券预发放"],
        ),
        promo_phase(
            "爆发期",
            2,
            budget,
            0.55,
            "冲销量",
            &["限时折扣", "满减叠加", "直播带货"],
        ),
        promo_phase(
            "续航期",
            5,
            budget,
            0.20,
            "长尾转化",
            &["返场优惠", "老客复购"],
        ),
        promo_phase(
            "复盘期",
            2,
            budget,
            0.10,
            "数据复盘",
            &["ROI分析", "用户反馈收集"],
        ),
    ];
    let target_gmv = round2(budget * 5.0);
    json!({
        "has_data": true,
        "数据来源": "用户输入 + 确定性预算模型",
        "商品": product,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "平台": platform,
        "总预算": currency(budget),
        "活动阶段": phases,
        "KPI": {
            "目标GMV": currency(target_gmv),
            "目标ROI": round1(target_gmv / budget.max(1.0)),
            "预估UV": (budget * 2.0).round() as i64,
            "预估转化率": "3.5%",
            "预算消耗节奏": "预热15% / 爆发55% / 续航20% / 复盘10%",
        },
        "风险规则": [
            "爆发期首日ROI低于目标70%时，暂停低转化素材并转投高点击素材",
            "库存可售天数低于7天时，下调券面力度并切换为预约/预售",
            "退款率连续2天高于8%时，停止扩大流量并排查商品承诺"
        ],
    })
}

pub(super) fn channel_strategy(input: &Value) -> Value {
    let product_type = string(input, "product_type", "通用");
    let product_ref = string(input, "product_ref", "");
    let budget = num(input, "budget", 10_000.0).max(0.0);
    let audience = string(input, "target_audience", "泛兴趣人群");
    let weights = if product_type.contains("美妆") || product_type.contains("服饰") {
        vec![
            ("抖音/快手短视频", 0.35, "种草与转化"),
            ("小红书/内容种草", 0.25, "信任背书"),
            ("淘宝/京东搜索", 0.25, "承接成交"),
            ("私域复购", 0.15, "老客激活"),
        ]
    } else if product_type.contains("数码") || product_type.contains("家电") {
        vec![
            ("淘宝/京东搜索", 0.40, "高意向成交"),
            ("测评内容", 0.25, "参数解释"),
            ("抖音信息流", 0.20, "新品曝光"),
            ("私域复购", 0.15, "配件和延保"),
        ]
    } else {
        vec![
            ("平台搜索", 0.35, "成交承接"),
            ("短视频信息流", 0.30, "需求激发"),
            ("内容种草", 0.20, "信任建设"),
            ("私域/会员", 0.15, "复购维护"),
        ]
    };
    let mix: Vec<_> = weights
        .iter()
        .map(|(channel, weight, role)| {
            json!({
                "渠道": channel,
                "预算": currency(round2(budget * weight)),
                "占比": percent(weight * 100.0, 0),
                "角色": role,
                "核心KPI": if channel.contains("搜索") { "ROI/转化率" } else if channel.contains("私域") { "复购率/客单价" } else { "CTR/加购率" },
            })
        })
        .collect();
    json!({
        "has_data": true,
        "数据来源": "用户输入 + 类目渠道规则",
        "商品类型": product_type,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "目标人群": audience,
        "月预算": currency(budget),
        "渠道组合": mix,
        "内容节奏": [
            {"周期": "第1周", "重点": "人群测试", "动作": "每渠道至少3组素材，筛CTR和加购率"},
            {"周期": "第2-3周", "重点": "预算放大", "动作": "将预算转向ROI前40%的渠道/素材"},
            {"周期": "第4周", "重点": "复盘沉淀", "动作": "沉淀关键词、素材脚本和复购触达包"}
        ],
    })
}

pub(super) fn inventory_planning(input: &Value) -> Value {
    let avg_demand = num(input, "avg_daily_demand", 100.0);
    let lead_time = num(input, "lead_time_days", 7.0).max(0.0);
    let product_ref = string(input, "product_ref", "");
    let demand_std = num(input, "demand_std", avg_demand * 0.2).max(0.0);
    let service_level = num(input, "service_level", 0.95);
    let z = z_score(service_level);
    let safety_stock = (z * demand_std * lead_time.sqrt()).round();
    let reorder_point = (avg_demand * lead_time + safety_stock).round();
    let eoq = (2.0 * avg_demand * 365.0 * 50.0 / 2.0).sqrt().round();
    let turnover_days = if avg_demand > 0.0 {
        round1(eoq / avg_demand)
    } else {
        0.0
    };

    json!({
        "日均销量": avg_demand,
        "供货周期": format!("{}天", lead_time.round()),
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "安全库存": safety_stock,
        "补货点": reorder_point,
        "建议订货量": eoq,
        "预计周转天数": turnover_days,
        "月度采购预算参考": (avg_demand * 30.0).round(),
        "风险提示": if turnover_days > 30.0 { "库存周转 > 30天建议优化供应链" } else { "库存周转正常" },
    })
}

pub(super) fn listing_copy(input: &Value) -> Value {
    let product_name = string(input, "product_name", "");
    let product_ref = string(input, "product_ref", "");
    if product_name.is_empty() && product_ref.is_empty() && input.get("product_id").is_none() {
        return json!({
            "has_data": false,
            "提示": "请提供 product_name 或 product_ref；纯计算版本不会按商品引用自动读取数据库信息。",
        });
    }
    let name = if product_name.is_empty() {
        if product_ref.is_empty() {
            format!("商品ID {}", num(input, "product_id", 0.0).round() as i64)
        } else {
            product_ref.clone()
        }
    } else {
        product_name
    };
    let platform = string(input, "platform", "淘宝");
    let mut features = strings(input, "features");
    if features.is_empty() {
        features = vec![
            "高性价比".to_owned(),
            "品质稳定".to_owned(),
            "适合日常使用".to_owned(),
        ];
    }
    let keywords = listing_keywords(&name, &features, &platform);
    json!({
        "has_data": true,
        "数据来源": "用户输入 + 确定性文案模板",
        "平台": platform,
        "商品": name,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "标题版本": [
            format!("{} {} 官方同款 高性价比", name, features[0]),
            format!("{} {} 新品热卖 现货速发", name, features.join(" ")),
            format!("{} 旗舰品质 家用/送礼优选", name),
        ],
        "五点描述": features.iter().take(5).enumerate().map(|(idx, f)| {
            json!({"序号": idx + 1, "卖点": f, "表达": format!("突出「{}」，降低决策成本并强化使用场景", f)})
        }).collect::<Vec<_>>(),
        "关键词矩阵": keywords,
        "常见问答": [
            {"问": "适合什么场景？", "答": "适合日常使用、礼赠和新手入门，具体以商品规格为准。"},
            {"问": "发货和售后如何保障？", "答": "建议在详情页明确发货时效、退换规则和质保承诺。"}
        ],
    })
}

pub(super) fn assortment_planning(input: &Value) -> Value {
    let category = string(input, "category", "通用");
    let product_ref = string(input, "product_ref", "");
    let budget = num(input, "budget", 50_000.0).max(0.0);
    let hero = round2(budget * 0.45);
    let test = round2(budget * 0.30);
    let profit = round2(budget * 0.15);
    let clearance = round2(budget * 0.10);
    json!({
        "has_data": true,
        "数据来源": "用户输入 + 确定性选品组合模型",
        "类目": category,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "选品预算": currency(budget),
        "商品组合": [
            {"角色": "引流款", "预算": currency(hero), "SKU数": 2, "目标": "获取曝光与搜索权重", "毛利要求": "15%-25%"},
            {"角色": "测品款", "预算": currency(test), "SKU数": 4, "目标": "验证卖点/价格带", "毛利要求": "25%-35%"},
            {"角色": "利润款", "预算": currency(profit), "SKU数": 2, "目标": "承接复购和套装", "毛利要求": "35%+"},
            {"角色": "清仓/备用", "预算": currency(clearance), "SKU数": 1, "目标": "应对活动和库存风险", "毛利要求": "不低于现金成本"}
        ],
        "测品规则": {
            "单SKU首批采购": currency(round2(test / 4.0)),
            "放量门槛": "7天点击率>行业均值且转化率>2.5%",
            "淘汰门槛": "连续7天无加购或退款率>10%",
        },
        "风险提醒": ["避免同价格带SKU过密", "首批采购不超过预算30%", "优先选择可补货周期小于14天的供应商"],
    })
}

pub(super) fn execution_plan(input: &Value) -> Value {
    let goal = string(input, "goal", "提升GMV");
    let stage = string(input, "stage", "成长期");
    let product_ref = string(input, "product_ref", "");
    json!({
        "has_data": true,
        "数据来源": "用户输入 + 确定性执行框架",
        "目标": goal,
        "阶段": stage,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "30天执行计划": [
            {"周期": "第1周", "主题": "诊断与基线", "行动": ["确认GMV/转化/ROI基线", "梳理主推SKU和库存", "搭建日更看板"], "交付物": "基线表+问题清单"},
            {"周期": "第2周", "主题": "素材与流量测试", "行动": ["上线3组价格/利益点测试", "投放小预算渠道实验", "优化标题和详情首屏"], "交付物": "测试结论+预算调整表"},
            {"周期": "第3周", "主题": "放量与转化", "行动": ["放大ROI达标渠道", "配置优惠券/满减", "跟进客服转化话术"], "交付物": "放量计划+库存预警"},
            {"周期": "第4周", "主题": "复盘与固化", "行动": ["复盘活动ROI", "固化高转化素材", "制定下月预算"], "交付物": "复盘报告+下月计划"}
        ],
        "AARRR指标": {
            "Acquisition": "UV、CTR、渠道ROI",
            "Activation": "加购率、收藏率、详情页停留",
            "Retention": "复购率、会员触达率",
            "Revenue": "GMV、客单价、净利率",
            "Referral": "评价率、晒单率、内容转发"
        },
        "风险预案": [
            "ROI低于目标70%连续2天：收缩预算并回滚素材",
            "库存低于7天销量：暂停大券并切换预售",
            "退款率高于8%：暂停放量并排查品控/承诺"
        ],
    })
}
