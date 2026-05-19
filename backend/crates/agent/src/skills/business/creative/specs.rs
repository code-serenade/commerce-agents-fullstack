use serde_json::json;

use super::category;
use crate::skills::{SkillInputField, SkillPriority, SkillSpec};

pub fn specs() -> Vec<SkillSpec> {
    vec![
        creative_video_script_spec(),
        creative_seeding_copy_spec(),
        creative_ip_branding_spec(),
        creative_content_calendar_spec(),
        creative_trend_catch_spec(),
        creative_live_script_spec(),
        creative_product_article_spec(),
        creative_title_ctr_scorer_spec(),
    ]
}

pub(super) fn creative_video_script_spec() -> SkillSpec {
    deferred_spec(
        "creative_video_script",
        "短视频脚本",
        "根据商品和平台特点，生成短视频脚本框架；传入product_ref可关联新产品商品上下文",
        vec![
            SkillInputField::optional("product_name", "商品名称", json!({"type": "string"})),
            SkillInputField::optional(
                "product_ref",
                "新产品商品引用，如 product:{uuid}",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "video_type",
                "视频类型：种草/测评/教程/剧情/开箱",
                json!({"type": "string"}),
            ),
            SkillInputField::optional("duration", "目标时长（秒）", json!({"type": "integer"})),
            SkillInputField::optional(
                "platform",
                "平台：抖音/快手/小红书/B站",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "product_id",
                "旧整数商品ID，兼容旧参数",
                json!({"type": "integer"}),
            ),
        ],
    )
}

pub(super) fn creative_seeding_copy_spec() -> SkillSpec {
    deferred_spec(
        "creative_seeding_copy",
        "种草文案",
        "生成小红书/社交媒体种草文案框架；传入product_ref可关联新产品商品卖点",
        vec![
            SkillInputField::optional("product_name", "商品名称", json!({"type": "string"})),
            SkillInputField::optional(
                "product_ref",
                "新产品商品引用，如 product:{uuid}",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "selling_points",
                "核心卖点",
                json!({"type": "array", "items": {"type": "string"}}),
            ),
            SkillInputField::optional("target_audience", "目标人群", json!({"type": "string"})),
            SkillInputField::optional(
                "tone",
                "语气：闺蜜分享/专业测评/生活记录",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "product_id",
                "旧整数商品ID，兼容旧参数",
                json!({"type": "integer"}),
            ),
        ],
    )
}

pub(super) fn creative_ip_branding_spec() -> SkillSpec {
    deferred_spec(
        "creative_ip_branding",
        "IP定位",
        "为品牌或个人账号做IP人设定位",
        vec![
            SkillInputField::required("brand_name", "品牌/账号名称", json!({"type": "string"})),
            SkillInputField::required("category", "领域类目", json!({"type": "string"})),
            SkillInputField::optional("target_audience", "目标受众", json!({"type": "string"})),
            SkillInputField::optional("differentiator", "核心差异化点", json!({"type": "string"})),
            SkillInputField::optional(
                "platform",
                "主要运营平台，如 小红书/抖音/B站",
                json!({"type": "string"}),
            ),
        ],
    )
}

pub(super) fn creative_title_ctr_scorer_spec() -> SkillSpec {
    SkillSpec::new(
        "creative_title_ctr_scorer",
        "标题CTR预测评分",
        "AIDA框架×情感触发词密度评分，预测商品标题点击率",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "title",
        "待评分的商品/内容标题",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音/小红书",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "target_audience",
        "目标受众描述",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "product_id",
        "商品ID，自动加载标题",
        json!({"type": "integer"}),
    ))
    .with_tag("deterministic")
}

pub(super) fn creative_content_calendar_spec() -> SkillSpec {
    SkillSpec::new(
        "creative_content_calendar",
        "内容排期",
        "根据运营节奏生成周/月内容发布计划",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::optional(
        "period",
        "周期：周/月",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "platforms",
        "发布平台列表",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "content_types",
        "内容类型",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "weekly_posts",
        "每周发布数量",
        json!({"type": "integer"}),
    ))
    .with_tag("deterministic")
}

pub(super) fn creative_trend_catch_spec() -> SkillSpec {
    deferred_spec(
        "creative_trend_catch",
        "趋势捕捉",
        "分析当前内容趋势，输出可借鉴的创意方向",
        vec![
            SkillInputField::required("category", "行业类目", json!({"type": "string"})),
            SkillInputField::optional("platform", "平台", json!({"type": "string"})),
        ],
    )
}

pub(super) fn creative_live_script_spec() -> SkillSpec {
    deferred_spec(
        "creative_live_script",
        "直播脚本",
        "生成完整直播话术脚本，含每个阶段的真实台词；可传入product_refs关联新产品商品列表",
        vec![
            SkillInputField::required(
                "duration_hours",
                "直播时长（小时）",
                json!({"type": "number"}),
            ),
            SkillInputField::optional(
                "product_count",
                "上架商品数量（无product_ids时用）",
                json!({"type": "integer"}),
            ),
            SkillInputField::optional(
                "live_type",
                "直播类型：日常带货/大促专场/新品首发/清仓特卖",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "platform",
                "平台：抖音/快手/淘宝直播",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "product_refs",
                "新产品商品引用列表，如 product:{uuid}",
                json!({"type": "array", "items": {"type": "string"}}),
            ),
            SkillInputField::optional(
                "product_ids",
                "旧整数商品ID列表，兼容旧参数",
                json!({"type": "array", "items": {"type": "integer"}}),
            ),
        ],
    )
}

pub(super) fn creative_product_article_spec() -> SkillSpec {
    deferred_spec(
        "creative_product_article",
        "产品软文",
        "生成完整可发布的产品软文/推广文章，含标题和正文；传入product_ref可关联新产品商品上下文",
        vec![
            SkillInputField::optional("product_name", "商品名称", json!({"type": "string"})),
            SkillInputField::optional(
                "product_ref",
                "新产品商品引用，如 product:{uuid}",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "product_id",
                "旧整数商品ID，兼容旧参数",
                json!({"type": "integer"}),
            ),
            SkillInputField::optional(
                "article_type",
                "文章类型：测评软文/种草长文/对比评测/使用攻略/品牌故事",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "word_count",
                "目标字数，默认800字",
                json!({"type": "integer"}),
            ),
            SkillInputField::optional(
                "platform",
                "发布平台：微信公众号/小红书/知乎/抖音",
                json!({"type": "string"}),
            ),
        ],
    )
}

fn deferred_spec(
    id: &str,
    name: &str,
    description: &str,
    inputs: Vec<SkillInputField>,
) -> SkillSpec {
    inputs.into_iter().fold(
        SkillSpec::new(id, name, description)
            .with_category(category())
            .with_priority(SkillPriority::High)
            .with_tag("deterministic"),
        SkillSpec::with_input,
    )
}
