use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillResult, SkillSpec, object_params, optional_f64_param, optional_i64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("creative".to_owned())
}

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

pub fn creative_video_script_spec() -> SkillSpec {
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

pub fn creative_seeding_copy_spec() -> SkillSpec {
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

pub fn creative_ip_branding_spec() -> SkillSpec {
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

pub fn creative_title_ctr_scorer_spec() -> SkillSpec {
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

pub struct CreativeTitleCtrScorer {
    spec: SkillSpec,
}

impl CreativeTitleCtrScorer {
    pub fn new() -> Self {
        Self {
            spec: creative_title_ctr_scorer_spec(),
        }
    }
}

impl Default for CreativeTitleCtrScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CreativeTitleCtrScorer {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let title = optional_string_param(&params, "title")?
            .ok_or_else(|| SkillError::missing_parameter("title"))?;
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let audience =
            optional_string_param(&params, "target_audience")?.unwrap_or_else(|| "通用".to_owned());
        let aida = score_aida(&title);
        let trigger = score_triggers(&title);
        let title_len = title.chars().count() as i64;
        let (lo, hi) = match platform.as_str() {
            "淘宝" => (25, 32),
            "京东" => (25, 35),
            "拼多多" => (18, 25),
            "抖音" => (12, 20),
            _ => (20, 30),
        };
        let mid = (lo + hi) / 2;
        let struct_score = if (lo ..= hi).contains(&title_len) {
            25
        } else {
            (25 - (title_len - mid).abs()).max(5)
        };
        let aida_score = (aida.weighted * 40.0).round() as i64;
        let trigger_score = (trigger.normalized * 35.0).round() as i64;
        let total = aida_score + trigger_score + struct_score;
        let base_ctr = match platform.as_str() {
            "淘宝" => 3.2,
            "京东" => 2.8,
            "拼多多" => 4.5,
            "抖音" => 5.0,
            _ => 3.5,
        };
        let predicted_ctr = ((base_ctr + (total as f64 - 50.0) * 0.03) * 100.0).round() / 100.0;
        let mut suggestions = Vec::new();
        if aida.weighted < 0.5 {
            suggestions.push("加强AIDA缺失维度：注意力、兴趣、欲望或行动号召".to_owned());
        }
        if trigger.normalized < 0.3 {
            suggestions.push("增加情感触发词（紧迫感/稀缺性/社交证明）".to_owned());
        }
        if title_len < lo {
            suggestions.push(format!("标题偏短，可补充卖点至{lo}字以上"));
        }
        if suggestions.is_empty() {
            suggestions.push("标题CTR潜力良好，可微调紧迫感词语".to_owned());
        }

        let mut output = json!({
            "平台": platform,
            "标题": title,
            "目标受众": audience,
            "综合CTR评分": format!("{total}/100"),
            "总分": total,
            "评级": ctr_grade(total),
            "预测CTR": format!("{:.2}%（行业基准{}%）", predicted_ctr.clamp(0.5, 15.0), base_ctr),
            "分项评分": {
                "AIDA结构": {"得分": format!("{aida_score}/40"), "命中要素": aida.hits},
                "情感触发词密度": {"得分": format!("{trigger_score}/35"), "命中词": trigger.words},
                "标题结构": {"得分": format!("{struct_score}/25"), "字数": title_len, "推荐范围": format!("{lo}-{hi}字")}
            },
            "优化建议": suggestions
        });
        normalize_creative_output("creative_title_ctr_scorer", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("标题CTR预测评分已完成"))
    }
}

pub fn creative_content_calendar_spec() -> SkillSpec {
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

pub fn creative_trend_catch_spec() -> SkillSpec {
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

pub fn creative_live_script_spec() -> SkillSpec {
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

pub fn creative_product_article_spec() -> SkillSpec {
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

macro_rules! native_creative_skill {
    ($type_name:ident, $spec_fn:ident, required [$($required:literal),*], defaults {$($key:literal => $value:expr),* $(,)?}) => {
        pub struct $type_name {
            spec: SkillSpec,
        }

        impl $type_name {
            pub fn new() -> Self {
                Self { spec: $spec_fn() }
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }

        #[async_trait]
        impl Skill for $type_name {
            fn spec(&self) -> &SkillSpec {
                &self.spec
            }

            async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
                object_params(&params)?;
                $(let _ = string_param(&params, $required)?;)*
                let mut normalized = serde_json::Map::new();
                $(normalized.insert($key.to_owned(), json!($value(&params)?));)*
                let mut outcome = native_creative_outcome(&self.spec.id, normalized);
                normalize_creative_output(&self.spec.id, &params, &mut outcome.output);
                Ok(outcome)
            }
        }
    };
}

native_creative_skill!(
    CreativeVideoScript,
    creative_video_script_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "video_type" => |p: &Value| Ok(optional_string_param(p, "video_type")?.unwrap_or_else(|| "种草".to_owned())),
        "duration" => |p: &Value| Ok(optional_i64_param(p, "duration")?.unwrap_or(30)),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "抖音".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
    }
);
native_creative_skill!(
    CreativeSeedingCopy,
    creative_seeding_copy_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "selling_points" => |p: &Value| Ok(optional_string_vec_param(p, "selling_points")?.unwrap_or_default()),
        "target_audience" => |p: &Value| Ok(optional_string_param(p, "target_audience")?.unwrap_or_else(|| "年轻女性".to_owned())),
        "tone" => |p: &Value| Ok(optional_string_param(p, "tone")?.unwrap_or_else(|| "闺蜜分享".to_owned())),
    }
);
native_creative_skill!(
    CreativeIpBranding,
    creative_ip_branding_spec,
    required ["brand_name", "category"],
    defaults {
        "brand_name" => |p: &Value| string_param(p, "brand_name").map(Some),
        "category" => |p: &Value| string_param(p, "category").map(Some),
        "target_audience" => |p: &Value| Ok(optional_string_param(p, "target_audience")?.unwrap_or_else(|| "18-35岁年轻消费者".to_owned())),
        "differentiator" => |p: &Value| Ok(optional_string_param(p, "differentiator")?.unwrap_or_else(|| "专业+有温度".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "小红书/抖音".to_owned())),
    }
);
native_creative_skill!(
    CreativeTrendCatch,
    creative_trend_catch_spec,
    required ["category"],
    defaults {
        "category" => |p: &Value| string_param(p, "category").map(Some),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "抖音".to_owned())),
    }
);
native_creative_skill!(
    CreativeLiveScript,
    creative_live_script_spec,
    required [],
    defaults {
        "duration_hours" => |p: &Value| Ok(optional_f64_param(p, "duration_hours")?.unwrap_or(3.0)),
        "product_count" => |p: &Value| Ok(optional_i64_param(p, "product_count")?.unwrap_or(6)),
        "product_refs" => |p: &Value| Ok(optional_string_vec_param(p, "product_refs")?.unwrap_or_default()),
        "live_type" => |p: &Value| Ok(optional_string_param(p, "live_type")?.unwrap_or_else(|| "日常带货".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "抖音".to_owned())),
    }
);
native_creative_skill!(
    CreativeProductArticle,
    creative_product_article_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "article_type" => |p: &Value| Ok(optional_string_param(p, "article_type")?.unwrap_or_else(|| "测评软文".to_owned())),
        "word_count" => |p: &Value| Ok(optional_i64_param(p, "word_count")?.unwrap_or(800)),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "微信公众号".to_owned())),
    }
);

fn normalize_creative_output(skill_id: &str, input: &Value, output: &mut Value) {
    let Some(map) = output.as_object_mut() else {
        return;
    };
    if map.contains_key("status") {
        return;
    }
    map.insert("status".to_owned(), json!("success"));
    map.insert("has_data".to_owned(), json!(true));
    map.insert("adapter_status".to_owned(), json!("manual_input"));
    map.insert(
        "data_readiness".to_owned(),
        json!(creative_data_readiness(skill_id, input)),
    );
    map.insert("summary".to_owned(), json!(creative_summary(skill_id)));
}

fn creative_data_readiness(skill_id: &str, input: &Value) -> &'static str {
    match skill_id {
        "creative_video_script"
            if !has_any_input(input, &["product_name", "product_ref", "product_id"]) =>
        {
            "calculated_from_defaults"
        }
        "creative_seeding_copy"
            if !has_any_input(
                input,
                &[
                    "product_name",
                    "product_ref",
                    "selling_points",
                    "product_id",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "creative_content_calendar"
            if !has_any_input(input, &["platforms", "content_types", "weekly_posts"]) =>
        {
            "calculated_from_defaults"
        }
        "creative_live_script"
            if !has_any_input(
                input,
                &[
                    "duration_hours",
                    "product_count",
                    "product_refs",
                    "product_ids",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "creative_product_article"
            if !has_any_input(input, &["product_name", "product_ref", "product_id"]) =>
        {
            "calculated_from_defaults"
        }
        _ => "user_provided",
    }
}

fn has_any_input(input: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        input.get(*key).is_some_and(|value| match value {
            Value::Null => false,
            Value::String(value) => !value.trim().is_empty(),
            Value::Array(values) => !values.is_empty(),
            Value::Object(values) => !values.is_empty(),
            _ => true,
        })
    })
}

fn creative_summary(skill_id: &str) -> &'static str {
    match skill_id {
        "creative_video_script" => "短视频脚本已生成",
        "creative_seeding_copy" => "种草文案已生成",
        "creative_ip_branding" => "IP定位方案已生成",
        "creative_content_calendar" => "内容排期已生成",
        "creative_trend_catch" => "内容趋势方向已生成",
        "creative_live_script" => "直播话术脚本已生成",
        "creative_product_article" => "产品软文草稿已生成",
        "creative_title_ctr_scorer" => "标题CTR预测评分已完成",
        _ => "创意技能已返回输入摘要",
    }
}

fn native_creative_outcome(skill_id: &str, input: Map<String, Value>) -> SkillOutcome {
    match skill_id {
        "creative_video_script" => {
            let product = normalized_string(&input, "product_name", "待推广商品");
            let video_type = normalized_string(&input, "video_type", "种草");
            let duration = normalized_i64(&input, "duration", 30).clamp(10, 180);
            let platform = normalized_string(&input, "platform", "抖音");
            SkillOutcome::new(json!({
                "能力": "短视频脚本",
                "商品": product,
                "平台": platform,
                "视频类型": video_type,
                "时长秒": duration,
                "脚本结构": [
                    {"时间": "0-3秒", "模块": "开场钩子", "画面": "直接展示使用前后的强对比或高频痛点", "台词": format!("还在为{}的选择纠结？先看这个真实场景。", product), "字幕": "先看效果，再决定要不要入手"},
                    {"时间": "3-10秒", "模块": "痛点场景", "画面": "用户在通勤、办公或直播间场景中遇到具体问题", "台词": "很多人不是不想买，而是不知道它到底解决什么问题。", "字幕": "把痛点讲具体"},
                    {"时间": format!("10-{}秒", (duration - 5).max(12)), "模块": "卖点展示", "画面": "用近景、手部动作和对比镜头展示核心卖点", "台词": format!("这款{}重点看三件事：场景适配、使用便利、转化利益点。", product), "字幕": "卖点1 / 卖点2 / 卖点3"},
                    {"时间": "最后5秒", "模块": "行动号召", "画面": "回到商品和优惠利益点", "台词": "想要同款体验，先收藏，再看当前活动价。", "字幕": "收藏 / 加购 / 领券"}
                ],
                "拍摄提示": ["首屏必须给结果或冲突", "单镜头不超过4秒", "每个卖点都配一个可见动作", "避免夸大功效和绝对化承诺"],
                "复盘指标": ["3秒留存", "完播率", "评论问题", "商品点击率", "加购率"],
                "数据边界": "本地 native 产物未读取外部实时趋势和商品详情，可在接入商品/搜索 adapter 后增强。"
            }))
            .with_summary("短视频脚本已生成")
        }
        "creative_seeding_copy" => {
            let product = normalized_string(&input, "product_name", "待推广商品");
            let audience = normalized_string(&input, "target_audience", "年轻女性");
            let tone = normalized_string(&input, "tone", "闺蜜分享");
            let selling_points = normalized_string_vec(&input, "selling_points");
            let points = if selling_points.is_empty() {
                vec![
                    "真实使用场景".to_owned(),
                    "决策门槛低".to_owned(),
                    "适合日常复购".to_owned(),
                ]
            } else {
                selling_points
            };
            SkillOutcome::new(json!({
                "能力": "种草文案",
                "商品": product,
                "目标人群": audience,
                "语气": tone,
                "标题备选": [
                    format!("用了{}之后，我才知道以前踩了多少坑", product),
                    format!("{}到底值不值得买？我按真实场景说", product),
                    format!("给{}人群的{}选择清单", audience, product)
                ],
                "正文草稿": [
                    format!("最近在看{}，我最关心的不是参数堆得多不多，而是它能不能解决真实场景里的问题。", product),
                    format!("实际体验下来，最值得说的是：{}。", points.join("、")),
                    "如果你和我一样更在意省心、稳定和日常使用感，可以先从核心卖点和售后承诺两件事判断。".to_owned(),
                    "建议先收藏，再对比同价位竞品的评价和真实买家反馈，别只看单条爆款内容下单。".to_owned()
                ],
                "话题标签": ["#电商好物", "#真实测评", "#种草笔记", "#购物决策", "#好物分享"],
                "发布建议": {"平台": "小红书/抖音图文", "时间": "工作日20:00-22:30或周末10:00-12:00", "配图": "首图给结果，二图列卖点，三图放使用场景"},
                "转化动作": ["评论区收集顾虑", "置顶优惠和规格说明", "把高频问题沉淀为下一条内容"]
            }))
            .with_summary("种草文案已生成")
        }
        "creative_ip_branding" => {
            let brand = normalized_string(&input, "brand_name", "品牌");
            let category = normalized_string(&input, "category", "通用类目");
            let audience = normalized_string(&input, "target_audience", "18-35岁年轻消费者");
            let differentiator = normalized_string(&input, "differentiator", "专业+有温度");
            let platform = normalized_string(&input, "platform", "小红书/抖音");
            SkillOutcome::new(json!({
                "能力": "IP定位",
                "品牌": brand,
                "类目": category,
                "目标受众": audience,
                "差异化": differentiator,
                "平台": platform,
                "人设定位": format!("{}领域里的实用型选购顾问，表达风格是{}。", category, differentiator),
                "账号简介": format!("帮{}用户少踩坑、会选择、买得值。", audience),
                "内容支柱": [
                    {"栏目": "选购避坑", "作用": "建立专业可信度"},
                    {"栏目": "真实场景测评", "作用": "让用户看到使用后变化"},
                    {"栏目": "价格与规格解释", "作用": "降低决策成本"},
                    {"栏目": "用户问题答疑", "作用": "沉淀客服与内容共用资产"}
                ],
                "语气规范": ["少喊口号，多讲证据", "不做绝对化承诺", "把参数翻译成用户能理解的场景利益"],
                "30天起号动作": ["每天1条场景内容", "每周2条测评/对比", "每周复盘评论问题并改选题"]
            }))
            .with_summary("IP定位方案已生成")
        }
        "creative_trend_catch" => {
            let category = normalized_string(&input, "category", "通用类目");
            let platform = normalized_string(&input, "platform", "抖音");
            SkillOutcome::new(json!({
                "能力": "趋势捕捉",
                "类目": category,
                "平台": platform,
                "趋势方向": [
                    {"方向": "真实测评透明化", "内容切入": "把优缺点、适用人群和不适用人群讲清楚", "适合指标": "收藏率/评论问题量"},
                    {"方向": "场景解决方案", "内容切入": "从通勤、办公、家庭、礼赠等场景切入", "适合指标": "完播率/商品点击率"},
                    {"方向": "价格价值解释", "内容切入": "拆解为什么值这个价，和竞品差异在哪里", "适合指标": "加购率/转化率"},
                    {"方向": "评论区共创", "内容切入": "把用户质疑直接做成下一条内容", "适合指标": "互动率/复访率"}
                ],
                "首批选题": [
                    format!("{}用户最容易忽略的3个选择标准", category),
                    format!("{}到底怎么选才不浪费钱", category),
                    format!("同价位{}，我会先看这4件事", category)
                ],
                "风险": ["未接实时外部搜索，不能代表当天热点榜单", "涉及功效、销量、排名时必须用真实证据"],
                "下一步": ["接入平台热搜或站内搜索词", "把趋势方向映射到内容排期", "用标题CTR和完播率做淘汰"]
            }))
            .with_summary("内容趋势方向已生成")
        }
        "creative_live_script" => {
            let duration = normalized_f64(&input, "duration_hours", 3.0).clamp(0.5, 12.0);
            let product_count = normalized_i64(&input, "product_count", 6).clamp(1, 30);
            let live_type = normalized_string(&input, "live_type", "日常带货");
            let platform = normalized_string(&input, "platform", "抖音");
            SkillOutcome::new(json!({
                "能力": "直播脚本",
                "直播类型": live_type,
                "平台": platform,
                "时长小时": duration,
                "商品数": product_count,
                "阶段脚本": [
                    {"阶段": "开场暖场", "时间": "0-15分钟", "话术": "欢迎刚进来的朋友，今天先不急着下单，我会把适合谁、不适合谁、怎么选讲清楚。先点关注，后面有限时福利。", "目标": "建立信任和停留"},
                    {"阶段": "福利引流款", "时间": "15-30分钟", "话术": "这款先给新朋友试水，重点不是便宜，而是用最低门槛确认你适不适合。库存和券都有限，先领券再决定。", "目标": "拉动互动和首单"},
                    {"阶段": "主推款讲解", "时间": "循环讲解", "话术": "看这个商品先抓三点：使用场景、核心卖点、售后保障。适合的朋友可以加购，不适合我也会直接说。", "目标": "提高商品点击和加购"},
                    {"阶段": "返场收尾", "时间": "最后30分钟", "话术": "最后把今天问得最多的问题再统一回答一次，错过本轮福利的先关注，下次开播会继续复盘真实反馈。", "目标": "收口转化和沉淀粉丝"}
                ],
                "高频话术": ["先领券，不急着拍", "适合再买，不适合别冲动", "这个问题问得好，我现场拆一下", "库存不做夸张承诺，以后台为准", "拍下前先看规格和售后"],
                "运营提醒": ["每15分钟复述核心利益点", "把高频问题交给客服沉淀FAQ", "避免虚假最低价和绝对化效果承诺"]
            }))
            .with_summary("直播话术脚本已生成")
        }
        "creative_product_article" => {
            let product = normalized_string(&input, "product_name", "待推广商品");
            let article_type = normalized_string(&input, "article_type", "测评软文");
            let word_count = normalized_i64(&input, "word_count", 800).clamp(300, 3000);
            let platform = normalized_string(&input, "platform", "微信公众号");
            SkillOutcome::new(json!({
                "能力": "产品软文",
                "商品": product,
                "文章类型": article_type,
                "目标字数": word_count,
                "平台": platform,
                "标题": format!("{}怎么选？一篇讲清真实场景、核心卖点和避坑点", product),
                "文章结构": [
                    {"模块": "开头", "写法": "用具体痛点切入，说明为什么需要重新评估这个商品"},
                    {"模块": "场景", "写法": "把目标用户、使用场景和购买动机讲清楚"},
                    {"模块": "卖点", "写法": "每个卖点都对应一个使用结果，不只列参数"},
                    {"模块": "对比", "写法": "说明同价位选择标准，不编造竞品数据"},
                    {"模块": "结尾", "写法": "给出适合/不适合人群和购买前检查项"}
                ],
                "正文草稿": [
                    format!("很多人在选{}时，容易先看价格或单条推荐，但真正影响复购和满意度的是使用场景是否匹配。", product),
                    "建议先判断三个问题：你在哪个场景使用、最不能接受的缺点是什么、售后和规格是否清楚。".to_owned(),
                    format!("如果{}能在核心场景里稳定解决问题，再结合价格、评价和服务承诺判断是否值得入手。", product),
                    "购买前建议保留对比清单，不要只被限时话术推动。".to_owned()
                ],
                "CTA": "先收藏这份选购清单，再结合你的预算和使用场景做最终选择。",
                "合规提醒": ["不写绝对功效", "不编造销量排名", "涉及数据时标注来源"]
            }))
            .with_summary("产品软文草稿已生成")
        }
        _ => SkillOutcome::new(json!({
            "能力": skill_id,
            "输入": input,
            "说明": "未知创意 skill，已返回输入摘要。"
        }))
        .with_summary("创意技能已返回输入摘要"),
    }
}

pub struct CreativeContentCalendar {
    spec: SkillSpec,
}

impl CreativeContentCalendar {
    pub fn new() -> Self {
        Self {
            spec: creative_content_calendar_spec(),
        }
    }
}

impl Default for CreativeContentCalendar {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CreativeContentCalendar {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let period = optional_string_param(&params, "period")?.unwrap_or_else(|| "月".to_owned());
        let platforms = optional_string_vec_param(&params, "platforms")?
            .unwrap_or_else(|| vec!["抖音".to_owned(), "小红书".to_owned()]);
        let content_types =
            optional_string_vec_param(&params, "content_types")?.unwrap_or_else(|| {
                vec![
                    "种草测评".to_owned(),
                    "干货教程".to_owned(),
                    "生活日常".to_owned(),
                    "互动话题".to_owned(),
                ]
            });
        let weekly_posts = optional_i64_param(&params, "weekly_posts")?
            .unwrap_or(5)
            .max(1);
        let schedule = build_content_schedule(&platforms, &content_types, weekly_posts);
        let mut output = json!({
            "能力": "内容排期",
            "周期": period,
            "发布平台": platforms,
            "内容类型": content_types,
            "每周发布数量": weekly_posts,
            "排期": schedule,
            "选题原则": ["首条给痛点或结果", "中段做测评和对比", "末段承接直播/私域/客服问题", "每周复盘标题CTR和完播率"],
            "复盘字段": ["发布时间", "平台", "选题", "内容形式", "曝光", "点击", "完播", "加购", "评论问题"]
        });
        normalize_creative_output("creative_content_calendar", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("内容排期已生成"))
    }
}

fn build_content_schedule(
    platforms: &[String],
    content_types: &[String],
    weekly_posts: i64,
) -> Vec<Value> {
    let days = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
    let count = weekly_posts.clamp(1, 14) as usize;
    let default_platform = "抖音".to_owned();
    let default_type = "种草测评".to_owned();
    (0..count)
        .map(|index| {
            let platform = if platforms.is_empty() {
                &default_platform
            } else {
                &platforms[index % platforms.len()]
            };
            let content_type = if content_types.is_empty() {
                &default_type
            } else {
                &content_types[index % content_types.len()]
            };
            json!({
                "时间": days[index % days.len()],
                "平台": platform,
                "内容类型": content_type,
                "选题": format!("{}方向：围绕用户痛点、卖点证据和购买顾虑展开", content_type),
                "目标": if index % 3 == 0 { "拉新曝光" } else if index % 3 == 1 { "建立信任" } else { "促进转化" },
                "指标": ["曝光", "点击", "互动", "加购"]
            })
        })
        .collect()
}

fn normalized_string(input: &Map<String, Value>, key: &str, fallback: &str) -> String {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| fallback.to_owned())
}

fn normalized_i64(input: &Map<String, Value>, key: &str, fallback: i64) -> i64 {
    input.get(key).and_then(Value::as_i64).unwrap_or(fallback)
}

fn normalized_f64(input: &Map<String, Value>, key: &str, fallback: f64) -> f64 {
    input.get(key).and_then(Value::as_f64).unwrap_or(fallback)
}

fn normalized_string_vec(input: &Map<String, Value>, key: &str) -> Vec<String> {
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

struct AidaScore {
    weighted: f64,
    hits: Value,
}

const AIDA_GROUPS: &[(&str, f64, &[&str])] = &[
    (
        "Attention",
        0.30,
        &[
            "限时", "爆款", "独家", "首发", "全网", "新品", "震撼", "惊喜", "超级",
        ],
    ),
    (
        "Interest",
        0.25,
        &[
            "功能",
            "特点",
            "原因",
            "为什么",
            "如何",
            "秘密",
            "揭秘",
            "技巧",
            "方法",
            "攻略",
        ],
    ),
    (
        "Desire",
        0.25,
        &[
            "完美",
            "必买",
            "值得",
            "推荐",
            "好用",
            "高品质",
            "升级",
            "加强",
            "专业",
            "极致",
        ],
    ),
    (
        "Action",
        0.20,
        &[
            "抢",
            "买",
            "下单",
            "立即",
            "马上",
            "赶快",
            "别错过",
            "收藏",
            "加购",
            "点击",
        ],
    ),
];

fn score_aida(title: &str) -> AidaScore {
    let mut weighted = 0.0;
    let mut hits = serde_json::Map::new();
    for (name, weight, words) in AIDA_GROUPS {
        let matched = words
            .iter()
            .filter(|word| title.contains(**word))
            .cloned()
            .collect::<Vec<_>>();
        let raw = (0.5 + matched.len() as f64 * 0.25).min(1.0);
        weighted += raw * weight;
        if !matched.is_empty() {
            hits.insert((*name).to_owned(), json!(matched));
        }
    }
    AidaScore {
        weighted: (weighted * 1000.0).round() / 1000.0,
        hits: Value::Object(hits),
    }
}

struct TriggerScore {
    normalized: f64,
    words: Vec<&'static str>,
}

const TRIGGER_GROUPS: &[(f64, &[&str])] = &[
    (
        1.8,
        &[
            "限时",
            "今天",
            "最后",
            "仅剩",
            "抢购",
            "秒杀",
            "即将",
            "截止",
            "倒计时",
            "马上",
        ],
    ),
    (
        1.6,
        &[
            "限量",
            "仅余",
            "稀缺",
            "孤品",
            "断码",
            "最后一批",
            "售完为止",
            "库存告急",
        ],
    ),
    (
        1.4,
        &[
            "爆款",
            "热卖",
            "畅销",
            "好评",
            "口碑",
            "网红",
            "明星",
            "万人",
            "口碑推荐",
            "人气",
        ],
    ),
    (
        1.3,
        &[
            "官方", "正品", "认证", "权威", "专业", "国家", "医院", "大牌", "品牌",
        ],
    ),
    (
        1.5,
        &[
            "超值",
            "划算",
            "省钱",
            "折扣",
            "优惠",
            "打折",
            "特价",
            "白菜价",
            "半价",
        ],
    ),
    (
        1.2,
        &[
            "惊喜", "感动", "完美", "心动", "必买", "真香", "安心", "放心", "幸福", "快乐",
        ],
    ),
];

fn score_triggers(title: &str) -> TriggerScore {
    let mut raw: f64 = 0.0;
    let mut words = Vec::new();
    for (weight, group) in TRIGGER_GROUPS {
        for word in *group {
            if title.contains(word) {
                raw += weight;
                words.push(*word);
            }
        }
    }
    TriggerScore {
        normalized: (raw / 3.0).min(1.0),
        words,
    }
}

fn ctr_grade(total: i64) -> &'static str {
    if total >= 85 {
        "S 高转化"
    } else if total >= 70 {
        "A 良好"
    } else if total >= 55 {
        "B 中等"
    } else if total >= 40 {
        "C 待改进"
    } else {
        "D 需重写"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_success_contract(output: &Value, summary: &str, readiness: &str) {
        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(output.get("has_data").and_then(Value::as_bool), Some(true));
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some(readiness)
        );
        assert_eq!(output.get("summary").and_then(Value::as_str), Some(summary));
    }

    #[tokio::test]
    async fn creative_video_script_discloses_default_inputs() {
        let outcome = CreativeVideoScript::new()
            .run(json!({}), SkillContext::default())
            .await
            .expect("video script should run with defaults");

        assert_success_contract(
            &outcome.output,
            "短视频脚本已生成",
            "calculated_from_defaults",
        );
        assert_eq!(
            outcome.output.get("商品").and_then(Value::as_str),
            Some("待推广商品")
        );
    }

    #[tokio::test]
    async fn creative_video_script_accepts_product_ref_as_user_context() {
        let outcome = CreativeVideoScript::new()
            .run(
                json!({
                    "product_ref": "product:11111111-1111-1111-1111-111111111111",
                    "platform": "抖音"
                }),
                SkillContext::default(),
            )
            .await
            .expect("video script should accept product_ref");

        assert_success_contract(&outcome.output, "短视频脚本已生成", "user_provided");
    }

    #[tokio::test]
    async fn creative_trend_catch_marks_success_for_category() {
        let outcome = CreativeTrendCatch::new()
            .run(json!({"category": "冷萃咖啡"}), SkillContext::default())
            .await
            .expect("trend catch should run with category");

        assert_success_contract(&outcome.output, "内容趋势方向已生成", "user_provided");
        assert_eq!(
            outcome.output.get("类目").and_then(Value::as_str),
            Some("冷萃咖啡")
        );
    }

    #[tokio::test]
    async fn creative_title_ctr_scorer_marks_success_for_user_title() {
        let outcome = CreativeTitleCtrScorer::new()
            .run(
                json!({"title": "限时新品冷萃咖啡，通勤党立即加购"}),
                SkillContext::default(),
            )
            .await
            .expect("title CTR scorer should run with title");

        assert_success_contract(&outcome.output, "标题CTR预测评分已完成", "user_provided");
        assert!(
            outcome
                .output
                .get("综合CTR评分")
                .and_then(Value::as_str)
                .is_some()
        );
    }

    #[tokio::test]
    async fn creative_content_calendar_discloses_default_inputs() {
        let outcome = CreativeContentCalendar::new()
            .run(json!({}), SkillContext::default())
            .await
            .expect("content calendar should run with defaults");

        assert_success_contract(
            &outcome.output,
            "内容排期已生成",
            "calculated_from_defaults",
        );
        assert!(
            outcome
                .output
                .get("排期")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty())
        );
    }
}
