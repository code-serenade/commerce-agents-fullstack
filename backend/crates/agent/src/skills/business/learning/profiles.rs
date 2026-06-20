use serde_json::{Value, json};

pub(super) struct TopicProfile {
    pub(super) conclusion: String,
    pub(super) terms: Vec<Value>,
    pub(super) why: Vec<String>,
    pub(super) how: Vec<String>,
    pub(super) pitfalls: Vec<String>,
}

pub(super) fn topic_profile(topic: &str) -> TopicProfile {
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
        conclusion: format!("{topic}要先讲清定义、适用场景、判断方法，再落到一个可验证的小动作。"),
        terms: vec![
            json!({"术语": topic, "白话解释": "先把抽象概念翻译成用户能观察、能计算、能行动的业务指标。"}),
            json!({"术语": "判断方法", "白话解释": "同一个词在不同岗位可能算法不同，必须先说清分母、分子和时间范围。"}),
        ],
        why: vec![
            "概念不清会导致岗位之间各说各话，后续动作难以验收。".to_owned(),
            "先明确怎么算、看哪段时间，可以减少错误归因和无效优化。".to_owned(),
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
