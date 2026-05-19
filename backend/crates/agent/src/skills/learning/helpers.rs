use serde_json::{Value, json};

use super::profiles::TopicProfile;

pub(super) fn infer_learning_level(topic: &str) -> &'static str {
    if contains_any(topic, &["本科", "undergraduate"]) {
        "undergraduate"
    } else if contains_any(topic, &["新手", "小白", "入门", "beginner"]) {
        "beginner"
    } else if contains_any(topic, &["专家", "深入", "高级", "expert"]) {
        "expert"
    } else {
        "practitioner"
    }
}

pub(super) fn infer_scenario(topic: &str) -> &'static str {
    if contains_any(topic, &["投放", "广告", "roi", "ROAS", "roas", "千川"]) {
        "电商投放复盘"
    } else if contains_any(topic, &["客服", "售后", "退款", "DSR", "nps", "NPS"]) {
        "客服售后分析"
    } else if contains_any(topic, &["转化", "详情页", "标题", "主图", "点击率", "CTR"])
    {
        "商品转化优化"
    } else {
        "电商经营学习"
    }
}

pub(super) fn scenario_example(scenario: &str, product_bound: bool) -> String {
    let prefix = if product_bound {
        "以本轮显式选择的商品为例"
    } else {
        "以一个普通电商商品为例"
    };
    match scenario {
        "电商投放复盘" => format!(
            "{prefix}：如果广告花费1000元带来4000元成交额，ROAS是4；但还要扣除货品成本、优惠、\
             退款和物流，才能判断ROI是否健康。"
        ),
        "客服售后分析" => format!(
            "{prefix}：如果满意度下降，先拆解响应速度、解决率、退款原因和平台规则，\
             再决定是改话术、补偿还是升级处理。"
        ),
        "商品转化优化" => format!(
            "{prefix}：如果点击率高但付款率低，说明用户被入口吸引了，但详情页、价格、\
             评价或信任承接可能没有打通。"
        ),
        _ => format!(
            "{prefix}：先把概念对应到一个指标，再找影响这个指标的岗位动作，最后用一次小实验验证。"
        ),
    }
}

pub(super) fn self_check_question(topic: &str, profile: &TopicProfile) -> Value {
    json!({
        "题目": format!("请用自己的话说明：{topic}在电商经营里解决什么问题？"),
        "合格答案要点": [
            "能说清概念定义",
            "能说清适用场景",
            "能指出至少一个常见误区",
            "能给出一个可验证动作"
        ],
        "参考提示": profile.how.first().cloned().unwrap_or_else(|| "先从定义、场景和动作三步回答。".to_owned())
    })
}

pub(super) fn next_practice(scenario: &str) -> Vec<String> {
    match scenario {
        "电商投放复盘" => vec![
            "拿一条投放计划，分别计算ROAS和粗略ROI。".to_owned(),
            "写出ROAS高但ROI低时最可能的3个原因。".to_owned(),
            "给出一个明天可验证的小调整。".to_owned(),
        ],
        "客服售后分析" => vec![
            "选3条真实售后消息，标注客户情绪和核心诉求。".to_owned(),
            "把处理动作分成可立即解决、需升级、需平台规则确认三类。".to_owned(),
            "复盘哪类问题最影响评分。".to_owned(),
        ],
        "商品转化优化" => vec![
            "拆一条商品链路的曝光、点击、加购、下单、付款。".to_owned(),
            "找掉点最大的一环，只设计一个变量的测试。".to_owned(),
            "记录测试前后的指标变化。".to_owned(),
        ],
        _ => vec![
            "把本次概念写成一句白话定义。".to_owned(),
            "找一个业务场景说明它为什么重要。".to_owned(),
            "设计一个能在一天内验证的小动作。".to_owned(),
        ],
    }
}

pub(super) fn normalized_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}
