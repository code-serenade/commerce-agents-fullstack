pub fn specs() -> Vec<SkillSpec> {
    vec![
        web_seo_optimize_spec(),
        web_title_generator_spec(),
        web_page_conversion_spec(),
        web_store_design_spec(),
        web_keyword_research_spec(),
        web_product_description_spec(),
        web_title_seo_scorer_spec(),
    ]
}

pub fn web_seo_optimize_spec() -> SkillSpec {
    native_spec(
        "web_seo_optimize",
        "SEO优化方案",
        "AI生成完整SEO方案（关键词矩阵/标题优化/内容改写清单/技术SEO检查/提排路径），\
         自动关联真实商品和平台数据",
        vec![
            SkillInputField::optional(
                "target_keywords",
                "目标关键词列表（可从product_id自动生成）",
                json!({"type": "array", "items": {"type": "string"}}),
            ),
            SkillInputField::optional(
                "current_rank",
                "当前排名（可选）",
                json!({"type": "integer"}),
            ),
            SkillInputField::optional(
                "product_id",
                "旧整数商品ID，兼容旧参数",
                json!({"type": "integer"}),
            ),
            SkillInputField::optional(
                "product_ref",
                "新产品商品引用，如 product:{uuid}",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "平台：淘宝/京东/拼多多/抖音",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn web_title_generator_spec() -> SkillSpec {
    native_spec(
        "web_title_generator",
        "标题生成",
        "根据商品信息生成多种风格的SEO友好标题；传入product_ref可关联新产品商品上下文",
        vec![
            SkillInputField::optional(
                "product_name",
                "商品名称（可自动从product_id加载）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "keywords",
                "核心关键词（可从商品卖点自动生成）",
                json!({"type": "array", "items": {"type": "string"}}),
            ),
            SkillInputField::optional(
                "style",
                "风格：营销型/信息型/品牌型",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "product_id",
                "旧整数商品ID，兼容旧参数",
                json!({"type": "integer"}),
            ),
            SkillInputField::optional(
                "product_ref",
                "新产品商品引用，如 product:{uuid}",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "目标平台：淘宝/京东/拼多多/抖音",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn web_title_seo_scorer_spec() -> SkillSpec {
    SkillSpec::new(
        "web_title_seo_scorer",
        "标题SEO评分",
        "对电商商品标题进行SEO质量评分：关键词密度、主关键词位置、字符长度和可读性",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "title",
        "待评分的商品标题",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "target_keywords",
        "目标关键词列表（主关键词放第一位）",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "category",
        "商品类目",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "旧整数商品ID，兼容旧参数",
        json!({"type": "integer"}),
    ))
    .with_input(SkillInputField::optional(
        "product_ref",
        "新产品商品引用，如 product:{uuid}",
        json!({"type": "string"}),
    ))
    .with_tag("deterministic")
}

pub fn web_page_conversion_spec() -> SkillSpec {
    SkillSpec::new(
        "web_page_conversion",
        "页面转化优化",
        "分析页面转化瓶颈，输出优化方案；可接入真实转化率数据",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::required(
        "page_type",
        "页面类型：首页/商品详情/购物车/结算/列表页",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "current_conversion",
        "当前转化率(%)",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "bounce_rate",
        "跳出率(%)",
        json!({"type": "number"}),
    ))
    .with_input(SkillInputField::optional(
        "avg_stay_time",
        "平均停留时间(秒)",
        json!({"type": "number"}),
    ))
    .with_tag("deterministic")
    .with_tag("p1")
}

pub fn web_store_design_spec() -> SkillSpec {
    native_spec(
        "web_store_design",
        "店铺装修方案",
        "AI生成完整店铺装修设计方案（视觉定位/模块结构/尺寸规格/文案方向/素材清单/验收标准），\
         基于店铺真实数据个性化定制",
        vec![
            SkillInputField::required(
                "store_type",
                "店铺类型：旗舰店/专营店/个人店",
                json!({"type": "string"}),
            ),
            SkillInputField::optional("category", "主营类目", json!({"type": "string"})),
            SkillInputField::optional("platform", "平台", json!({"type": "string"})),
            SkillInputField::optional("brand_tone", "品牌调性", json!({"type": "string"})),
        ],
    )
}

pub fn web_keyword_research_spec() -> SkillSpec {
    native_spec(
        "web_keyword_research",
        "关键词研究",
        "AI拓展关键词矩阵（核心词/长尾词/蓝海词/场景词），分析搜索意图、竞争度和出价策略，\
         可自动关联真实商品",
        vec![
            SkillInputField::optional(
                "seed_keyword",
                "种子关键词（可从product_id自动提取）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional("category", "商品类目", json!({"type": "string"})),
            SkillInputField::optional(
                "platform",
                "平台：淘宝/京东/拼多多/抖音",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "product_id",
                "旧整数商品ID，兼容旧参数",
                json!({"type": "integer"}),
            ),
            SkillInputField::optional(
                "product_ref",
                "新产品商品引用，如 product:{uuid}",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn web_product_description_spec() -> SkillSpec {
    native_spec(
        "web_product_description",
        "详情页文案",
        "生成完整的详情页营销文案（首屏卖点/痛点/解决方案/亮点/行动号召）；\
         传入product_ref可关联新产品商品上下文",
        vec![
            SkillInputField::optional(
                "product_name",
                "商品名称（可从product_id自动加载）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "product_id",
                "旧整数商品ID，兼容旧参数",
                json!({"type": "integer"}),
            ),
            SkillInputField::optional(
                "product_ref",
                "新产品商品引用，如 product:{uuid}",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "目标平台：淘宝/京东/拼多多/抖音",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "style",
                "文案风格：营销型/信息型/故事型",
                json!({"type": "string"}),
            ),
        ],
    )
}

fn native_spec(id: &str, name: &str, description: &str, inputs: Vec<SkillInputField>) -> SkillSpec {
    inputs.into_iter().fold(
        SkillSpec::new(id, name, description)
            .with_category(category())
            .with_priority(SkillPriority::High)
            .with_tag("deterministic")
            .with_tag("p1"),
        SkillSpec::with_input,
    )
}
