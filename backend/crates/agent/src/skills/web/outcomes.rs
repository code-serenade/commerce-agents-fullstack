fn seo_optimize(input: &Map<String, Value>) -> Value {
    let platform = str_field(input, "platform", "淘宝");
    let mut keywords = string_vec_field(input, "target_keywords");
    if keywords.is_empty() {
        keywords = vec!["电商".to_owned(), "高性价比".to_owned(), "新品".to_owned()];
    }
    let current_rank = i64_field(input, "current_rank").unwrap_or(0);
    json!({
        "has_data": true,
        "执行模式": "native_seo_plan",
        "数据来源": "用户输入 + 平台SEO规则",
        "平台": platform,
        "目标关键词": keywords,
        "当前排名": if current_rank > 0 { json!(current_rank) } else { json!("未提供") },
        "关键词分层": keyword_layers(&keywords),
        "标题优化": {
            "结构": "{主关键词}+{核心卖点}+{场景/人群}+{信任背书}",
            "规则": title_rules(&platform),
            "示例": format!("{} 官方优选 高性价比 新品现货", keywords.first().cloned().unwrap_or_else(|| "商品".to_owned()))
        },
        "内容改写清单": [
            {"位置": "标题前半段", "动作": "放入主关键词和最强卖点"},
            {"位置": "详情页首屏", "动作": "补充使用场景、痛点和核心承诺"},
            {"位置": "FAQ", "动作": "覆盖尺码、材质、发货、售后等搜索问题"},
            {"位置": "图片alt/素材命名", "动作": "加入核心关键词，方便内部检索和复用"}
        ],
        "技术检查": ["移动端首屏加载", "图片体积", "标题字符数", "重复词", "核心词是否靠前"],
        "提排路径": ["先改标题和首屏", "7天观察点击率/搜索曝光", "再扩展长尾词", "沉淀高转化词到商品知识"]
    })
}

fn title_generator(input: &Map<String, Value>) -> Value {
    let product = display_product(input);
    let platform = str_field(input, "platform", "淘宝");
    let style = str_field(input, "style", "营销型");
    let mut keywords = string_vec_field(input, "keywords");
    if keywords.is_empty() {
        keywords = vec![product.clone(), "新品".to_owned(), "高性价比".to_owned()];
    }
    let primary = keywords.first().cloned().unwrap_or_else(|| product.clone());
    json!({
        "has_data": true,
        "执行模式": "native_title_generation",
        "数据来源": "用户输入 + 平台标题规则",
        "商品": product,
        "平台": platform,
        "风格": style,
        "关键词": keywords,
        "标题候选": [
            {"类型": "搜索承接型", "标题": format!("{primary} {} 官方优选 高性价比 现货速发", product), "适用": "平台搜索和货架流量"},
            {"类型": "利益点型", "标题": format!("{} 新品升级 {} 日常家用送礼优选", product, primary), "适用": "促销和新品冷启动"},
            {"类型": "人群场景型", "标题": format!("{} 适合通勤/家用 {} 品质保障", product, primary), "适用": "内容种草后的搜索承接"},
            {"类型": "品牌信任型", "标题": format!("{} 官方旗舰品质 {} 售后无忧", product, primary), "适用": "高客单或礼赠场景"}
        ],
        "标题规则": title_rules(&platform),
        "避坑": ["不要堆砌无关热词", "同义词不要重复超过2次", "促销词需与实际权益一致"]
    })
}

fn keyword_research(input: &Map<String, Value>) -> Value {
    let seed = opt_str_field(input, "seed_keyword")
        .or_else(|| i64_field(input, "product_id").map(|id| format!("商品{id}")))
        .unwrap_or_else(|| str_field(input, "category", "电商消费品"));
    let category = str_field(input, "category", "通用");
    let platform = str_field(input, "platform", "淘宝");
    json!({
        "has_data": true,
        "执行模式": "native_keyword_matrix",
        "数据来源": "用户输入 + 确定性关键词拓展规则；未包含实时搜索量",
        "种子词": seed,
        "类目": category,
        "平台": platform,
        "关键词矩阵": [
            {"类型": "核心词", "词": [seed.clone(), category.clone()], "意图": "明确找商品", "竞争度": "高", "出价建议": "小预算精准匹配"},
            {"类型": "长尾词", "词": [format!("{seed} 推荐"), format!("{seed} 怎么选"), format!("{seed} 性价比")], "意图": "比较和决策", "竞争度": "中", "出价建议": "内容页和FAQ承接"},
            {"类型": "场景词", "词": [format!("{seed} 家用"), format!("{seed} 送礼"), format!("{seed} 通勤")], "意图": "场景触发", "竞争度": "中低", "出价建议": "详情页首屏加入场景图"},
            {"类型": "蓝海词", "词": [format!("{seed} 小众高质"), format!("{seed} 新品"), format!("{seed} 官方同款")], "意图": "低竞争探索", "竞争度": "低", "出价建议": "测试CTR后再放量"}
        ],
        "内容落点": [
            {"位置": "标题", "放置": "核心词+1个高转化长尾词"},
            {"位置": "详情首屏", "放置": "场景词和核心痛点"},
            {"位置": "FAQ", "放置": "怎么选/适合谁/售后等问题词"},
            {"位置": "素材命名", "放置": "核心词+场景词"}
        ],
        "缺口说明": "当前版本不读取实时搜索量和竞价数据，接入外部搜索/平台数据后可补充热度、趋势和CPC。"
    })
}

fn store_design(input: &Map<String, Value>) -> Value {
    let store_type = str_field(input, "store_type", "旗舰店");
    let category = str_field(input, "category", "通用");
    let platform = str_field(input, "platform", "淘宝");
    let tone = str_field(input, "brand_tone", "专业");
    json!({
        "has_data": true,
        "执行模式": "native_store_design_brief",
        "数据来源": "用户输入 + 店铺装修结构规则",
        "店铺类型": store_type,
        "主营类目": category,
        "平台": platform,
        "品牌调性": tone,
        "首页模块": [
            {"模块": "首屏品牌区", "目标": "让用户知道卖什么、为什么可信", "内容": ["品牌口号", "主推商品", "核心权益"]},
            {"模块": "爆款/新品区", "目标": "承接点击和转化", "内容": ["主推SKU", "价格权益", "评价背书"]},
            {"模块": "场景解决方案", "目标": "从商品陈列升级到需求解决", "内容": ["人群/场景", "组合推荐", "使用效果"]},
            {"模块": "信任与服务", "目标": "降低售后疑虑", "内容": ["发货时效", "退换承诺", "客服保障"]},
            {"模块": "内容种草", "目标": "提高停留和复购", "内容": ["教程", "测评", "买家秀"]}
        ],
        "视觉策略": {
            "配色方向": palette_name(&tone),
            "图片风格": "商品实拍 + 场景图 + 统一角标",
            "信息密度": if platform == "抖音" { "短屏强转化，单屏只讲一个动作" } else { "货架页面强调分类、权益和信任" }
        },
        "素材清单": ["店招", "首屏Banner", "分类入口", "爆款卡片", "服务承诺图", "活动腰封"],
        "验收指标": ["首页点击率", "商品详情点击率", "加购率", "客服咨询率", "店铺停留时长"]
    })
}

fn product_description(input: &Map<String, Value>) -> Value {
    let product = display_product(input);
    let platform = str_field(input, "platform", "淘宝");
    let style = str_field(input, "style", "营销型");
    json!({
        "has_data": true,
        "执行模式": "native_product_description",
        "数据来源": "用户输入 + 详情页文案结构规则",
        "商品": product,
        "平台": platform,
        "文案风格": style,
        "首屏文案": {
            "标题": format!("{}，把核心需求一次解决", product),
            "副标题": "围绕真实使用场景设计，兼顾品质、效率和售后保障",
            "行动提示": "立即查看规格与优惠"
        },
        "痛点与解决": [
            {"痛点": "用户难以判断是否适合自己", "解决文案": "用场景图和人群标签说明适用边界"},
            {"痛点": "担心品质和售后", "解决文案": "明确材质/服务承诺/退换规则"},
            {"痛点": "同类商品难区分", "解决文案": "突出1-2个真正不同的卖点，不堆砌空话"}
        ],
        "卖点段落": [
            format!("{} 适合日常高频使用，主打稳定体验和省心选择。", product),
            "关键参数和使用步骤放在同一屏，让用户不用反复询问客服。",
            "售后、发货和使用限制前置说明，减少下单前顾虑。"
        ],
        "FAQ": [
            {"问": "适合什么人？", "答": "适合关注品质、效率和售后保障的用户，具体以规格参数为准。"},
            {"问": "和普通款有什么区别？", "答": "建议突出材质、功能、服务或场景中的一个核心差异。"},
            {"问": "售后如何处理？", "答": "在详情页明确退换条件、时效和客服响应方式。"}
        ],
        "CTA": ["领取优惠后下单", "查看规格参数", "咨询客服确认适配"]
    })
}
