fn pricing_strategy(input: &Value) -> Value {
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

fn promo_planning(input: &Value) -> Value {
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

fn channel_strategy(input: &Value) -> Value {
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

fn inventory_planning(input: &Value) -> Value {
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

fn listing_copy(input: &Value) -> Value {
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

fn assortment_planning(input: &Value) -> Value {
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

fn execution_plan(input: &Value) -> Value {
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
