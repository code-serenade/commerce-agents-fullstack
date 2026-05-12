use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillInputField, SkillOutcome, SkillPriority, SkillResult,
    SkillSpec, object_params, optional_i64_param, optional_string_param, optional_string_vec_param,
    string_param,
};

fn category() -> SkillCategory {
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
        "AI生成完整主图设计执行方案（色值/构图/文案/素材清单），传入product_id自动加载商品信息",
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
                "商品ID，自动加载商品信息和卖点",
                json!({"type": "integer"}),
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
         传入product_id自动加载商品信息",
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
                "商品ID，自动加载商品信息",
                json!({"type": "integer"}),
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
                "商品ID，自动加载类目信息",
                json!({"type": "integer"}),
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
        "商品ID（提供更精准的素材建议）",
        json!({"type": "integer"}),
    ))
    .with_tag("deterministic")
    .with_tag("p1")
}

pub fn design_campaign_poster_spec() -> SkillSpec {
    native_spec(
        "design_campaign_poster",
        "活动海报设计",
        "AI生成完整活动海报设计方案，含色值/文案/构图/各尺寸版本规格，\
         传入product_id自动加载商品信息",
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
                "商品ID，关联主推商品信息",
                json!({"type": "integer"}),
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

macro_rules! native_design_skill {
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
                let mut normalized = Map::new();
                $(normalized.insert($key.to_owned(), json!($value(&params)?));)*
                let mut output = design_native_output(&self.spec.id, &normalized);
                normalize_design_output(&self.spec.id, &params, &mut output);
                Ok(SkillOutcome::new(output).with_summary(design_summary(&self.spec.id)))
            }
        }
    };
}

native_design_skill!(
    DesignMainImage,
    design_main_image_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "简约现代".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "extra_requirements" => |p: &Value| optional_string_param(p, "extra_requirements"),
    }
);
native_design_skill!(
    DesignDetailPage,
    design_detail_page_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "简约现代".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "extra_requirements" => |p: &Value| optional_string_param(p, "extra_requirements"),
    }
);
native_design_skill!(
    DesignColorScheme,
    design_color_scheme_spec,
    required ["brand_tone"],
    defaults {
        "brand_tone" => |p: &Value| string_param(p, "brand_tone").map(Some),
        "category" => |p: &Value| optional_string_param(p, "category"),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "extra_requirements" => |p: &Value| optional_string_param(p, "extra_requirements"),
    }
);
native_design_skill!(
    DesignCampaignPoster,
    design_campaign_poster_spec,
    required ["campaign_name"],
    defaults {
        "campaign_name" => |p: &Value| string_param(p, "campaign_name").map(Some),
        "discount_info" => |p: &Value| Ok(optional_string_param(p, "discount_info")?.unwrap_or_else(|| "全场5折起".to_owned())),
        "campaign_date" => |p: &Value| optional_string_param(p, "campaign_date"),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "热闹促销".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
    }
);

fn normalize_design_output(skill_id: &str, input: &Value, output: &mut Value) {
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
        json!(design_data_readiness(skill_id, input)),
    );
    map.insert("summary".to_owned(), json!(design_summary(skill_id)));
}

fn design_data_readiness(skill_id: &str, input: &Value) -> &'static str {
    match skill_id {
        "design_main_image" | "design_detail_page"
            if !has_any_input(input, &["product_name", "product_id", "extra_requirements"]) =>
        {
            "calculated_from_defaults"
        }
        "design_material_spec" if !has_any_input(input, &["platforms", "product_id"]) => {
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

pub struct DesignMaterialSpec {
    spec: SkillSpec,
}

impl DesignMaterialSpec {
    pub fn new() -> Self {
        Self {
            spec: design_material_spec_spec(),
        }
    }
}

impl Default for DesignMaterialSpec {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for DesignMaterialSpec {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        let usage = string_param(&params, "usage")?;
        let platforms = optional_string_vec_param(&params, "platforms")?
            .unwrap_or_else(|| vec!["淘宝".to_owned(), "京东".to_owned(), "抖音".to_owned()]);
        let product_id = optional_i64_param(&params, "product_id")?;
        let mut output = json!({
            "has_data": true,
            "执行模式": "native_design_spec",
            "数据来源": "用户输入 + 内置平台素材规格规则",
            "用途": usage,
            "目标平台": platforms,
            "商品ID": product_id,
            "平台规格": platforms.iter().map(|platform| material_spec_for(platform, &usage)).collect::<Vec<_>>(),
            "命名规范": "{platform}_{usage}_{product_or_campaign}_{size}_{version}",
            "交付清单": ["源文件", "平台上传图", "压缩备份", "文案备份", "验收截图"],
            "验收标准": ["画面主体清晰", "关键信息在安全区内", "文案无违规词", "导出文件符合平台体积限制", "移动端首屏可读"]
        });
        normalize_design_output("design_material_spec", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("素材规范已生成"))
    }
}

fn design_native_output(skill_id: &str, input: &Map<String, Value>) -> Value {
    match skill_id {
        "design_main_image" => main_image_plan(input),
        "design_detail_page" => detail_page_plan(input),
        "design_color_scheme" => color_scheme_plan(input),
        "design_campaign_poster" => campaign_poster_plan(input),
        _ => json!({"has_data": false, "error": format!("unknown design skill: {skill_id}")}),
    }
}

fn design_summary(skill_id: &str) -> &'static str {
    match skill_id {
        "design_main_image" => "主图设计方案已生成",
        "design_detail_page" => "详情页设计方案已生成",
        "design_color_scheme" => "品牌配色方案已生成",
        "design_material_spec" => "素材规范已生成",
        "design_campaign_poster" => "活动海报设计方案已生成",
        _ => "设计方案已生成",
    }
}

fn main_image_plan(input: &Map<String, Value>) -> Value {
    let product = display_product(input);
    let platform = str_field(input, "platform", "淘宝");
    let style = str_field(input, "style", "简约现代");
    let extra = opt_str_field(input, "extra_requirements");
    json!({
        "has_data": true,
        "执行模式": "native_design_brief",
        "数据来源": "用户输入 + 平台主图规则",
        "商品": product,
        "平台": platform,
        "视觉风格": style,
        "额外要求": extra,
        "主图目标": ["3秒识别品类", "突出核心利益点", "降低点击前理解成本", "适配移动端缩略图"],
        "构图方案": {
            "画面结构": "左上利益点 / 中央商品主体 / 右下信任背书",
            "主体占比": "60%-70%",
            "安全区": "核心文字距边缘不少于8%",
            "视觉动线": "卖点 -> 商品 -> 行动提示"
        },
        "文案层级": [
            {"层级": "主标题", "建议": format!("{} 一眼看懂核心卖点", product)},
            {"层级": "副标题", "建议": "用1句短文案说明适用人群或使用场景"},
            {"层级": "信任背书", "建议": "突出官方、质保、销量、评价或认证中的1-2项"}
        ],
        "色彩与字体": {
            "推荐配色": palette_for_tone(&style),
            "字体建议": "标题用黑体/思源黑体加粗，卖点用中等字重，避免超过3种字体层级"
        },
        "素材清单": ["商品正面透明底图", "场景背景图", "卖点图标", "平台活动角标", "品牌Logo"],
        "平台规格": material_spec_for(&platform, "主图套组"),
        "验收标准": ["缩略图下主标题仍可读", "商品边缘无抠图毛边", "价格/优惠不遮挡主体", "与竞品列表有明显识别差异"]
    })
}

fn detail_page_plan(input: &Map<String, Value>) -> Value {
    let product = display_product(input);
    let platform = str_field(input, "platform", "淘宝");
    let style = str_field(input, "style", "简约现代");
    json!({
        "has_data": true,
        "执行模式": "native_detail_page_brief",
        "数据来源": "用户输入 + 详情页转化结构规则",
        "商品": product,
        "平台": platform,
        "视觉风格": style,
        "页面结构": [
            {"模块": "首屏利益点", "目标": "3秒内说明买点", "内容": ["商品大图", "核心卖点", "适用人群"]},
            {"模块": "痛点对比", "目标": "解释为什么需要它", "内容": ["旧方案问题", "本商品解决方式", "对比图"]},
            {"模块": "功能拆解", "目标": "降低理解成本", "内容": ["3-5个功能点", "局部特写", "使用步骤"]},
            {"模块": "信任证明", "目标": "降低购买风险", "内容": ["材质/认证", "评价摘录", "售后承诺"]},
            {"模块": "规格与FAQ", "目标": "减少客服咨询", "内容": ["参数表", "适配场景", "退换规则"]}
        ],
        "视觉规范": {
            "配色": palette_for_tone(&style),
            "图文比例": "图片70% / 文字30%",
            "移动端字号": "标题不低于28px，正文不低于22px"
        },
        "素材清单": ["首屏场景图", "功能局部图", "对比图", "参数图", "评价/证书截图"],
        "平台规格": material_spec_for(&platform, "详情页"),
        "验收标准": ["首屏无需滑动即可理解卖点", "每屏只讲一个主题", "参数和承诺可被客服复用", "长图加载后不失真"]
    })
}

fn color_scheme_plan(input: &Map<String, Value>) -> Value {
    let tone = str_field(input, "brand_tone", "专业可信");
    let category = str_field(input, "category", "通用消费品");
    let platform = str_field(input, "platform", "淘宝");
    let palette = palette_for_tone(&tone);
    json!({
        "has_data": true,
        "执行模式": "native_color_system",
        "数据来源": "用户输入 + 品牌调性配色规则",
        "品牌调性": tone,
        "商品类目": category,
        "平台": platform,
        "推荐配色": palette,
        "使用比例": [
            {"颜色角色": "主色", "比例": "60%", "用途": "背景、主按钮、关键识别"},
            {"颜色角色": "辅助色", "比例": "25%", "用途": "模块分隔、次级按钮、图标"},
            {"颜色角色": "强调色", "比例": "10%", "用途": "价格、优惠、CTA"},
            {"颜色角色": "中性色", "比例": "5%", "用途": "正文、边框、辅助说明"}
        ],
        "心理暗示": color_psychology(&tone),
        "Figma变量命名": {
            "color.brand.primary": "主色",
            "color.brand.secondary": "辅助色",
            "color.brand.accent": "强调色",
            "color.text.primary": "正文主色"
        },
        "禁忌": ["同屏不要超过2个高饱和主色", "促销强调色只用于CTA和价格", "移动端深底浅字需检查对比度"]
    })
}

fn campaign_poster_plan(input: &Map<String, Value>) -> Value {
    let campaign = str_field(input, "campaign_name", "活动");
    let discount = str_field(input, "discount_info", "限时优惠");
    let style = str_field(input, "style", "热闹促销");
    let platform = str_field(input, "platform", "淘宝");
    let campaign_date = opt_str_field(input, "campaign_date");
    json!({
        "has_data": true,
        "执行模式": "native_campaign_poster_brief",
        "数据来源": "用户输入 + 活动海报结构规则",
        "活动名称": campaign,
        "优惠信息": discount,
        "活动日期": campaign_date,
        "平台": platform,
        "风格": style,
        "海报结构": {
            "顶部": "活动名 + 平台/店铺标识",
            "中部": "主推商品或利益点视觉",
            "底部": "优惠信息 + 时间 + 行动按钮",
            "角标": "库存/限时/赠品等紧迫感信息"
        },
        "文案版本": [
            format!("{campaign}｜{discount}"),
            format!("{discount}，现在下单更划算"),
            "限时权益已开启，错过需等下一场"
        ],
        "色彩方案": palette_for_tone(&style),
        "尺寸版本": [
            material_spec_for(&platform, "活动海报"),
            material_spec_for("抖音", "信息流广告"),
            material_spec_for("小红书", "社交媒体")
        ],
        "验收标准": ["优惠信息一眼可见", "活动时间无遮挡", "商品主体不被角标压住", "不同尺寸裁切后仍保留主信息"]
    })
}

fn material_spec_for(platform: &str, usage: &str) -> Value {
    let normalized_usage = if usage.contains("详情") {
        "详情页"
    } else if usage.contains("信息流") || usage.contains("广告") {
        "信息流广告"
    } else if usage.contains("直播") {
        "直播封面"
    } else if usage.contains("海报") {
        "活动海报"
    } else {
        "主图套组"
    };
    let (size, max_size, safe_area) = match (platform, normalized_usage) {
        ("抖音", "信息流广告") => ("1080x1920", "5MB以内", "上下各保留160px安全区"),
        ("抖音", "直播封面") => ("1080x1920", "5MB以内", "头像/按钮区域避免放核心字"),
        ("小红书", _) => ("1242x1660", "10MB以内", "中心80%为核心内容"),
        ("京东", "详情页") => (
            "宽750px，单屏高度建议900-1200px",
            "单图2MB以内",
            "移动端首屏优先",
        ),
        ("淘宝", "详情页") => (
            "宽790px，单屏高度建议900-1200px",
            "单图3MB以内",
            "顶部首屏必须说明卖点",
        ),
        (_, "活动海报") => ("1080x1440", "5MB以内", "核心文字居中70%区域"),
        (_, "信息流广告") => ("1080x1440", "5MB以内", "核心主体居中，边缘留白"),
        _ => ("800x800", "3MB以内", "主体和主文案放在中心80%区域"),
    };
    json!({
        "平台": platform,
        "用途": normalized_usage,
        "建议尺寸": size,
        "格式": ["JPG", "PNG", "WebP"],
        "文件大小": max_size,
        "安全区": safe_area,
        "导出建议": "sRGB色彩空间，2x源文件备份，上传图按平台体积压缩"
    })
}

fn display_product(input: &Map<String, Value>) -> String {
    opt_str_field(input, "product_name")
        .filter(|value| !value.trim().is_empty())
        .or_else(|| i64_field(input, "product_id").map(|id| format!("商品ID {id}")))
        .unwrap_or_else(|| "主推商品".to_owned())
}

fn str_field(input: &Map<String, Value>, key: &str, default: &str) -> String {
    opt_str_field(input, key).unwrap_or_else(|| default.to_owned())
}

fn opt_str_field(input: &Map<String, Value>, key: &str) -> Option<String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn i64_field(input: &Map<String, Value>, key: &str) -> Option<i64> {
    input.get(key).and_then(Value::as_i64)
}

fn palette_for_tone(tone: &str) -> Value {
    if tone.contains("高端") || tone.contains("奢华") {
        json!([
            {"角色": "主色", "HEX": "#111827", "说明": "深色建立品质感"},
            {"角色": "辅助色", "HEX": "#C8A45D", "说明": "金色用于高级感和重点信息"},
            {"角色": "强调色", "HEX": "#F5F1E8", "说明": "浅底提升留白"},
            {"角色": "文字色", "HEX": "#374151", "说明": "正文稳定可读"}
        ])
    } else if tone.contains("年轻") || tone.contains("活力") || tone.contains("可爱") {
        json!([
            {"角色": "主色", "HEX": "#06B6D4", "说明": "清爽高识别"},
            {"角色": "辅助色", "HEX": "#F97316", "说明": "用于促销和行动提示"},
            {"角色": "强调色", "HEX": "#FEF3C7", "说明": "柔和背景层"},
            {"角色": "文字色", "HEX": "#1F2937", "说明": "控制可读性"}
        ])
    } else if tone.contains("国潮") || tone.contains("复古") {
        json!([
            {"角色": "主色", "HEX": "#B91C1C", "说明": "节庆和国潮识别"},
            {"角色": "辅助色", "HEX": "#0F766E", "说明": "降低红色疲劳"},
            {"角色": "强调色", "HEX": "#FBBF24", "说明": "活动利益点"},
            {"角色": "文字色", "HEX": "#292524", "说明": "复古沉稳"}
        ])
    } else if tone.contains("科技") {
        json!([
            {"角色": "主色", "HEX": "#2563EB", "说明": "科技和信任感"},
            {"角色": "辅助色", "HEX": "#14B8A6", "说明": "参数和功能模块"},
            {"角色": "强调色", "HEX": "#F8FAFC", "说明": "干净背景"},
            {"角色": "文字色", "HEX": "#0F172A", "说明": "高对比正文"}
        ])
    } else {
        json!([
            {"角色": "主色", "HEX": "#2563EB", "说明": "稳定可信"},
            {"角色": "辅助色", "HEX": "#10B981", "说明": "品质和服务承诺"},
            {"角色": "强调色", "HEX": "#F59E0B", "说明": "价格和行动提示"},
            {"角色": "文字色", "HEX": "#111827", "说明": "正文可读"}
        ])
    }
}

fn color_psychology(tone: &str) -> &'static str {
    if tone.contains("高端") || tone.contains("奢华") {
        "深色和金色组合会强化稀缺、品质和礼赠感，适合客单价较高的商品。"
    } else if tone.contains("年轻") || tone.contains("活力") || tone.contains("可爱") {
        "高明度冷色配暖色强调能提升轻松感和点击意愿，适合内容种草和新品测试。"
    } else if tone.contains("国潮") || tone.contains("复古") {
        "红、绿、金组合能建立节庆和文化记忆点，但需要控制饱和度避免廉价感。"
    } else if tone.contains("科技") {
        "蓝绿组合有利于表达效率、参数、理性和可信赖，适合数码、家电和功能型商品。"
    } else {
        "蓝绿橙组合兼顾信任、品质和行动提示，适合通用电商转化页面。"
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
    async fn design_main_image_discloses_default_product() {
        let outcome = DesignMainImage::new()
            .run(json!({}), SkillContext::default())
            .await
            .expect("main image plan should run with defaults");

        assert_success_contract(
            &outcome.output,
            "主图设计方案已生成",
            "calculated_from_defaults",
        );
        assert_eq!(
            outcome.output.get("商品").and_then(Value::as_str),
            Some("主推商品")
        );
    }

    #[tokio::test]
    async fn design_color_scheme_marks_user_provided_tone() {
        let outcome = DesignColorScheme::new()
            .run(json!({"brand_tone": "科技感"}), SkillContext::default())
            .await
            .expect("color scheme should run with brand tone");

        assert_success_contract(&outcome.output, "品牌配色方案已生成", "user_provided");
        assert_eq!(
            outcome.output.get("品牌调性").and_then(Value::as_str),
            Some("科技感")
        );
    }

    #[tokio::test]
    async fn design_material_spec_discloses_default_platforms() {
        let outcome = DesignMaterialSpec::new()
            .run(json!({"usage": "主图套组"}), SkillContext::default())
            .await
            .expect("material spec should run with usage");

        assert_success_contract(
            &outcome.output,
            "素材规范已生成",
            "calculated_from_defaults",
        );
        assert!(
            outcome
                .output
                .get("平台规格")
                .and_then(Value::as_array)
                .is_some_and(|items| items.len() == 3)
        );
    }

    #[tokio::test]
    async fn design_campaign_poster_marks_user_provided_campaign() {
        let outcome = DesignCampaignPoster::new()
            .run(json!({"campaign_name": "618大促"}), SkillContext::default())
            .await
            .expect("campaign poster should run with campaign name");

        assert_success_contract(&outcome.output, "活动海报设计方案已生成", "user_provided");
        assert_eq!(
            outcome.output.get("活动名称").and_then(Value::as_str),
            Some("618大促")
        );
    }
}
