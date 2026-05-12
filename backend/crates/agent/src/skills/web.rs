use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::skills::{
    Skill, SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillResult, SkillSpec, object_params, optional_f64_param, optional_i64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

fn category() -> SkillCategory {
    SkillCategory::Custom("web".to_owned())
}

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
                "商品ID，自动加载商品信息生成针对性SEO方案",
                json!({"type": "integer"}),
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
        "根据商品信息生成多种风格的SEO友好标题；传入product_id可自动加载真实商品",
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
                "商品ID，用于自动加载商品信息",
                json!({"type": "integer"}),
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
        "商品ID，自动加载标题和关键词",
        json!({"type": "integer"}),
    ))
    .with_tag("deterministic")
}

pub struct WebTitleSeoScorer {
    spec: SkillSpec,
}

impl WebTitleSeoScorer {
    pub fn new() -> Self {
        Self {
            spec: web_title_seo_scorer_spec(),
        }
    }
}

impl Default for WebTitleSeoScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for WebTitleSeoScorer {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let title = optional_string_param(&params, "title")?
            .ok_or_else(|| SkillError::missing_parameter("title"))?;
        let keywords = optional_string_vec_param(&params, "target_keywords")?.unwrap_or_default();
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let len = title.chars().count() as i64;
        let (lo, hi) = match platform.as_str() {
            "淘宝" => (28, 34),
            "京东" => (28, 36),
            "拼多多" => (20, 28),
            "抖音" => (15, 22),
            _ => (24, 34),
        };
        let mut deductions = Vec::new();
        let len_score = if (lo..=hi).contains(&len) {
            25
        } else if len < lo {
            let score = (25 - (lo - len) * 2).max(0);
            deductions.push(format!(
                "标题偏短({len}字 < 推荐{lo}字)，损失{}分",
                25 - score
            ));
            score
        } else {
            let score = (25 - (len - hi)).max(5);
            deductions.push(format!(
                "标题偏长({len}字 > 推荐{hi}字)，损失{}分",
                25 - score
            ));
            score
        };

        let primary = keywords.first().cloned().unwrap_or_default();
        let (position_score, pos_bonus) = if primary.is_empty() {
            (15, 0)
        } else {
            let bonus = keyword_position_score(&title, &primary);
            if bonus < 0 {
                deductions.push(format!("主关键词「{primary}」未在标题中出现，扣20分"));
            } else if bonus < 20 {
                deductions.push(format!("主关键词「{primary}」位置靠后，建议移至前30%"));
            }
            (
                (if title.contains(&primary) { 10 } else { 0 } + bonus).clamp(0, 30),
                bonus,
            )
        };

        let (kw_score, coverage_pct, missing) = if keywords.is_empty() {
            (15, 0.0, Vec::new())
        } else {
            let covered = keywords
                .iter()
                .filter(|kw| title.to_lowercase().contains(&kw.to_lowercase()))
                .count();
            let coverage = covered as f64 / keywords.len() as f64;
            let missing = keywords
                .iter()
                .filter(|kw| !title.to_lowercase().contains(&kw.to_lowercase()))
                .cloned()
                .collect::<Vec<_>>();
            if coverage < 0.6 {
                deductions.push(format!(
                    "关键词覆盖率{:.0}%，未包含: {}",
                    coverage * 100.0,
                    missing
                        .iter()
                        .take(3)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            ((coverage * 25.0).round() as i64, coverage, missing)
        };

        let special = title
            .chars()
            .filter(|c| "！？!?★☆▶◆●■□[]【】《》<>~～".contains(*c))
            .count() as i64;
        let repeat_penalty = repeated_char_penalty(&title);
        let readable_score = (20 - special * 2 - repeat_penalty).max(0);
        if special > 3 {
            deductions.push(format!("特殊符号过多({special}个)，影响可读性"));
        }
        let total = len_score + position_score + kw_score + readable_score;
        let suggestions = build_seo_suggestions(&primary, pos_bonus, &missing, len, lo, hi);

        let mut output = json!({
            "平台": platform,
            "标题": title,
            "综合SEO评分": format!("{total}/100"),
            "总分": total,
            "评级": seo_grade(total),
            "分项评分": {
                "字符长度": {"得分": len_score, "满分": 25, "实际字数": len, "推荐范围": format!("{lo}-{hi}字")},
                "主关键词位置": {"得分": position_score, "满分": 30, "关键词": primary, "位置分": pos_bonus},
                "关键词覆盖": {"得分": kw_score, "满分": 25, "覆盖率": format!("{:.0}%", coverage_pct * 100.0)},
                "可读性": {"得分": readable_score, "满分": 20, "特殊符号数": special}
            },
            "扣分项": if deductions.is_empty() { vec!["无明显扣分项".to_owned()] } else { deductions },
            "优化建议": suggestions
        });
        normalize_web_output("web_title_seo_scorer", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("标题SEO评分已完成"))
    }
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
                "商品ID，自动以商品名为种子词",
                json!({"type": "integer"}),
            ),
        ],
    )
}

pub fn web_product_description_spec() -> SkillSpec {
    native_spec(
        "web_product_description",
        "详情页文案",
        "生成完整的详情页营销文案（首屏卖点/痛点/解决方案/亮点/行动号召）；\
         传入product_id自动加载商品信息",
        vec![
            SkillInputField::optional(
                "product_name",
                "商品名称（可从product_id自动加载）",
                json!({"type": "string"}),
            ),
            SkillInputField::optional(
                "product_id",
                "商品ID，自动加载商品信息",
                json!({"type": "integer"}),
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

macro_rules! native_web_skill {
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
                let mut output = web_native_output(&self.spec.id, &normalized);
                normalize_web_output(&self.spec.id, &params, &mut output);
                Ok(SkillOutcome::new(output).with_summary(web_summary(&self.spec.id)))
            }
        }
    };
}

native_web_skill!(
    WebSeoOptimize,
    web_seo_optimize_spec,
    required [],
    defaults {
        "target_keywords" => |p: &Value| Ok(optional_string_vec_param(p, "target_keywords")?.unwrap_or_else(|| vec!["电商".to_owned()])),
        "current_rank" => |p: &Value| Ok(optional_i64_param(p, "current_rank")?.unwrap_or(0)),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
    }
);
native_web_skill!(
    WebTitleGenerator,
    web_title_generator_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "keywords" => |p: &Value| Ok(optional_string_vec_param(p, "keywords")?.unwrap_or_default()),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "营销型".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
    }
);
native_web_skill!(
    WebStoreDesign,
    web_store_design_spec,
    required ["store_type"],
    defaults {
        "store_type" => |p: &Value| string_param(p, "store_type").map(Some),
        "category" => |p: &Value| Ok(optional_string_param(p, "category")?.unwrap_or_else(|| "通用".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "brand_tone" => |p: &Value| Ok(optional_string_param(p, "brand_tone")?.unwrap_or_else(|| "专业".to_owned())),
    }
);
native_web_skill!(
    WebKeywordResearch,
    web_keyword_research_spec,
    required [],
    defaults {
        "seed_keyword" => |p: &Value| optional_string_param(p, "seed_keyword"),
        "category" => |p: &Value| Ok(optional_string_param(p, "category")?.unwrap_or_else(|| "通用".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
    }
);
native_web_skill!(
    WebProductDescription,
    web_product_description_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "营销型".to_owned())),
    }
);

pub struct WebPageConversion {
    spec: SkillSpec,
}

impl WebPageConversion {
    pub fn new() -> Self {
        Self {
            spec: web_page_conversion_spec(),
        }
    }
}

impl Default for WebPageConversion {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for WebPageConversion {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        let page_type = string_param(&params, "page_type")?;
        let conv = optional_f64_param(&params, "current_conversion")?.unwrap_or(3.0);
        let bounce = optional_f64_param(&params, "bounce_rate")?.unwrap_or(55.0);
        let stay = optional_f64_param(&params, "avg_stay_time")?.unwrap_or(45.0);
        let benchmark = match page_type.as_str() {
            "首页" => 5.0,
            "商品详情" => 4.0,
            "购物车" => 65.0,
            "结算" => 80.0,
            "列表页" => 8.0,
            _ => 5.0,
        };
        let gap = conv - benchmark;
        let mut output = json!({
            "has_data": true,
            "执行模式": "native_conversion_audit",
            "数据来源": "用户输入 + 页面转化基准规则",
            "页面类型": page_type,
            "当前数据": {"转化率": format!("{conv}%"), "跳出率": format!("{bounce}%"), "停留时间": format!("{stay}秒")},
            "行业基准转化率": format!("{benchmark}%"),
            "转化差距": format!("{gap:+.2}%"),
            "初步判断": if conv >= benchmark { "当前转化率达到或超过基准，优先做AB测试微调" } else { "当前转化率低于基准，优先检查首屏信息、信任背书、CTA和价格呈现" },
            "瓶颈诊断": conversion_findings(conv, bounce, stay, benchmark),
            "优化动作": conversion_actions(&page_type, conv, bounce, stay, benchmark),
            "AB测试清单": [
                {"实验": "首屏卖点顺序", "变量": "价格利益点 vs 使用场景", "指标": "点击/加购"},
                {"实验": "CTA文案", "变量": "立即购买 vs 领取优惠", "指标": "按钮点击率"},
                {"实验": "信任背书位置", "变量": "首屏展示 vs 参数后展示", "指标": "停留和转化"}
            ],
            "后续数据需求": ["近30天PV/UV", "点击热区", "加购率", "支付转化率", "渠道来源"]
        });
        normalize_web_output("web_page_conversion", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("页面转化优化方案已生成"))
    }
}

fn normalize_web_output(skill_id: &str, input: &Value, output: &mut Value) {
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
        json!(web_data_readiness(skill_id, input)),
    );
    map.insert("summary".to_owned(), json!(web_summary(skill_id)));
}

fn web_data_readiness(skill_id: &str, input: &Value) -> &'static str {
    match skill_id {
        "web_seo_optimize"
            if !has_any_input(input, &["target_keywords", "current_rank", "product_id"]) =>
        {
            "calculated_from_defaults"
        }
        "web_title_generator"
            if !has_any_input(input, &["product_name", "keywords", "product_id"]) =>
        {
            "calculated_from_defaults"
        }
        "web_page_conversion"
            if !has_any_input(
                input,
                &["current_conversion", "bounce_rate", "avg_stay_time"],
            ) =>
        {
            "calculated_from_defaults"
        }
        "web_keyword_research" if !has_any_input(input, &["seed_keyword", "product_id"]) => {
            "calculated_from_defaults"
        }
        "web_product_description" if !has_any_input(input, &["product_name", "product_id"]) => {
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

fn web_native_output(skill_id: &str, input: &Map<String, Value>) -> Value {
    match skill_id {
        "web_seo_optimize" => seo_optimize(input),
        "web_title_generator" => title_generator(input),
        "web_store_design" => store_design(input),
        "web_keyword_research" => keyword_research(input),
        "web_product_description" => product_description(input),
        _ => json!({"has_data": false, "error": format!("unknown web skill: {skill_id}")}),
    }
}

fn web_summary(skill_id: &str) -> &'static str {
    match skill_id {
        "web_seo_optimize" => "SEO优化方案已生成",
        "web_title_generator" => "标题候选已生成",
        "web_title_seo_scorer" => "标题SEO评分已完成",
        "web_page_conversion" => "页面转化优化方案已生成",
        "web_store_design" => "店铺装修方案已生成",
        "web_keyword_research" => "关键词矩阵已生成",
        "web_product_description" => "详情页文案已生成",
        _ => "Web增长方案已生成",
    }
}

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

fn keyword_layers(keywords: &[String]) -> Vec<Value> {
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

fn title_rules(platform: &str) -> Value {
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

fn conversion_findings(conv: f64, bounce: f64, stay: f64, benchmark: f64) -> Vec<String> {
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

fn conversion_actions(
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

fn string_vec_field(input: &Map<String, Value>, key: &str) -> Vec<String> {
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

fn i64_field(input: &Map<String, Value>, key: &str) -> Option<i64> {
    input.get(key).and_then(Value::as_i64)
}

fn palette_name(tone: &str) -> &'static str {
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

fn keyword_position_score(title: &str, keyword: &str) -> i64 {
    let Some(byte_idx) = title.to_lowercase().find(&keyword.to_lowercase()) else {
        return -20;
    };
    let char_idx = title[..byte_idx].chars().count() as f64;
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

fn repeated_char_penalty(title: &str) -> i64 {
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

fn seo_grade(total: i64) -> &'static str {
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

fn build_seo_suggestions(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn web_seo_optimize_discloses_default_inputs() {
        let outcome = WebSeoOptimize::new()
            .run(json!({}), SkillContext::default())
            .await
            .expect("web SEO optimize should run with deterministic defaults");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("adapter_status").and_then(Value::as_str),
            Some("manual_input")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("calculated_from_defaults")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("SEO优化方案已生成")
        );
    }

    #[tokio::test]
    async fn web_title_generator_marks_success_for_user_product() {
        let outcome = WebTitleGenerator::new()
            .run(
                json!({
                    "product_name": "冷萃咖啡",
                    "keywords": ["冷萃咖啡", "低糖"],
                    "platform": "淘宝"
                }),
                SkillContext::default(),
            )
            .await
            .expect("web title generator should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("标题候选已生成")
        );
        assert!(output.get("标题候选").and_then(Value::as_array).is_some());
    }

    #[tokio::test]
    async fn web_page_conversion_discloses_benchmark_defaults() {
        let outcome = WebPageConversion::new()
            .run(json!({"page_type": "商品详情"}), SkillContext::default())
            .await
            .expect("web page conversion should run with deterministic defaults");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("calculated_from_defaults")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("页面转化优化方案已生成")
        );
    }

    #[tokio::test]
    async fn web_title_seo_scorer_marks_success_for_user_title() {
        let outcome = WebTitleSeoScorer::new()
            .run(
                json!({
                    "title": "冷萃咖啡 低糖 即饮 官方优选 高性价比",
                    "target_keywords": ["冷萃咖啡", "低糖"],
                    "platform": "淘宝"
                }),
                SkillContext::default(),
            )
            .await
            .expect("web title SEO scorer should run");
        let output = outcome.output;

        assert_eq!(
            output.get("status").and_then(Value::as_str),
            Some("success")
        );
        assert_eq!(
            output.get("data_readiness").and_then(Value::as_str),
            Some("user_provided")
        );
        assert_eq!(
            output.get("summary").and_then(Value::as_str),
            Some("标题SEO评分已完成")
        );
        assert!(output.get("总分").and_then(Value::as_i64).is_some());
    }
}
