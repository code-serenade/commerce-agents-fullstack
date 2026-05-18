use async_trait::async_trait;
use serde_json::{Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillInputField, SkillOutcome, SkillPriority, SkillResult,
    SkillSpec, object_params, optional_bool_param, optional_string_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("learning".to_owned())
}

pub fn specs() -> Vec<SkillSpec> {
    vec![learning_teaching_explain_spec()]
}

pub fn learning_teaching_explain_spec() -> SkillSpec {
    SkillSpec::new(
        "learning_teaching_explain",
        "教学解释",
        "按教学模式解释电商概念、指标、策略或操作方法，输出30秒结论、为什么、怎么做、\
         常见误区和自检题。",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::required(
        "topic",
        "需要教学解释的主题，例如 ROI和ROAS的区别、DSR是什么、怎么理解转化率",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "learning_level",
        "教学层级：beginner / undergraduate / practitioner / expert",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "scenario",
        "业务场景，例如 电商投放复盘 / 商品详情页优化 / 客服售后分析",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "product_ref",
        "新产品商品引用，如 product:{uuid}；只有本轮显式选择商品时传入",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "product_name",
        "商品名称；只有本轮显式选择商品时传入",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/抖音/京东/拼多多/小红书等",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "include_self_check",
        "是否包含自检题，默认 true",
        json!({"type": "boolean"}),
    ))
    .with_tag("deterministic")
    .with_tag("teaching")
}

pub struct LearningTeachingExplain {
    spec: SkillSpec,
}

impl LearningTeachingExplain {
    pub fn new() -> Self {
        Self {
            spec: learning_teaching_explain_spec(),
        }
    }
}

impl Default for LearningTeachingExplain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for LearningTeachingExplain {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let topic = string_param(&params, "topic")?;
        let learning_level = optional_string_param(&params, "learning_level")?
            .unwrap_or_else(|| infer_learning_level(&topic).to_owned());
        let scenario = optional_string_param(&params, "scenario")?
            .unwrap_or_else(|| infer_scenario(&topic).to_owned());
        let product_ref =
            normalized_optional_string(optional_string_param(&params, "product_ref")?);
        let product_name =
            normalized_optional_string(optional_string_param(&params, "product_name")?);
        let platform = normalized_optional_string(optional_string_param(&params, "platform")?);
        let include_self_check =
            optional_bool_param(&params, "include_self_check")?.unwrap_or(true);

        let product_bound = product_ref.is_some() || product_name.is_some();
        let profile = topic_profile(&topic);
        let product_context = if product_bound {
            json!({
                "mode": "explicit_selected_product",
                "product_ref": product_ref,
                "product_name": product_name,
                "platform": platform,
                "note": "仅使用本轮显式选择的商品上下文；未读取历史商品或预设样例商品。"
            })
        } else {
            json!({
                "mode": "unselected",
                "note": "本轮未显式选择商品，以下为通用教学解释；不会注入历史商品、预设商品或演示商品。"
            })
        };

        let mut output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": if product_bound { "explicit_product_context" } else { "user_provided" },
            "summary": format!("教学解释已生成：{topic}"),
            "skill": self.spec.id,
            "教学主题": topic,
            "教学层级": learning_level,
            "业务场景": scenario,
            "商品上下文": product_context,
            "30秒结论": profile.conclusion,
            "术语白话解释": profile.terms,
            "为什么": profile.why,
            "怎么做": profile.how,
            "场景示例": scenario_example(&scenario, product_bound),
            "常见误区": profile.pitfalls,
            "下一步练习": next_practice(&scenario),
            "治理说明": [
                "本 skill 是新产品 Rust Agent native 教学能力，不调用外部平台、不读取实时店铺数据、不执行高风险动作。",
                "如果需要结合具体商品，必须由本轮显式选品或显式 contextRefs 提供商品上下文。",
                "如果用户要求最新规则、公开来源或实时数据，应另行触发 search_* adapter，不能把教学解释伪造成实时搜索。"
            ]
        });
        if include_self_check {
            output["自检问题"] = self_check_question(&topic, &profile);
        }

        Ok(SkillOutcome::new(output).with_summary(format!("教学解释已生成：{topic}")))
    }
}

struct TopicProfile {
    conclusion: String,
    terms: Vec<Value>,
    why: Vec<String>,
    how: Vec<String>,
    pitfalls: Vec<String>,
}

fn topic_profile(topic: &str) -> TopicProfile {
    let lower = topic.to_ascii_lowercase();
    if lower.contains("roi") || lower.contains("roas") || topic.contains("投产") {
        return TopicProfile {
            conclusion: "ROI更偏财务投入产出，ROAS更偏广告投放产出；看广告效果先看ROAS，\
                         看整体赚钱能力还要回到ROI和利润。"
                .to_owned(),
            terms: vec![
                json!({"术语": "ROI", "白话解释": "投了多少钱，最后整体赚回多少；通常要考虑成本、毛利、人工、优惠和退款。"}),
                json!({"术语": "ROAS", "白话解释": "广告花了多少钱，广告带来多少成交额；它更像投放账户的效率表。"}),
                json!({"术语": "边界", "白话解释": "ROAS高不一定赚钱，ROI低也可能是成本、退款或履约拖累。"}),
            ],
            why: vec![
                "投放人员常用ROAS判断广告账户能不能继续放量。".to_owned(),
                "经营负责人需要用ROI判断这门生意最终是否值得继续投入。".to_owned(),
                "两者分开看，才能避免只看成交额却忽略毛利和履约成本。".to_owned(),
            ],
            how: vec![
                "先算ROAS：广告成交额 / 广告花费，用来判断投放效率。".to_owned(),
                "再算ROI：利润或净收益 / 总投入，用来判断经营收益。".to_owned(),
                "最后做动作：ROAS低先查素材、人群、转化链路；ROAS高但ROI低则查毛利、优惠、\
                 退款和物流成本。"
                    .to_owned(),
            ],
            pitfalls: vec![
                "把ROAS当成利润，看到成交额高就盲目加预算。".to_owned(),
                "只看单日ROAS，不看退款、复购、客单价和毛利。".to_owned(),
                "不同商品毛利不同，却使用同一条放量线。".to_owned(),
            ],
        };
    }

    if lower.contains("ctr") || topic.contains("点击率") {
        return TopicProfile {
            conclusion: "CTR是用户看到内容后愿不愿意点进来的比例，主要反映标题、主图、\
                         素材和人群匹配度。"
                .to_owned(),
            terms: vec![
                json!({"术语": "CTR", "白话解释": "曝光100次，有多少次被点击。"}),
                json!({"术语": "曝光", "白话解释": "用户看到了你的商品、广告或内容。"}),
                json!({"术语": "点击", "白话解释": "用户对这个入口产生兴趣并进入下一步。"}),
            ],
            why: vec![
                "CTR低说明第一眼吸引力或人群匹配可能有问题。".to_owned(),
                "CTR影响后续转化漏斗的流量基础。".to_owned(),
                "在同预算下，CTR改善通常能降低获客成本。".to_owned(),
            ],
            how: vec![
                "先确认曝光来源和目标人群是否准确。".to_owned(),
                "再比较主图、标题、价格锚点和利益点。".to_owned(),
                "最后做A/B测试，只改一个变量，避免不知道是哪项生效。".to_owned(),
            ],
            pitfalls: vec![
                "只追求高点击，忽略点击后的转化和退款。".to_owned(),
                "把不同渠道、不同人群的CTR直接混在一起比较。".to_owned(),
                "一次改太多素材变量，无法复盘原因。".to_owned(),
            ],
        };
    }

    if lower.contains("cvr") || topic.contains("转化率") {
        return TopicProfile {
            conclusion: "转化率说明用户进入下一步后有多少人真正完成购买或目标动作，\
                         它反映的是承接能力。"
                .to_owned(),
            terms: vec![
                json!({"术语": "CVR", "白话解释": "进入页面或直播间的人里面，有多少人完成购买、加购或留资。"}),
                json!({"术语": "承接", "白话解释": "用户来了以后，页面、价格、信任、客服和促销能不能把他留下来。"}),
            ],
            why: vec![
                "转化率决定流量能否变成订单。".to_owned(),
                "同样的流量成本下，转化率越高，获客效率越好。".to_owned(),
                "转化率变化能帮助定位页面、价格、信任或库存问题。".to_owned(),
            ],
            how: vec![
                "先按曝光、点击、加购、下单、付款拆漏斗。".to_owned(),
                "找掉点最大的环节，再对应检查素材、详情页、优惠、评价和客服。".to_owned(),
                "用小流量实验验证，不要一次性重做全链路。".to_owned(),
            ],
            pitfalls: vec![
                "只看总转化率，不分新客老客、渠道和商品。".to_owned(),
                "把低转化都归因给流量差，忽略页面承接。".to_owned(),
                "只做降价促销，忽略信任、评价和卖点表达。".to_owned(),
            ],
        };
    }

    TopicProfile {
        conclusion: format!("{topic}要先讲清定义、适用场景、判断口径，再落到一个可验证的小动作。"),
        terms: vec![
            json!({"术语": topic, "白话解释": "先把抽象概念翻译成用户能观察、能计算、能行动的业务指标。"}),
            json!({"术语": "判断口径", "白话解释": "同一个词在不同岗位可能算法不同，必须先统一分母、分子和时间范围。"}),
        ],
        why: vec![
            "概念不清会导致岗位之间各说各话，后续动作难以验收。".to_owned(),
            "先明确口径，可以减少错误归因和无效优化。".to_owned(),
            "教学解释的目标不是背概念，而是把概念迁移到真实电商动作。".to_owned(),
        ],
        how: vec![
            "第一步：写出它是什么，以及它不是什么。".to_owned(),
            "第二步：说明它影响哪个业务环节。".to_owned(),
            "第三步：给出一个能在今天验证的小动作或小实验。".to_owned(),
        ],
        pitfalls: vec![
            "只记名词，不看适用前提。".to_owned(),
            "直接套用别人的结论，不结合自己的类目、渠道和商品阶段。".to_owned(),
            "没有验证动作，导致学习和经营结果脱节。".to_owned(),
        ],
    }
}

fn infer_learning_level(topic: &str) -> &'static str {
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

fn infer_scenario(topic: &str) -> &'static str {
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

fn scenario_example(scenario: &str, product_bound: bool) -> String {
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

fn self_check_question(topic: &str, profile: &TopicProfile) -> Value {
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

fn next_practice(scenario: &str) -> Vec<String> {
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

fn normalized_optional_string(value: Option<String>) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn teaching_explain_handles_roi_roas() {
        let result = LearningTeachingExplain::new()
            .run(
                json!({
                    "topic": "ROI和ROAS的区别",
                    "learning_level": "undergraduate",
                    "scenario": "电商投放复盘",
                    "include_self_check": true
                }),
                SkillContext::default(),
            )
            .await
            .expect("skill should run");

        assert_eq!(
            result.output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert!(
            result
                .output
                .get("30秒结论")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("ROI")
        );
        assert!(result.output.get("自检问题").is_some());
    }

    #[tokio::test]
    async fn teaching_explain_does_not_default_product_context() {
        let result = LearningTeachingExplain::new()
            .run(
                json!({
                    "topic": "怎么理解转化率",
                    "include_self_check": false
                }),
                SkillContext::default(),
            )
            .await
            .expect("skill should run");

        assert_eq!(
            result
                .output
                .pointer("/商品上下文/mode")
                .and_then(Value::as_str),
            Some("unselected")
        );
        assert!(result.output.get("自检问题").is_none());
    }
}
