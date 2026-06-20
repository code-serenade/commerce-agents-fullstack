use serde_json::{Value, json};

use super::common::{
    average, commission_rate, currency, cv_xyz, elasticity_label, matrix_cells, matrix_strategy,
    num, nums, pct, percent, price_position, round2, signed_currency, signed_percent, string,
    val_num, value_nums,
};

pub(super) fn smart_pricing(input: &Value) -> Value {
    let cost = num(input, "cost", 0.0);
    let current_price = num(input, "current_price", 0.0);
    if cost <= 0.0 || current_price <= 0.0 {
        return json!({"error": "请提供有效的成本和当前售价"});
    }

    let current_sales = num(input, "current_monthly_sales", 500.0).max(1.0);
    let competitors = nums(input, "competitor_prices");
    let platform = string(input, "platform", "淘宝");
    let product_ref = string(input, "product_ref", "");
    let elasticity = num(input, "elasticity", -1.5);
    let stock_days = num(input, "stock_days", 0.0);
    let commission_rate = commission_rate(&platform);
    let current_profit_per_unit = current_price - cost - current_price * commission_rate;
    let current_margin = pct(current_profit_per_unit, current_price);
    let current_monthly_profit = current_profit_per_unit * current_sales;

    let effective_cost = cost + current_price * commission_rate;
    let lerner_optimal = if elasticity < -1.0 {
        effective_cost * elasticity / (elasticity + 1.0)
    } else {
        effective_cost * 2.0
    };

    let (avg_comp, min_comp, max_comp, price_floor, price_ceiling) = if competitors.is_empty() {
        (
            current_price,
            current_price * 0.8,
            current_price * 1.3,
            cost * 1.15,
            current_price * 1.4,
        )
    } else {
        let avg = average(&competitors);
        let min = competitors
            .iter()
            .copied()
            .reduce(f64::min)
            .unwrap_or(current_price);
        let max = competitors
            .iter()
            .copied()
            .reduce(f64::max)
            .unwrap_or(current_price);
        (avg, min, max, (cost * 1.1).max(min * 0.9), max * 1.2)
    };

    let stock_pressure_price = if stock_days > 90.0 {
        let discount = if stock_days <= 120.0 {
            0.05
        } else if stock_days <= 180.0 {
            0.10
        } else {
            0.15
        };
        Some(round2(price_floor.max(current_price * (1.0 - discount))))
    } else {
        None
    };

    let mut price_points = vec![
        (current_price * 0.90).max(price_floor),
        (current_price * 0.95).max(price_floor),
        current_price,
        (current_price * 1.05).min(price_ceiling),
        (current_price * 1.10).min(price_ceiling),
        lerner_optimal.max(price_floor).min(price_ceiling),
    ];
    if let Some(p) = stock_pressure_price {
        price_points.push(p);
    }
    price_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    price_points.dedup_by(|a, b| (*a - *b).abs() < 0.01);

    let mut candidates = Vec::new();
    let mut best_price = current_price;
    let mut best_profit = f64::NEG_INFINITY;
    for p in price_points {
        let demand_ratio = (p / current_price).powf(elasticity);
        let projected_sales = (current_sales * demand_ratio).round().max(1.0);
        let unit_profit = p - cost - p * commission_rate;
        let monthly_profit = unit_profit * projected_sales;
        let uplift_pct = (monthly_profit - current_monthly_profit)
            / current_monthly_profit.abs().max(1.0)
            * 100.0;
        let mut entry = json!({
            "定价": round2(p),
            "预估月销量": projected_sales as i64,
            "单品利润": round2(unit_profit),
            "利润率": percent(pct(unit_profit, p), 2),
            "月度总利润": round2(monthly_profit),
            "vs当前利润": signed_percent(uplift_pct, 1),
            "竞争定位": price_position(p, avg_comp),
        });
        if let Some(sp) = stock_pressure_price
            && (round2(p) - sp).abs() < 0.01
        {
            entry["标签"] = json!(format!("库存压力价（{}天库存）", stock_days.round()));
        }
        if monthly_profit > best_profit {
            best_profit = monthly_profit;
            best_price = round2(p);
        }
        candidates.push(entry);
    }
    candidates.sort_by(|a, b| {
        b["月度总利润"]
            .as_f64()
            .partial_cmp(&a["月度总利润"].as_f64())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let price_gap_vs_comp = pct(current_price - avg_comp, avg_comp);
    let mut result = json!({
        "当前经营状态": {
            "成本": currency(cost),
            "当前售价": currency(current_price),
            "当前利润率": percent(current_margin, 2),
            "当前月利润": currency(current_monthly_profit),
            "市场定位": if price_gap_vs_comp < -10.0 { "低价位" } else if price_gap_vs_comp > 10.0 { "高价位" } else { "中价位" },
            "vs竞品均价": signed_percent(price_gap_vs_comp, 1),
        },
        "价格弹性": {
            "弹性系数": elasticity,
            "弹性类型": elasticity_label(elasticity),
            "理论最优价格(Lerner)": currency(lerner_optimal.max(price_floor).min(price_ceiling)),
        },
        "定价方案对比": candidates,
        "推荐定价": currency(best_price),
        "推荐理由": format!("在当前弹性({elasticity})和竞品约束下，{}预期月利润最大化", currency(best_price)),
        "竞品价格区间": if competitors.is_empty() { json!("无竞品数据") } else { json!({"最低": currency(min_comp), "均价": currency(avg_comp), "最高": currency(max_comp)}) },
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
    });

    if let Some(sp) = stock_pressure_price {
        result["库存压力告警"] = json!({
            "当前库存天数": stock_days.round(),
            "风险等级": if stock_days > 180.0 { "高" } else if stock_days > 120.0 { "中" } else { "低" },
            "建议清仓价": currency(sp),
            "说明": format!("库存周转超过{}天，建议优先考虑库存压力价{}加速去库", stock_days.round(), currency(sp)),
        });
    }
    result
}

pub(super) fn inventory_optimizer(input: &Value) -> Value {
    let skus = input["skus"].as_array().cloned().unwrap_or_default();
    let product_ref = string(input, "product_ref", "");
    if skus.is_empty() {
        return json!({
            "has_data": false,
            "提示": "请提供 skus 参数；纯计算版本不会自动读取店铺数据。",
            "示例参数": {"skus": [{"name": "SKU1", "monthly_sales": 300, "cost": 40, "stock_qty": 500}]}
        });
    }

    let ordering_cost = num(input, "ordering_cost", 200.0).max(0.01);
    let holding_rate = num(input, "holding_rate", 25.0) / 100.0;
    let lead_time = num(input, "lead_time_days", 14.0).max(1.0);
    let service_level = string(input, "service_level", "95%");
    let z = match service_level.as_str() {
        "90%" => 1.282,
        "98%" => 2.054,
        "99%" => 2.326,
        _ => 1.645,
    };

    let mut sku_results = Vec::new();
    let mut gmv_list = Vec::new();
    let mut total_inv_value = 0.0;
    let mut annual_sales_cost = 0.0;
    for (idx, sku) in skus.iter().enumerate() {
        let name = sku["name"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("SKU{}", idx + 1));
        let monthly_sales = val_num(sku, "monthly_sales", 1.0).max(1.0);
        let unit_cost = val_num(sku, "cost", 1.0).max(0.01);
        let stock_qty = val_num(sku, "stock_qty", 0.0).max(0.0);
        let daily_demand = monthly_sales / 30.0;
        let annual_demand = monthly_sales * 12.0;
        let annual_holding_cost = (unit_cost * holding_rate).max(0.01);
        let eoq = (2.0 * annual_demand * ordering_cost / annual_holding_cost)
            .sqrt()
            .round()
            .max(1.0);
        let safety_stock = (z * daily_demand * 0.30 * lead_time.sqrt())
            .round()
            .max(1.0);
        let rop = (daily_demand * lead_time + safety_stock).round();
        let optimal_cycle = (eoq / daily_demand.max(0.01)).round();
        let current_stock_days = (stock_qty / daily_demand.max(0.01)).round();
        let inv_value = round2(stock_qty * unit_cost);
        total_inv_value += inv_value;
        annual_sales_cost += monthly_sales * unit_cost * 12.0;
        let health = if current_stock_days < lead_time {
            "缺货风险"
        } else if current_stock_days < lead_time + 7.0 {
            "临近再订货点"
        } else if current_stock_days > 90.0 {
            "库存积压"
        } else if current_stock_days > 60.0 {
            "偏高"
        } else {
            "健康"
        };
        sku_results.push(json!({
            "SKU": name,
            "月销量": monthly_sales.round(),
            "单位成本": currency(unit_cost),
            "当前库存": stock_qty.round(),
            "当前库存天数": format!("{}天", current_stock_days),
            "库存健康": health,
            "库存价值": currency(inv_value),
            "EOQ经济订货量": format!("{}件", eoq),
            "安全库存": format!("{}件（服务水平{}）", safety_stock, service_level),
            "再订货点(ROP)": format!("{}件", rop),
            "最优订货周期": format!("每{}天补货一次", optimal_cycle),
            "建议": match health {
                "缺货风险" => "立即补货",
                "临近再订货点" => "准备下单",
                "库存积压" => "清库促销",
                _ => "正常维护",
            }
        }));
        gmv_list.push((name, monthly_sales * unit_cost * 2.5));
    }

    gmv_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let total_gmv: f64 = gmv_list.iter().map(|(_, g)| *g).sum();
    let mut cumulative = 0.0;
    let mut abc_result = Vec::new();
    let mut counts = (0, 0, 0);
    for (name, gmv) in &gmv_list {
        cumulative += *gmv / total_gmv.max(1.0) * 100.0;
        let class = if cumulative <= 80.0 {
            counts.0 += 1;
            "A"
        } else if cumulative <= 95.0 {
            counts.1 += 1;
            "B"
        } else {
            counts.2 += 1;
            "C"
        };
        abc_result.push(json!({
            "SKU": name,
            "月GMV估算": currency(*gmv),
            "分类": class,
            "累计GMV占比": percent(cumulative.min(100.0), 1),
        }));
    }

    let turnover_ratio = round2(annual_sales_cost / total_inv_value.max(1.0));
    let turnover_days = (365.0 / turnover_ratio.max(0.01)).round();
    json!({
        "has_data": true,
        "数据来源": "用户提供",
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "服务水平": service_level,
        "供应商交货期": format!("{}天", lead_time.round()),
        "库存优化明细": sku_results,
        "ABC分类汇总": [
            {"分类": "A", "款式数": counts.0, "价值占比": 80, "策略": "重点保障，不允许缺货"},
            {"分类": "B", "款式数": counts.1, "价值占比": 15, "策略": "正常补货，EOQ控制"},
            {"分类": "C", "款式数": counts.2, "价值占比": 5, "策略": "缩减库存，清库促销"},
        ],
        "ABC分类明细": abc_result.into_iter().take(10).collect::<Vec<_>>(),
        "整体库存健康": {
            "总库存价值": currency(total_inv_value),
            "库存周转率": format!("{turnover_ratio}次/年"),
            "库存周转天数": format!("{turnover_days}天"),
            "健康状态": if turnover_days < 30.0 { "优秀" } else if turnover_days < 60.0 { "正常" } else { "偏高" },
            "行业基准": "电商库存周转<30天为优秀",
        }
    })
}

pub(super) fn ad_fatigue_detector(input: &Value) -> Value {
    let channels = input["channels"].as_array().cloned().unwrap_or_default();
    let product_ref = string(input, "product_ref", "");
    if channels.is_empty() {
        return json!({
            "has_data": false,
            "提示": "请提供 channels 参数；纯计算版本不会自动读取广告数据。",
            "示例参数": {"channels": [{"name": "直通车", "ctr_baseline": 3.2, "ctr_recent": 2.4, "impressions_per_user": 6, "budget": 5000, "roas": 3.2}]}
        });
    }
    let platform = string(input, "platform", "淘宝");
    let mut total_budget = num(input, "total_budget", 0.0);
    if total_budget == 0.0 {
        total_budget = channels.iter().map(|c| val_num(c, "budget", 0.0)).sum();
    }

    let mut channel_results = Vec::new();
    let mut raw_scores = Vec::new();
    let mut fatigued_count = 0;
    for ch in &channels {
        let name = ch["name"].as_str().unwrap_or("未知渠道");
        let ctr_base = val_num(ch, "ctr_baseline", 1.0);
        let ctr_recent = val_num(ch, "ctr_recent", 1.0);
        let freq = val_num(ch, "impressions_per_user", 0.0);
        let budget = val_num(ch, "budget", 0.0);
        let roas = val_num(ch, "roas", 0.0);
        let decay_rate = if ctr_base > 0.0 {
            (ctr_base - ctr_recent) / ctr_base * 100.0
        } else {
            0.0
        };
        let mut fatigue_score = 0.0;
        if decay_rate > 15.0 {
            fatigue_score += (decay_rate * 2.0).min(50.0);
        }
        if freq >= 8.0 {
            fatigue_score += ((freq - 7.0) * 10.0).min(30.0);
        }
        if decay_rate > 25.0 {
            fatigue_score += 20.0;
        }
        fatigue_score = fatigue_score.min(100.0).round();
        let status = if fatigue_score >= 70.0 {
            fatigued_count += 1;
            "严重疲劳（立即换素材）"
        } else if fatigue_score >= 40.0 {
            fatigued_count += 1;
            "中度疲劳（本周更新创意）"
        } else if fatigue_score >= 15.0 {
            "轻微疲劳（持续监控）"
        } else {
            "正常"
        };
        let roas_health = if roas == 0.0 {
            "未知"
        } else if roas >= 4.0 {
            "高效"
        } else if roas >= 2.0 {
            "正常"
        } else {
            "低效（建议削减预算）"
        };
        raw_scores.push(fatigue_score);
        channel_results.push(json!({
            "渠道": name,
            "基线CTR": percent(ctr_base, 2),
            "近期CTR": percent(ctr_recent, 2),
            "CTR衰减率": signed_percent(decay_rate, 1),
            "曝光频次": if freq > 0.0 { format!("{:.1}次/用户", freq) } else { "未知".to_owned() },
            "疲劳评分": format!("{}/100", fatigue_score as i64),
            "疲劳状态": status,
            "广告预算": if budget > 0.0 { currency(budget) } else { "未知".to_owned() },
            "ROAS": if roas > 0.0 { format!("{roas:.2}") } else { "未知".to_owned() },
            "ROAS健康": roas_health,
        }));
    }

    let mut result = json!({
        "数据来源": "用户提供",
        "总广告预算": if total_budget > 0.0 { currency(total_budget) } else { "未知".to_owned() },
        "平台": platform,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "行业疲劳基准": "CTR衰减>15%（连续7天）= 疲劳信号；频次≥8次/用户 = 受众疲劳",
        "渠道疲劳分析": channel_results,
        "疲劳渠道数": format!("{}/{}", fatigued_count, channels.len()),
    });

    if total_budget > 0.0 && channels.len() > 1 {
        let mut weights = Vec::new();
        for (idx, ch) in channels.iter().enumerate() {
            let roas = val_num(ch, "roas", 2.0).max(0.5);
            weights.push((roas * (1.0 - raw_scores[idx] / 150.0)).max(0.1));
        }
        let total_weight: f64 = weights.iter().sum();
        let mut reallocation = Vec::new();
        for (idx, ch) in channels.iter().enumerate() {
            let old_budget = val_num(ch, "budget", 0.0);
            let new_budget = (total_budget * weights[idx] / total_weight).round();
            reallocation.push(json!({
                "渠道": ch["name"].as_str().unwrap_or(""),
                "当前预算": currency(old_budget),
                "建议预算": currency(new_budget),
                "变化": signed_currency(new_budget - old_budget),
            }));
        }
        result["建议预算分配"] = json!(reallocation);
    }
    result
}

pub(super) fn abc_xyz_classifier(input: &Value) -> Value {
    let mut skus = input["skus"].as_array().cloned().unwrap_or_default();
    let product_ref = string(input, "product_ref", "");
    if skus.is_empty() {
        return json!({
            "has_data": false,
            "error": "需要提供 skus 才能执行 ABC-XYZ 库存分类",
            "required_input": "skus: [{sku_id, gmv, sales_history:[...]}]",
            "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        });
    }
    let platform = string(input, "platform", "淘宝");
    let top_n = num(input, "top_n", 20.0).round().max(1.0) as usize;

    let total_gmv: f64 = skus
        .iter()
        .map(|s| val_num(s, "gmv", 0.0))
        .sum::<f64>()
        .max(1.0);
    skus.sort_by(|a, b| {
        val_num(b, "gmv", 0.0)
            .partial_cmp(&val_num(a, "gmv", 0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut classified = Vec::new();
    let mut cumulative = 0.0;
    for sku in skus {
        let gmv = val_num(&sku, "gmv", 0.0);
        cumulative += gmv / total_gmv;
        let abc = if cumulative <= 0.80 {
            "A"
        } else if cumulative <= 0.95 {
            "B"
        } else {
            "C"
        };
        let history = value_nums(&sku["sales_history"]);
        let (cv, xyz) = cv_xyz(&history);
        let sku_id = sku["sku_id"]
            .as_str()
            .or_else(|| sku["name"].as_str())
            .unwrap_or("SKU");
        classified.push(json!({
            "sku_id": sku_id,
            "gmv": gmv,
            "gmv_pct": gmv / total_gmv,
            "abc": abc,
            "xyz": xyz,
            "cv": cv,
        }));
    }

    let mut matrix = matrix_cells();
    for item in &classified {
        let cell = format!(
            "{}{}",
            item["abc"].as_str().unwrap_or("C"),
            item["xyz"].as_str().unwrap_or("Z")
        );
        if let Some((count, gmv)) = matrix.get_mut(&cell) {
            *count += 1;
            *gmv += item["gmv"].as_f64().unwrap_or(0.0);
        }
    }
    let matrix_display: serde_json::Map<String, Value> = matrix
        .iter()
        .filter(|(_, (count, _))| *count > 0)
        .map(|(cell, (count, gmv))| {
            (
                cell.clone(),
                json!({
                    "款式数量": count,
                    "GMV贡献": percent(*gmv / total_gmv * 100.0, 1),
                    "策略": matrix_strategy(cell).0,
                    "库存倍率": matrix_strategy(cell).1,
                }),
            )
        })
        .collect();

    let top_display: Vec<_> = classified
        .iter()
        .take(top_n)
        .map(|item| {
            let class = format!(
                "{}{}",
                item["abc"].as_str().unwrap_or("?"),
                item["xyz"].as_str().unwrap_or("?")
            );
            json!({
                "SKU": item["sku_id"],
                "GMV": currency(item["gmv"].as_f64().unwrap_or(0.0)),
                "GMV占比": percent(item["gmv_pct"].as_f64().unwrap_or(0.0) * 100.0, 1),
                "CV": item["cv"],
                "分类": class,
                "策略": matrix_strategy(&class).0,
            })
        })
        .collect();

    json!({
        "数据来源": "用户输入",
        "平台": platform,
        "商品引用": if product_ref.is_empty() { Value::Null } else { json!(product_ref) },
        "分析款式总数": classified.len(),
        "9格矩阵": matrix_display,
        "矩阵说明": "ABC: A=前80%GMV / B=80-95% / C=尾部; XYZ: X=CV<0.2稳定 / Y=CV0.2-0.5 / Z=CV>0.5高波动",
        format!("高价值款式(前{}个)", top_display.len()): top_display,
        "关键洞察": {
            "AX稳定高价值": matrix.get("AX").map(|v| v.0).unwrap_or(0),
            "AZ高价值高波动": matrix.get("AZ").map(|v| v.0).unwrap_or(0),
            "CZ建议清仓": matrix.get("CZ").map(|v| v.0).unwrap_or(0),
        }
    })
}
