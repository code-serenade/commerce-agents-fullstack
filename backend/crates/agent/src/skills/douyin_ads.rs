use serde_json::{Value, json};

use crate::skills::{
    SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillSpec,
};

pub fn specs() -> Vec<SkillSpec> {
    vec![
        spec(
            "douyin_ads_plan_structure",
            "千川计划结构",
            "根据目标、预算、阶段和商品上下文生成巨量千川计划结构、预算拆分、冷启动赛马和止损规则",
            vec![
                field(
                    "goal",
                    "string",
                    "投放目标，例如直播间成交、短视频引流、新品冷启动",
                    false,
                ),
                field("budget", "number", "投放总预算（元）", false),
                field("platform", "string", "平台，默认抖音", false),
                field(
                    "product_name",
                    "string",
                    "商品名称；仅来自本轮显式输入或显式选品",
                    false,
                ),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品",
                    false,
                ),
                field("stage", "string", "投放阶段：冷启动/测试/放量/稳定", false),
                field("days", "integer", "投放周期天数，默认7", false),
                field("target_roas", "number", "目标ROAS，默认2.5", false),
            ],
        ),
        spec(
            "douyin_ads_audience_package",
            "千川人群包设计",
            "根据商品、人群、类目和预算生成千川人群包矩阵、排除规则、再营销池和测试顺序",
            vec![
                field("target_audience", "string", "目标人群描述", false),
                field(
                    "product_name",
                    "string",
                    "商品名称；仅来自本轮显式输入或显式选品",
                    false,
                ),
                field("category", "string", "商品类目", false),
                field("budget", "number", "投放预算（元）", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品",
                    false,
                ),
                field("stage", "string", "投放阶段：冷启动/测试/放量/稳定", false),
            ],
        ),
        spec(
            "douyin_ads_budget_pacing",
            "千川预算节奏",
            "根据预算、消耗、GMV或ROAS计算日预算节奏、放量/收缩判断和ROI止损线",
            vec![
                field("budget", "number", "投放总预算（元）", false),
                field("days", "integer", "投放周期天数，默认7", false),
                field("spend", "number", "已消耗金额（元）", false),
                field("gmv", "number", "已产生GMV（元）", false),
                field("target_roas", "number", "目标ROAS，默认2.5", false),
                field(
                    "current_roas",
                    "number",
                    "当前ROAS；如果缺失则用GMV/消耗计算",
                    false,
                ),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品",
                    false,
                ),
            ],
        ),
        spec(
            "douyin_ads_live_funnel_diagnosis",
            "直播间投放漏斗诊断",
            "基于曝光、点击、进房、商品点击、下单、付款和GMV诊断直播间千川投放漏斗瓶颈",
            vec![
                field("impressions", "number", "曝光量", false),
                field("clicks", "number", "广告点击数", false),
                field("live_room_entries", "number", "直播间进房数", false),
                field("product_clicks", "number", "商品点击数", false),
                field("orders", "number", "下单数", false),
                field("payments", "number", "付款数", false),
                field("ad_spend", "number", "广告消耗（元）", false),
                field("gmv", "number", "成交GMV（元）", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品",
                    false,
                ),
            ],
        ),
    ]
}

pub async fn execute(
    skill_name: &str,
    input: Value,
    _context: &SkillContext,
) -> Result<SkillOutcome, SkillError> {
    let output = match skill_name {
        "douyin_ads_plan_structure" => plan_structure(&input),
        "douyin_ads_audience_package" => audience_package(&input),
        "douyin_ads_budget_pacing" => budget_pacing(&input),
        "douyin_ads_live_funnel_diagnosis" => live_funnel_diagnosis(&input),
        other => {
            return Err(SkillError::execution(format!(
                "unknown douyin ads skill: {other}"
            )));
        }
    };
    Ok(SkillOutcome::new(output))
}

fn plan_structure(input: &Value) -> Value {
    let goal = string(input, "goal", "直播间成交与商品转化");
    let budget = num(input, "budget", 10_000.0).max(0.0);
    let platform = string(input, "platform", "抖音");
    let product_name = string(input, "product_name", "未指定商品");
    let product_ref = string(input, "product_ref", "");
    let stage = string(input, "stage", "冷启动");
    let days = int(input, "days", 7).max(1);
    let target_roas = num(input, "target_roas", 2.5).max(0.1);
    let daily_budget = budget / days as f64;

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": if has_any(input, &["budget", "goal", "product_name", "product_ref", "target_roas"]) { "user_provided" } else { "calculated_from_defaults" },
        "summary": "千川计划结构已生成",
        "数据来源": "用户显式输入 + 新产品 Rust Agent 千川投放 deterministic skill；未直连巨量千川、抖音罗盘或广告后台实时数据",
        "平台": platform,
        "商品": product_name,
        "商品引用": empty_string_as_null(&product_ref),
        "投放目标": goal,
        "阶段": stage,
        "周期天数": days,
        "总预算": currency(budget),
        "日均预算": currency(daily_budget),
        "目标ROAS": round2(target_roas),
        "计划结构": [
            {
                "层级": "冷启动赛马",
                "预算占比": "30%",
                "建议预算": currency(budget * 0.30),
                "动作": ["拆5-8个小计划", "每计划绑定1个素材方向和1个人群假设", "先看点击率、进房率和首单成本"],
                "保留规则": format!("24小时内ROAS达到目标{}的70%以上或点击率明显优于均值", round2(target_roas))
            },
            {
                "层级": "优胜计划放量",
                "预算占比": "45%",
                "建议预算": currency(budget * 0.45),
                "动作": ["预算每次上调不超过30%", "保留同素材同人群稳定计划", "复制计划测试相邻出价"],
                "放量规则": "连续2个观察窗口转化成本稳定且退款/差评风险可控"
            },
            {
                "层级": "稳定收割",
                "预算占比": "20%",
                "建议预算": currency(budget * 0.20),
                "动作": ["沉淀高意向人群再营销", "直播间承接款和利润款分计划", "控制频次避免素材疲劳"],
                "止损规则": format!("ROAS连续2个观察窗口低于{}则降预算或暂停", round2(target_roas * 0.7))
            },
            {
                "层级": "复盘留样",
                "预算占比": "5%",
                "建议预算": currency(budget * 0.05),
                "动作": ["保留少量探索计划", "记录素材-人群-出价组合", "输出下一轮测试假设"],
                "复盘口径": "按计划、素材、人群包、直播间承接分别复盘"
            }
        ],
        "关键KPI": {
            "目标GMV": currency(budget * target_roas),
            "计划级ROAS止损线": round2(target_roas * 0.70),
            "素材疲劳检查": "CTR下降超过20%或频次持续升高时更换素材",
            "直播承接检查": "点击正常但成交弱时优先排查进房、停留、商品点击和逼单话术"
        },
        "风险提示": [
            "本结果不是千川后台读取结果，只能作为计划结构和预算规则建议",
            "放量前必须补齐消耗、GMV、点击、进房、商品点击、下单和付款指标",
            "平台规则、审核政策和行业基准需要通过 search/platform adapter 单独取得真实来源"
        ]
    })
}

fn audience_package(input: &Value) -> Value {
    let audience = string(input, "target_audience", "目标消费者");
    let product_name = string(input, "product_name", "未指定商品");
    let category = string(input, "category", "电商消费品");
    let budget = num(input, "budget", 8_000.0).max(0.0);
    let product_ref = string(input, "product_ref", "");
    let stage = string(input, "stage", "冷启动");

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": if has_any(input, &["target_audience", "product_name", "category", "budget", "product_ref"]) { "user_provided" } else { "calculated_from_defaults" },
        "summary": "千川人群包设计已生成",
        "数据来源": "用户显式输入 + 新产品 Rust Agent 千川投放 deterministic skill；未读取巨量千川DMP或广告后台人群数据",
        "商品": product_name,
        "商品引用": empty_string_as_null(&product_ref),
        "类目": category,
        "目标人群": audience,
        "阶段": stage,
        "预算": currency(budget),
        "人群包矩阵": [
            {
                "人群包": "核心转化人群",
                "预算占比": "35%",
                "建议预算": currency(budget * 0.35),
                "定向逻辑": format!("{category}强兴趣 + 近期互动/购买意向 + {audience}"),
                "观察指标": ["点击率", "进房率", "商品点击率", "支付ROI"],
                "动作": "优先跑转化目标，保留ROI稳定计划"
            },
            {
                "人群包": "相似扩展人群",
                "预算占比": "25%",
                "建议预算": currency(budget * 0.25),
                "定向逻辑": "基于成交/加购/直播互动种子做相似扩展",
                "观察指标": ["获客成本", "首单转化率", "新客占比"],
                "动作": "冷启动后承接放量，严格控制成本波动"
            },
            {
                "人群包": "内容兴趣人群",
                "预算占比": "25%",
                "建议预算": currency(budget * 0.25),
                "定向逻辑": "短视频互动、达人内容、类目兴趣、场景兴趣组合",
                "观察指标": ["3秒播放", "点击率", "进房成本"],
                "动作": "用于素材赛马和直播间引流，不直接按GMV单点判断"
            },
            {
                "人群包": "再营销人群",
                "预算占比": "15%",
                "建议预算": currency(budget * 0.15),
                "定向逻辑": "近7/15/30天互动、进房、商品点击、加购和未支付用户",
                "观察指标": ["支付转化率", "客单价", "ROI"],
                "动作": "配合限时权益和直播间逼单话术收割"
            }
        ],
        "排除规则": [
            "排除近期已购买但未到复购周期用户，避免浪费预算",
            "排除高频曝光无互动人群，降低素材疲劳",
            "排除投诉/退款高风险标签，避免放大售后风险"
        ],
        "测试顺序": [
            "先跑核心转化人群验证商品与直播间承接",
            "再跑内容兴趣人群验证素材点击和进房",
            "最后扩展相似人群并建立再营销收割池"
        ],
        "风险提示": [
            "本结果没有读取真实DMP人群包规模或覆盖率",
            "实际投放前需要补齐历史成交人群、直播互动人群和素材表现"
        ]
    })
}

fn budget_pacing(input: &Value) -> Value {
    let budget = optional_num(input, "budget");
    let spend = optional_num(input, "spend");
    let gmv = optional_num(input, "gmv");
    let current_roas = optional_num(input, "current_roas").or_else(|| match (gmv, spend) {
        (Some(gmv), Some(spend)) if spend > 0.0 => Some(gmv / spend),
        _ => None,
    });

    if budget.is_none() || current_roas.is_none() {
        return deferred(
            "douyin_ads_budget_pacing",
            "千川预算节奏需要投放总预算，并提供 current_roas，或同时提供 spend 与 gmv。",
            &[
                "budget",
                "current_roas 或 spend + gmv",
                "days",
                "target_roas",
            ],
        );
    }

    let budget = budget.unwrap_or_default().max(0.0);
    let days = int(input, "days", 7).max(1);
    let target_roas = num(input, "target_roas", 2.5).max(0.1);
    let current_roas = current_roas.unwrap_or_default().max(0.0);
    let spend = spend.unwrap_or(0.0).max(0.0);
    let gmv = gmv.unwrap_or(current_roas * spend).max(0.0);
    let product_ref = string(input, "product_ref", "");
    let remaining_budget = (budget - spend).max(0.0);
    let daily_budget = budget / days as f64;
    let remaining_daily_budget = remaining_budget / days as f64;
    let stop_line = target_roas * 0.70;
    let action = if current_roas >= target_roas * 1.15 {
        "放量"
    } else if current_roas >= target_roas * 0.90 {
        "稳投观察"
    } else if current_roas >= stop_line {
        "收缩优化"
    } else {
        "止损暂停"
    };

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": "user_provided",
        "summary": "千川预算节奏已计算",
        "数据来源": "用户显式输入 + 新产品 Rust Agent 千川投放 deterministic skill；未读取巨量千川实时消耗或转化数据",
        "商品引用": empty_string_as_null(&product_ref),
        "预算": {
            "总预算": currency(budget),
            "已消耗": currency(spend),
            "剩余预算": currency(remaining_budget),
            "周期天数": days,
            "理论日预算": currency(daily_budget),
            "剩余日均预算": currency(remaining_daily_budget)
        },
        "产出": {
            "GMV": currency(gmv),
            "当前ROAS": round2(current_roas),
            "目标ROAS": round2(target_roas),
            "止损线": round2(stop_line),
            "目标差距": signed_percent((current_roas / target_roas - 1.0) * 100.0, 1)
        },
        "动作判断": action,
        "预算动作": budget_action(action),
        "观察窗口": [
            "每30分钟观察点击率、进房率、商品点击率和付款转化",
            "每2-4小时观察ROAS、消耗速度和素材疲劳",
            "跨天保留稳定计划，不在短窗口内频繁重置学习"
        ],
        "风险提示": [
            "本结果只基于本轮显式输入，不代表已读取千川后台实时数据",
            "如果GMV未剔除退款、佣金、赠品和履约成本，ROAS不能等同利润ROI"
        ]
    })
}

fn live_funnel_diagnosis(input: &Value) -> Value {
    let Some(impressions) = optional_num(input, "impressions") else {
        return deferred(
            "douyin_ads_live_funnel_diagnosis",
            "直播间投放漏斗诊断至少需要 impressions 和 clicks。",
            &[
                "impressions",
                "clicks",
                "live_room_entries",
                "product_clicks",
                "orders",
                "payments",
                "ad_spend",
                "gmv",
            ],
        );
    };
    let Some(clicks) = optional_num(input, "clicks") else {
        return deferred(
            "douyin_ads_live_funnel_diagnosis",
            "直播间投放漏斗诊断至少需要 impressions 和 clicks。",
            &[
                "impressions",
                "clicks",
                "live_room_entries",
                "product_clicks",
                "orders",
                "payments",
                "ad_spend",
                "gmv",
            ],
        );
    };

    let live_room_entries = optional_num(input, "live_room_entries");
    let product_clicks = optional_num(input, "product_clicks");
    let orders = optional_num(input, "orders");
    let payments = optional_num(input, "payments");
    let ad_spend = optional_num(input, "ad_spend");
    let gmv = optional_num(input, "gmv");
    let product_ref = string(input, "product_ref", "");

    let ctr = rate(clicks, impressions);
    let entry_rate = live_room_entries.map(|value| rate(value, clicks));
    let product_click_rate = match (product_clicks, live_room_entries) {
        (Some(product_clicks), Some(entries)) => Some(rate(product_clicks, entries)),
        _ => None,
    };
    let order_rate = match (orders, product_clicks) {
        (Some(orders), Some(product_clicks)) => Some(rate(orders, product_clicks)),
        _ => None,
    };
    let pay_rate = match (payments, orders) {
        (Some(payments), Some(orders)) => Some(rate(payments, orders)),
        _ => None,
    };
    let roas = match (gmv, ad_spend) {
        (Some(gmv), Some(ad_spend)) if ad_spend > 0.0 => Some(gmv / ad_spend),
        _ => None,
    };
    let bottleneck = funnel_bottleneck(ctr, entry_rate, product_click_rate, order_rate, pay_rate);

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": if has_any(input, &["live_room_entries", "product_clicks", "orders", "payments", "ad_spend", "gmv"]) { "user_provided" } else { "partial_user_input" },
        "summary": "直播间投放漏斗诊断已生成",
        "数据来源": "用户显式输入 + 新产品 Rust Agent 千川投放 deterministic skill；未读取抖音直播间或千川后台实时漏斗数据",
        "商品引用": empty_string_as_null(&product_ref),
        "漏斗指标": {
            "曝光": round0(impressions),
            "点击": round0(clicks),
            "点击率": percent(ctr, 2),
            "进房数": live_room_entries.map(round0).map(Value::from).unwrap_or(Value::Null),
            "点击到进房率": entry_rate.map(|value| json!(percent(value, 2))).unwrap_or(Value::Null),
            "商品点击数": product_clicks.map(round0).map(Value::from).unwrap_or(Value::Null),
            "进房到商品点击率": product_click_rate.map(|value| json!(percent(value, 2))).unwrap_or(Value::Null),
            "下单数": orders.map(round0).map(Value::from).unwrap_or(Value::Null),
            "商品点击到下单率": order_rate.map(|value| json!(percent(value, 2))).unwrap_or(Value::Null),
            "付款数": payments.map(round0).map(Value::from).unwrap_or(Value::Null),
            "下单到付款率": pay_rate.map(|value| json!(percent(value, 2))).unwrap_or(Value::Null),
            "广告消耗": ad_spend.map(currency).map(Value::from).unwrap_or(Value::Null),
            "GMV": gmv.map(currency).map(Value::from).unwrap_or(Value::Null),
            "ROAS": roas.map(round2).map(Value::from).unwrap_or(Value::Null)
        },
        "主要瓶颈": bottleneck,
        "动作建议": funnel_actions(&bottleneck),
        "待补字段": missing_funnel_fields(input),
        "风险提示": [
            "只有曝光和点击时只能判断前链路点击效率，不能直接判断直播成交承接",
            "GMV和消耗口径未扣除退款、佣金和履约成本时，不能当作利润ROI"
        ]
    })
}

fn deferred(skill: &str, reason: &str, required_fields: &[&str]) -> Value {
    json!({
        "status": "deferred",
        "has_data": false,
        "adapter_status": "user_input_required",
        "data_readiness": "missing_user_input",
        "summary": reason,
        "skill": skill,
        "required_fields": required_fields,
        "说明": "当前不会伪造巨量千川、抖音罗盘、直播间或广告后台实时数据；请补充本轮显式指标，或后续接入平台 adapter。"
    })
}

fn budget_action(action: &str) -> Vec<&'static str> {
    match action {
        "放量" => vec![
            "保留原计划学习状态，单次加预算不超过30%",
            "复制优胜素材和人群包做相邻出价测试",
            "同步检查直播间库存、客服和优惠承接",
        ],
        "稳投观察" => vec![
            "维持预算，观察至少2个转化窗口",
            "优先优化低点击素材和低进房时段",
            "不要频繁重启计划，避免学习期反复重置",
        ],
        "收缩优化" => vec![
            "降低低效计划预算20%-30%",
            "保留高点击计划，替换转化弱素材或落地承接",
            "拆分人群包，排除高频无互动人群",
        ],
        _ => vec![
            "暂停低ROAS计划，保留少量复盘预算",
            "优先排查商品点击、下单和付款承接",
            "重新定义素材、人群和直播间话术假设后再测试",
        ],
    }
}

fn funnel_bottleneck(
    ctr: f64,
    entry_rate: Option<f64>,
    product_click_rate: Option<f64>,
    order_rate: Option<f64>,
    pay_rate: Option<f64>,
) -> String {
    if ctr < 1.0 {
        return "广告点击弱，优先排查素材钩子、定向匹配和出价竞争力".to_string();
    }
    if entry_rate.is_some_and(|value| value < 45.0) {
        return "点击到进房弱，优先排查直播间封面、标题、承诺一致性和加载承接".to_string();
    }
    if product_click_rate.is_some_and(|value| value < 12.0) {
        return "进房到商品点击弱，优先排查主播讲解、货品摆放、权益露出和商品卡点击引导"
            .to_string();
    }
    if order_rate.is_some_and(|value| value < 8.0) {
        return "商品点击到下单弱，优先排查价格锚点、优惠门槛、信任背书和库存规格".to_string();
    }
    if pay_rate.is_some_and(|value| value < 70.0) {
        return "下单到付款弱，优先排查支付激励、客服答疑、优惠锁单和犹豫点".to_string();
    }
    "漏斗未见单一极端瓶颈，建议按素材、人群包和直播间节奏继续分层A/B测试".to_string()
}

fn funnel_actions(bottleneck: &str) -> Vec<&'static str> {
    if bottleneck.contains("广告点击弱") {
        vec![
            "重做前3秒钩子",
            "按卖点拆素材赛马",
            "缩窄人群假设并提高素材相关性",
        ]
    } else if bottleneck.contains("点击到进房弱") {
        vec![
            "统一广告承诺和直播间首屏承接",
            "优化直播封面与标题",
            "检查投放时段和主播在线节奏",
        ]
    } else if bottleneck.contains("商品点击弱") {
        vec![
            "商品卡前置",
            "每15分钟重复核心卖点和权益",
            "用引流款承接进房用户",
        ]
    } else if bottleneck.contains("下单弱") {
        vec!["强化价格锚点", "缩短优惠路径", "补充真实评价和信任背书"]
    } else if bottleneck.contains("付款弱") {
        vec![
            "设置限时锁单权益",
            "客服及时处理支付疑虑",
            "降低优惠使用门槛",
        ]
    } else {
        vec![
            "继续拆素材、人群、时段做A/B",
            "保留稳定计划",
            "按ROAS和退款风险做预算迁移",
        ]
    }
}

fn missing_funnel_fields(input: &Value) -> Vec<&'static str> {
    [
        ("live_room_entries", "live_room_entries"),
        ("product_clicks", "product_clicks"),
        ("orders", "orders"),
        ("payments", "payments"),
        ("ad_spend", "ad_spend"),
        ("gmv", "gmv"),
    ]
    .into_iter()
    .filter_map(|(key, label)| (!input_has(input, key)).then_some(label))
    .collect()
}

fn spec(
    name: &str,
    display_name: &str,
    description: &str,
    input_fields: Vec<SkillInputField>,
) -> SkillSpec {
    let mut spec = SkillSpec::new(name, display_name, description)
        .with_category(SkillCategory::Custom("douyin_ads".to_owned()))
        .with_priority(SkillPriority::Critical)
        .with_tag("deterministic")
        .with_tag("p0")
        .with_version("0.1.0");
    for input in input_fields {
        spec = spec.with_input(input);
    }
    spec
}

fn field(name: &str, field_type: &str, description: &str, required: bool) -> SkillInputField {
    SkillInputField::new(name, description, required, field_schema(field_type))
}

fn field_schema(field_type: &str) -> Value {
    match field_type {
        "string" => json!({"type": "string"}),
        "number" => json!({"type": "number"}),
        "integer" => json!({"type": "integer"}),
        _ => json!({"type": "string"}),
    }
}

fn num(input: &Value, key: &str, default: f64) -> f64 {
    optional_num(input, key).unwrap_or(default)
}

fn optional_num(input: &Value, key: &str) -> Option<f64> {
    input[key]
        .as_f64()
        .or_else(|| input[key].as_i64().map(|value| value as f64))
        .or_else(|| input[key].as_u64().map(|value| value as f64))
        .filter(|value| value.is_finite())
}

fn int(input: &Value, key: &str, default: i64) -> i64 {
    input[key]
        .as_i64()
        .or_else(|| {
            input[key]
                .as_u64()
                .and_then(|value| i64::try_from(value).ok())
        })
        .or_else(|| optional_num(input, key).map(|value| value.round() as i64))
        .unwrap_or(default)
}

fn string(input: &Value, key: &str, default: &str) -> String {
    input[key]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
        .to_owned()
}

fn input_has(input: &Value, key: &str) -> bool {
    match &input[key] {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        _ => true,
    }
}

fn has_any(input: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| input_has(input, key))
}

fn empty_string_as_null(value: &str) -> Value {
    if value.trim().is_empty() {
        Value::Null
    } else {
        json!(value)
    }
}

fn rate(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() > f64::EPSILON {
        numerator / denominator * 100.0
    } else {
        0.0
    }
}

fn round0(value: f64) -> i64 {
    value.round().max(0.0) as i64
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn percent(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}%")
}

fn signed_percent(value: f64, decimals: usize) -> String {
    format!("{value:+.decimals$}%")
}

fn currency(value: f64) -> String {
    format!("¥{:.2}", round2(value))
}
