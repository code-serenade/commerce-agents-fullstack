use serde_json::{Map, Value, json};

pub(super) fn keyword_layers(keywords: &[String]) -> Vec<Value> {
    let primary = keywords
        .first()
        .cloned()
        .unwrap_or_else(|| "商品".to_owned());
    vec![
        json!({"层级": "主关键词", "关键词": primary, "用途": "标题前半段和首屏"}),
        json!({"层级": "长尾关键词", "关键词": keywords.iter().skip(1).take(3).cloned().collect::<Vec<_>>(), "用途": "详情页模块和FAQ"}),
        json!({"层级": "场景关键词", "关键词": ["家用", "送礼", "通勤", "新品"], "用途": "素材和内容种草"}),
    ]
}

pub(super) fn title_rules(platform: &str) -> Value {
    match platform {
        "拼多多" => {
            json!({"推荐字数": "20-28字", "重点": "价格权益、包邮、场景词", "避免": "过度品牌化"})
        }
        "抖音" => {
            json!({"推荐字数": "15-22字", "重点": "短利益点、场景、人群", "避免": "长标题堆词"})
        }
        "京东" => {
            json!({"推荐字数": "28-36字", "重点": "品牌、型号、规格、服务", "避免": "缺少参数"})
        }
        _ => {
            json!({"推荐字数": "28-34字", "重点": "核心词靠前、卖点明确、场景补充", "避免": "无关热词堆砌"})
        }
    }
}

pub(super) fn conversion_findings(
    conv: f64,
    bounce: f64,
    stay: f64,
    benchmark: f64,
) -> Vec<String> {
    let mut findings = Vec::new();
    if conv < benchmark {
        findings.push("转化率低于基准，首屏利益点、价格承诺或信任背书可能不足".to_owned());
    }
    if bounce >= 60.0 {
        findings.push("跳出率偏高，入口承诺和落地页内容可能不一致".to_owned());
    }
    if stay < 30.0 {
        findings.push("停留时间偏短，页面首屏需要更快说明商品价值".to_owned());
    }
    if findings.is_empty() {
        findings.push("核心指标未见明显异常，适合做小步AB测试提升效率".to_owned());
    }
    findings
}

pub(super) fn conversion_actions(
    page_type: &str,
    conv: f64,
    bounce: f64,
    stay: f64,
    benchmark: f64,
) -> Vec<Value> {
    let mut actions = Vec::new();
    if conv < benchmark {
        actions.push(
            json!({"优先级": "P0", "动作": "重排首屏卖点和CTA", "目标": "提高首屏理解和加购点击"}),
        );
    }
    if bounce >= 60.0 {
        actions.push(json!({"优先级": "P0", "动作": "核对流量入口文案与页面承诺", "目标": "降低无效点击和跳出"}));
    }
    if stay < 30.0 {
        actions.push(json!({"优先级": "P1", "动作": "压缩首屏文字、增加场景图和对比图", "目标": "提升继续浏览率"}));
    }
    actions.push(json!({
        "优先级": "P1",
        "动作": match page_type {
            "商品详情" => "补充评价背书、参数表和FAQ",
            "购物车" => "强化优惠、凑单和库存提醒",
            "结算" => "减少表单阻力并突出支付安全",
            "列表页" => "优化筛选、排序和商品卡片卖点",
            _ => "明确主行动路径和分类入口",
        },
        "目标": "减少决策阻力"
    }));
    actions
}

pub(super) fn display_product(input: &Map<String, Value>) -> String {
    opt_str_field(input, "product_name")
        .filter(|value| !value.trim().is_empty())
        .or_else(|| i64_field(input, "product_id").map(|id| format!("商品ID {id}")))
        .unwrap_or_else(|| "主推商品".to_owned())
}

pub(super) fn str_field(input: &Map<String, Value>, key: &str, default: &str) -> String {
    opt_str_field(input, key).unwrap_or_else(|| default.to_owned())
}

pub(super) fn opt_str_field(input: &Map<String, Value>, key: &str) -> Option<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn string_vec_field(input: &Map<String, Value>, key: &str) -> Vec<String> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn i64_field(input: &Map<String, Value>, key: &str) -> Option<i64> {
    input.get(key).and_then(Value::as_i64)
}

pub(super) fn palette_name(tone: &str) -> &'static str {
    if tone.contains("高端") || tone.contains("奢华") {
        "深色底 + 金色强调 + 大留白"
    } else if tone.contains("年轻") || tone.contains("活力") {
        "高明度主色 + 暖色CTA + 轻量插画"
    } else if tone.contains("科技") {
        "蓝绿科技色 + 参数模块化排版"
    } else if tone.contains("国潮") {
        "低饱和红绿 + 节庆金色点缀"
    } else {
        "蓝绿可信配色 + 橙色行动提示"
    }
}

pub(super) fn keyword_position_score(title: &str, keyword: &str) -> i64 {
    let Some(byte_idx) = title.to_lowercase().find(&keyword.to_lowercase()) else {
        return -20;
    };
    let char_idx = title[.. byte_idx].chars().count() as f64;
    let relative = char_idx / title.chars().count().max(1) as f64;
    if relative < 0.20 {
        30
    } else if relative < 0.40 {
        20
    } else if relative < 0.60 {
        10
    } else if relative < 0.80 {
        5
    } else {
        0
    }
}

pub(super) fn repeated_char_penalty(title: &str) -> i64 {
    let mut last = None;
    let mut penalty = 0;
    for ch in title.chars() {
        if Some(ch) == last && !ch.is_ascii_digit() {
            penalty += 2;
        }
        last = Some(ch);
    }
    penalty
}

pub(super) fn seo_grade(total: i64) -> &'static str {
    if total >= 85 {
        "S (优秀)"
    } else if total >= 70 {
        "A (良好)"
    } else if total >= 55 {
        "B (及格)"
    } else if total >= 40 {
        "C (需改进)"
    } else {
        "D (较差)"
    }
}

pub(super) fn build_seo_suggestions(
    primary: &str,
    pos_bonus: i64,
    missing: &[String],
    len: i64,
    lo: i64,
    hi: i64,
) -> Vec<String> {
    let mut out = Vec::new();
    if !primary.is_empty() && pos_bonus < 20 {
        out.push(format!("将主关键词「{primary}」移至标题最前30%字符"));
    }
    if !missing.is_empty() {
        out.push(format!(
            "补充关键词: {}",
            missing
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if len < lo || len > hi {
        out.push("控制字符数在推荐范围内".to_owned());
    }
    if out.is_empty() {
        out.push("标题质量良好，维持现有结构".to_owned());
    }
    out
}
