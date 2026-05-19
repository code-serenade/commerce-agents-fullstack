use serde_json::json;

use crate::skills::{SkillCategory, SkillInputField, SkillPriority, SkillSpec};

pub(super) fn category() -> SkillCategory {
    SkillCategory::Custom("design".to_owned())
}

pub fn specs() -> Vec<SkillSpec> {
    vec![
        design_main_image_spec(),
        design_detail_page_spec(),
        design_color_scheme_spec(),
        design_material_spec_spec(),
        design_campaign_poster_spec(),
    ]
}

pub fn design_main_image_spec() -> SkillSpec {
    native_spec(
        "design_main_image",
        "主图设计方案",
        "AI生成完整主图设计执行方案（色值/构图/文案/素材清单），\
         传入product_ref可关联新产品商品上下文",
        vec![
            SkillInputField::optional(
                "product_name",
                "商品名称（可从product_id自动加载）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "平台：淘宝/京东/拼多多/抖音",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "style",
                "视觉风格：简约现代/高端奢华/年轻活力/国潮复古/科技感",
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
                "extra_requirements",
                "额外要求（如指定背景色/必须有价格/A/B测试）",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn design_detail_page_spec() -> SkillSpec {
    native_spec(
        "design_detail_page",
        "详情页设计方案",
        "AI生成完整详情页设计执行方案（模块文案/构图/色彩/素材清单），\
         传入product_ref可关联新产品商品上下文",
        vec![
            SkillInputField::optional(
                "product_name",
                "商品名称（可从product_id自动加载）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "平台：淘宝/京东/拼多多/抖音",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "style",
                "风格：简约现代/高端奢华/年轻活力/国潮",
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
            SkillInputField::optional("extra_requirements", "额外要求", json!({"type": "string"})),
        ],
    )
}

pub fn design_color_scheme_spec() -> SkillSpec {
    native_spec(
        "design_color_scheme",
        "配色方案",
        "AI生成完整品牌配色方案，含具体HEX色值/使用比例/心理分析/Figma变量命名规范",
        vec![
            SkillInputField::required(
                "brand_tone",
                "品牌调性：高端奢华/年轻活力/清新自然/科技感/国潮复古/可爱萌系",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "category",
                "商品类目（如美妆/食品/数码）",
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
            SkillInputField::optional("platform", "目标平台", json!({"type": "string"})),
            SkillInputField::optional(
                "extra_requirements",
                "特殊要求（如指定主色/品牌色）",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub fn design_material_spec_spec() -> SkillSpec {
    SkillSpec::new(
        "design_material_spec",
        "素材规范",
        "生成各平台素材技术规格清单骨架，含尺寸/格式/文件大小/命名规范/验收标准",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::required(
        "usage",
        "用途：主图套组/详情页/直通车/信息流广告/社交媒体/直播封面",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "platforms",
        "目标平台列表（如 [淘宝, 京东, 抖音]）",
        json!({"type": "array", "items": {"type": "string"}}),
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
    .with_tag("p1")
}

pub fn design_campaign_poster_spec() -> SkillSpec {
    native_spec(
        "design_campaign_poster",
        "活动海报设计",
        "AI生成完整活动海报设计方案，含色值/文案/构图/各尺寸版本规格，\
         传入product_ref可关联新产品商品上下文",
        vec![
            SkillInputField::required(
                "campaign_name",
                "活动名称（如 618大促/新品首发/年货节）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "discount_info",
                "优惠信息（如 全场5折/满300减50）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional("campaign_date", "活动日期", json!({"type": "string"})),
            SkillInputField::optional(
                "style",
                "风格：热闹促销/简约高端/国潮节庆/科技感",
                json!({"type": "string"}),
            ),
            SkillInputField::optional("platform", "主要展示平台", json!({"type": "string"})),
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

pub(super) fn native_spec(
    id: &str,
    name: &str,
    description: &str,
    inputs: Vec<SkillInputField>,
) -> SkillSpec {
    inputs.into_iter().fold(
        SkillSpec::new(id, name, description)
            .with_category(category())
            .with_priority(SkillPriority::High)
            .with_tag("deterministic")
            .with_tag("p1"),
        SkillSpec::with_input,
    )
}
