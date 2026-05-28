use std::collections::BTreeSet;

use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::{
        compact_text, evidence_segments, meaningful_segments, metadata_refs, metadata_text,
        metadata_usize, normalized_optional_text,
    },
    specs::material_image_diagnosis_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_i64_param,
    optional_string_param, optional_string_vec_param,
};

const MAX_ANALYSIS_CHARS: usize = 6_000;
const MAX_DISPLAY_CHARS: usize = 420;

pub struct MaterialImageDiagnosis {
    spec: SkillSpec,
}

impl MaterialImageDiagnosis {
    pub fn new() -> Self {
        Self {
            spec: material_image_diagnosis_spec(),
        }
    }
}

impl Default for MaterialImageDiagnosis {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for MaterialImageDiagnosis {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let image_name = normalized_optional_text(optional_string_param(&params, "image_name")?)
            .or_else(|| {
                metadata_text(
                    &context,
                    &[
                        "request.attachments.image_titles",
                        "attachments.image_titles",
                        "request.context.source_titles",
                        "context.source_titles",
                    ],
                )
            });
        let user_question =
            normalized_optional_text(optional_string_param(&params, "user_question")?)
                .or_else(|| metadata_text(&context, &["task_description"]));
        let provided_image_type =
            normalized_optional_text(optional_string_param(&params, "image_type")?);
        let image_type = infer_image_type(
            provided_image_type.as_deref(),
            image_name.as_deref(),
            user_question.as_deref(),
        );
        let vision_summary =
            normalized_optional_text(optional_string_param(&params, "vision_summary")?).or_else(
                || {
                    metadata_text(
                        &context,
                        &[
                            "request.context.vision_summary",
                            "context.vision_summary",
                            "request.attachments.image_summaries_json",
                            "attachments.image_summaries_json",
                            "request.context.image_summaries_json",
                            "context.image_summaries_json",
                            "request.context.combined_summary",
                            "context.combined_summary",
                        ],
                    )
                },
            );
        let ocr_text = normalized_optional_text(optional_string_param(&params, "ocr_text")?)
            .or_else(|| {
                metadata_text(
                    &context,
                    &[
                        "request.attachments.ocr_text",
                        "attachments.ocr_text",
                        "request.context.ocr_text",
                        "context.ocr_text",
                        "request.ocr_text",
                    ],
                )
            });
        let image_notes = normalized_optional_text(optional_string_param(&params, "image_notes")?)
            .or_else(|| {
                metadata_text(
                    &context,
                    &[
                        "request.context.image_notes",
                        "context.image_notes",
                        "request.image_notes",
                    ],
                )
            });
        let mut context_refs = optional_string_vec_param(&params, "context_refs")?
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| normalized_optional_text(Some(value)))
            .collect::<Vec<_>>();
        context_refs.extend(metadata_refs(
            &context,
            &[
                "request.context.refs",
                "context.refs",
                "request.context.combined_refs",
                "context.combined_refs",
                "request.attachments.image_refs",
                "attachments.image_refs",
            ],
        ));
        dedupe(&mut context_refs);
        let image_count = optional_i64_param(&params, "image_count")?
            .and_then(|value| usize::try_from(value.max(0)).ok())
            .or_else(|| {
                metadata_usize(
                    &context,
                    &[
                        "request.attachments.image_count",
                        "attachments.image_count",
                        "request.context.image_count",
                        "context.image_count",
                        "request.context.source_count",
                        "context.source_count",
                    ],
                )
            })
            .unwrap_or(context_refs.len());

        let has_visual_summary = vision_summary.is_some() || image_notes.is_some();
        let has_ocr = ocr_text.is_some();
        let has_context = has_visual_summary || has_ocr;
        let data_readiness = if has_visual_summary {
            "visual_summary_available"
        } else if has_ocr {
            "ocr_only"
        } else if image_name.is_some() || image_count > 0 || !context_refs.is_empty() {
            "metadata_only"
        } else {
            "missing_visual_context"
        };
        let status = if has_context { "success" } else { "deferred" };
        let visual_text = join_parts(&[
            vision_summary.as_deref().unwrap_or_default(),
            image_notes.as_deref().unwrap_or_default(),
        ]);
        let combined = join_parts(&[
            visual_text.as_str(),
            ocr_text.as_deref().unwrap_or_default(),
        ]);
        let analysis_text = compact_text(&combined, MAX_ANALYSIS_CHARS);
        let question = user_question.as_deref().unwrap_or_default();
        let visible_summary = visible_summary_text(
            has_visual_summary,
            has_ocr,
            visual_text.as_str(),
            ocr_text.as_deref(),
        );
        let ocr_display = ocr_text
            .as_deref()
            .map(|text| compact_text(text, MAX_DISPLAY_CHARS))
            .unwrap_or_else(|| "未收到 OCR 文字。".to_owned());
        let dimensions = diagnosis_dimensions(
            &image_type,
            visual_text.as_str(),
            ocr_text.as_deref().unwrap_or_default(),
            has_visual_summary,
            has_ocr,
        );
        let opportunities =
            conversion_opportunities(&image_type, visual_text.as_str(), ocr_text.as_deref());
        let risk_items = risks_and_limits(
            has_context,
            has_visual_summary,
            has_ocr,
            &image_type,
            &analysis_text,
            &context_refs,
        );
        let downstream_skills =
            downstream_skill_suggestions(&image_type, &analysis_text, question, has_context);
        let next_action_items = next_actions(
            has_context,
            has_visual_summary,
            has_ocr,
            &image_type,
            &analysis_text,
        );
        let evidence = if has_context {
            evidence_segments(&analysis_text, question, 5)
        } else {
            Vec::new()
        };
        let summary_line = match data_readiness {
            "visual_summary_available" => format!(
                "已基于图片视觉摘要/OCR完成诊断：{}，识别 {} 个诊断维度、{} 条风险或限制。",
                display_image_name(image_name.as_deref()),
                dimensions.len(),
                risk_items.len()
            ),
            "ocr_only" => format!(
                "已基于图片 OCR 做文字层诊断：{}，但缺少画面摘要，不能判断构图和主体细节。",
                display_image_name(image_name.as_deref())
            ),
            "metadata_only" => format!(
                "只收到图片元数据：{}，尚未收到视觉摘要、OCR 或图片说明。",
                display_image_name(image_name.as_deref())
            ),
            _ => "未收到可诊断的图片内容、OCR 或图片引用。".to_owned(),
        };
        let output = json!({
            "status": status,
            "has_data": has_context,
            "adapter_status": "manual_or_context_input",
            "data_readiness": data_readiness,
            "summary": summary_line.clone(),
            "skill": self.spec.id,
            "图片名称": image_name,
            "图片类型": image_type,
            "图片数量": image_count,
            "资料引用": context_refs,
            "可见信息摘要": visible_summary,
            "OCR文字": ocr_display,
            "可引用证据": evidence,
            "诊断维度": dimensions,
            "卖点与转化机会": opportunities,
            "风险与限制": risk_items,
            "后续适配 Skill": downstream_skills,
            "下一步建议": next_action_items,
            "治理说明": [
                "本 skill 不直接读取原始图片像素，只诊断本轮显式传入的视觉摘要、OCR、图片说明或上下文元数据。",
                "当 data_readiness=ocr_only 时，只能评价文字表达和疑似合规词，不能断言主体、构图、色彩或视觉层级。",
                "当 status=deferred 时，调用方应先完成图片 OCR、视觉模型摘要或资料上下文打包，再让设计/创意/运营岗位继续回答。"
            ]
        });

        Ok(SkillOutcome::new(output).with_summary(summary_line))
    }
}

fn diagnosis_dimensions(
    image_type: &str,
    visual_text: &str,
    ocr_text: &str,
    has_visual_summary: bool,
    has_ocr: bool,
) -> Vec<Value> {
    let combined = join_parts(&[visual_text, ocr_text]);
    vec![
        subject_clarity_dimension(visual_text, has_visual_summary),
        selling_point_dimension(image_type, &combined, has_visual_summary || has_ocr),
        readability_dimension(visual_text, ocr_text, has_ocr),
        price_promo_dimension(image_type, &combined, has_visual_summary || has_ocr),
        compliance_dimension(&combined, has_visual_summary || has_ocr),
        conversion_blocker_dimension(
            image_type,
            visual_text,
            ocr_text,
            has_visual_summary,
            has_ocr,
        ),
    ]
}

fn subject_clarity_dimension(visual_text: &str, has_visual_summary: bool) -> Value {
    if !has_visual_summary {
        return dimension(
            "主体清晰度",
            "unknown",
            vec!["缺少视觉摘要，不能判断商品主体、构图和清晰度。".to_owned()],
            "先调用视觉模型生成主体、背景、构图和层级摘要。",
        );
    }

    let warning_keywords = [
        "模糊",
        "看不清",
        "遮挡",
        "过暗",
        "暗",
        "杂乱",
        "拥挤",
        "反光",
        "低清",
        "小字",
    ];
    if contains_any(visual_text, &warning_keywords) {
        return dimension(
            "主体清晰度",
            "warning",
            keyword_evidence(
                visual_text,
                &warning_keywords,
                "视觉摘要提到清晰度或层级问题。",
            ),
            "优先放大商品主体，减少背景干扰，保证首屏能一眼识别商品和核心利益点。",
        );
    }

    dimension(
        "主体清晰度",
        "good",
        meaningful_segments(visual_text, 2),
        "继续保持主体清晰，同时用卖点、价格或使用场景补强转化理由。",
    )
}

fn selling_point_dimension(image_type: &str, text: &str, has_context: bool) -> Value {
    if !has_context {
        return dimension(
            "卖点呈现",
            "unknown",
            vec!["缺少视觉摘要或 OCR，不能判断是否呈现卖点。".to_owned()],
            "先补充图片 OCR、视觉摘要或商品卖点说明。",
        );
    }

    let keywords = [
        "卖点",
        "低糖",
        "0脂",
        "零脂",
        "新品",
        "爆款",
        "官方",
        "限时",
        "舒缓",
        "修护",
        "高蛋白",
        "便携",
        "耐用",
        "包邮",
        "赠",
    ];
    if contains_any(text, &keywords) {
        return dimension(
            "卖点呈现",
            "good",
            keyword_evidence(text, &keywords, "已出现卖点或利益点表达。"),
            "把最强卖点放在视觉第一层级，并让标题、价格或场景图共同支撑该卖点。",
        );
    }

    let advice = if matches!(image_type, "main_image" | "poster") {
        "主图/海报建议至少出现一个清晰利益点，例如核心功效、适用人群、活动利益或差异化证据。"
    } else {
        "补充可被用户快速理解的商品利益点，避免只展示素材而没有购买理由。"
    };
    dimension(
        "卖点呈现",
        "warning",
        vec!["已提供内容中未识别到明确卖点词或利益点。".to_owned()],
        advice,
    )
}

fn readability_dimension(visual_text: &str, ocr_text: &str, has_ocr: bool) -> Value {
    if !has_ocr {
        return dimension(
            "文案可读性",
            "unknown",
            vec!["缺少 OCR 文字，不能判断图片文案长度、层级和可读性。".to_owned()],
            "先做 OCR；如果图片用于主图或海报，应同时检查手机端缩略图可读性。",
        );
    }

    let warning_keywords = ["小字", "看不清", "拥挤", "密集", "过多", "模糊"];
    if ocr_text.chars().count() > 80 || contains_any(visual_text, &warning_keywords) {
        return dimension(
            "文案可读性",
            "warning",
            vec![format!(
                "OCR 文字约 {} 个字符，或视觉摘要提示文案密度/可读性问题。",
                ocr_text.chars().count()
            )],
            "压缩主视觉文案，只保留一个主卖点、一个辅助证明和必要价格/活动信息。",
        );
    }

    dimension(
        "文案可读性",
        "good",
        vec![compact_text(ocr_text, 160)],
        "继续保持短句和层级清晰；上线前仍需用移动端预览验证小字号。",
    )
}

fn price_promo_dimension(image_type: &str, text: &str, has_context: bool) -> Value {
    if !has_context {
        return dimension(
            "价格/促销表达",
            "unknown",
            vec!["缺少视觉摘要或 OCR，不能判断价格、券、满减或活动利益。".to_owned()],
            "如果该图承担转化任务，应补充价格/活动机制的 OCR 或摘要。",
        );
    }

    let keywords = [
        "价格", "售价", "元", "￥", "¥", "折", "券", "满减", "立减", "包邮", "赠",
    ];
    if contains_any(text, &keywords) {
        return dimension(
            "价格/促销表达",
            "good",
            keyword_evidence(text, &keywords, "已出现价格或促销表达。"),
            "检查价格、券和门槛是否同层级呈现，避免用户只看到低价但看不清条件。",
        );
    }

    let level = if matches!(image_type, "main_image" | "poster") {
        "warning"
    } else {
        "unknown"
    };
    dimension(
        "价格/促销表达",
        level,
        vec!["已提供内容中未识别到价格、券、满减或活动利益。".to_owned()],
        "如果目标是成交转化，建议明确价格锚点或促销利益；如果目标是种草，\
         可弱化价格但补充信任证据。",
    )
}

fn compliance_dimension(text: &str, has_context: bool) -> Value {
    if !has_context {
        return dimension(
            "平台合规风险",
            "unknown",
            vec!["缺少图片文字和视觉摘要，无法做合规词初筛。".to_owned()],
            "先补充 OCR，再结合平台规则或 search_platform_policy 做进一步校验。",
        );
    }

    let risk_keywords = [
        "最",
        "第一",
        "全网最低",
        "永久",
        "绝对",
        "治愈",
        "治疗",
        "疗效",
        "官方认证",
        "100%",
        "无效退款",
    ];
    if contains_any(text, &risk_keywords) {
        return dimension(
            "平台合规风险",
            "warning",
            keyword_evidence(
                text,
                &risk_keywords,
                "已识别到疑似绝对化、功效化或强承诺表达。",
            ),
            "把绝对化和医疗/功效承诺改成可证明、可引用、可限定范围的表达，\
             并继续调用平台规则搜索核验。",
        );
    }

    dimension(
        "平台合规风险",
        "good",
        vec!["已提供文本中未识别到高风险绝对化或功效化关键词。".to_owned()],
        "这只是关键词初筛，不替代平台规则、类目规则和法务审核。",
    )
}

fn conversion_blocker_dimension(
    image_type: &str,
    visual_text: &str,
    ocr_text: &str,
    has_visual_summary: bool,
    has_ocr: bool,
) -> Value {
    if !has_visual_summary && !has_ocr {
        return dimension(
            "转化阻碍",
            "unknown",
            vec!["缺少视觉摘要和 OCR，不能判断点击或成交阻碍。".to_owned()],
            "先生成图片摘要，再按主体、卖点、信任证据、价格活动和行动入口逐项检查。",
        );
    }

    let text = join_parts(&[visual_text, ocr_text]);
    let visual_bad = contains_any(
        visual_text,
        &["模糊", "看不清", "遮挡", "杂乱", "拥挤", "过暗"],
    );
    let lacks_selling_point = !contains_any(
        &text,
        &[
            "卖点", "低糖", "0脂", "新品", "爆款", "舒缓", "修护", "包邮", "赠", "限时",
        ],
    );
    let lacks_price = !contains_any(&text, &["价格", "售价", "元", "￥", "¥", "券", "折"]);
    if visual_bad || (matches!(image_type, "main_image" | "poster") && lacks_selling_point) {
        let mut evidence = Vec::new();
        if visual_bad {
            evidence.push("视觉摘要提示清晰度、遮挡或层级问题。".to_owned());
        }
        if lacks_selling_point {
            evidence.push("未识别到明确卖点或利益点。".to_owned());
        }
        if matches!(image_type, "main_image" | "poster") && lacks_price {
            evidence.push("未识别到价格或活动利益，转化型图片可能缺少行动理由。".to_owned());
        }
        return dimension(
            "转化阻碍",
            "warning",
            evidence,
            "优先补强首屏识别、利益点和信任证据；转化型素材再补充价格/券/活动门槛。",
        );
    }

    dimension(
        "转化阻碍",
        "good",
        vec!["已提供信息中暂未发现明显转化阻碍。".to_owned()],
        "继续用点击率、加购率、停留时长或 A/B 测试验证真实效果。",
    )
}

fn dimension(dimension: &str, level: &str, evidence: Vec<String>, advice: &str) -> Value {
    json!({
        "dimension": dimension,
        "level": level,
        "evidence": evidence,
        "advice": advice
    })
}

fn conversion_opportunities(
    image_type: &str,
    visual_text: &str,
    ocr_text: Option<&str>,
) -> Vec<String> {
    let text = join_parts(&[visual_text, ocr_text.unwrap_or_default()]);
    let mut items = Vec::new();
    if !contains_any(
        &text,
        &[
            "卖点", "低糖", "0脂", "新品", "爆款", "舒缓", "修护", "便携", "包邮", "赠",
        ],
    ) {
        items.push("补充一个用户能立即理解的核心卖点，并让视觉第一层级服务这个卖点。".to_owned());
    }
    if matches!(image_type, "main_image" | "poster")
        && !contains_any(
            &text,
            &["价格", "售价", "元", "￥", "¥", "券", "折", "满减"],
        )
    {
        items
            .push("转化型主图/海报可增加价格锚点、券后价或活动利益，但要把门槛写清楚。".to_owned());
    }
    if ocr_text.is_some_and(|text| text.chars().count() > 80) {
        items.push("OCR 文案偏长，建议压缩为主标题、辅助证明、行动信息三层。".to_owned());
    }
    match image_type {
        "detail_page" => items.push(
            "详情页截图适合按痛点、卖点、证据、场景、规格、保障的顺序补齐信息链。".to_owned(),
        ),
        "poster" => items.push(
            "海报应突出活动主题、核心利益、时间节点和参与门槛，\
             避免用户只看到气氛但不知道怎么行动。"
                .to_owned(),
        ),
        "screenshot" => items.push(
            "后台或页面截图应先标注页面来源、关键字段和异常位置，再交给运营/数据岗位继续判断。"
                .to_owned(),
        ),
        _ => {}
    }
    if items.is_empty() {
        items.push(
            "当前摘要已包含基础转化信息，建议继续用设计 Skill 生成改版方向并做 A/B 验证。"
                .to_owned(),
        );
    }
    items
}

fn risks_and_limits(
    has_context: bool,
    has_visual_summary: bool,
    has_ocr: bool,
    image_type: &str,
    text: &str,
    refs: &[String],
) -> Vec<String> {
    let mut risks = Vec::new();
    if !has_context {
        risks.push("未收到视觉摘要、OCR 或图片说明，不能把文件名当成图片内容来诊断。".to_owned());
    }
    if !has_visual_summary && has_ocr {
        risks.push(
            "当前只有 OCR 文字，不能判断商品主体、构图、色彩、视觉层级或图片清晰度。".to_owned(),
        );
    }
    if has_visual_summary && !has_ocr {
        risks.push("缺少 OCR 文字，文案可读性、价格表达和合规词只能做弱判断。".to_owned());
    }
    if contains_any(
        text,
        &[
            "最",
            "第一",
            "全网最低",
            "绝对",
            "治愈",
            "治疗",
            "官方认证",
            "100%",
        ],
    ) {
        risks.push("疑似存在绝对化、功效化或强承诺表达，需要平台规则和人工复核。".to_owned());
    }
    if refs.is_empty() {
        risks.push("缺少明确图片引用，后续多图对比和证据追踪会变弱。".to_owned());
    }
    if image_type == "unknown" {
        risks.push(
            "图片类型未知，主图、详情页、海报和截图的评价标准不同，建议先补充用途。".to_owned(),
        );
    }
    if risks.is_empty() {
        risks.push(
            "当前诊断只基于传入摘要/OCR，真实转化效果仍需结合点击、加购、成交或 A/B 测试验证。"
                .to_owned(),
        );
    }
    risks
}

fn downstream_skill_suggestions(
    image_type: &str,
    text: &str,
    question: &str,
    has_context: bool,
) -> Vec<String> {
    let mut skills = Vec::new();
    if !has_context {
        skills.push("material_file_understanding".to_owned());
        return skills;
    }
    if image_type == "main_image" || question.contains("主图") {
        skills.push("design_main_image".to_owned());
    }
    if image_type == "detail_page" || question.contains("详情") {
        skills.push("design_detail_page".to_owned());
    }
    if image_type == "poster" || question.contains("海报") || question.contains("活动图") {
        skills.push("design_campaign_poster".to_owned());
    }
    if contains_any(text, &["标题", "文案", "卖点", "点击", "CTR", "ctr"]) {
        skills.push("creative_title_ctr_scorer".to_owned());
    }
    if contains_any(
        text,
        &["最", "第一", "全网最低", "治愈", "治疗", "官方认证", "100%"],
    ) {
        skills.push("search_platform_policy".to_owned());
    }
    if skills.is_empty() {
        skills.push("design_main_image".to_owned());
        skills.push("material_file_understanding".to_owned());
    }
    dedupe(&mut skills);
    skills
}

fn next_actions(
    has_context: bool,
    has_visual_summary: bool,
    has_ocr: bool,
    image_type: &str,
    text: &str,
) -> Vec<String> {
    if !has_context {
        return vec![
            "先触发图片 OCR、视觉模型摘要或让用户补充图片用途，再重新调用图片诊断。".to_owned(),
            "前端应显示图片读取状态，避免用户误以为 Agent 已经看到了原始图片内容。".to_owned(),
        ];
    }

    let mut actions = Vec::new();
    if !has_visual_summary {
        actions.push("补充视觉摘要，至少包含主体、背景、构图、颜色、文案位置和清晰度。".to_owned());
    }
    if !has_ocr {
        actions.push("补充 OCR，便于检查价格、卖点、活动门槛和平台合规词。".to_owned());
    }
    match image_type {
        "main_image" => actions
            .push("把诊断结果交给 design_main_image，生成主图改版方向和首屏信息层级。".to_owned()),
        "detail_page" => actions.push(
            "把诊断结果交给 design_detail_page，按详情页信息链补齐证据和转化模块。".to_owned(),
        ),
        "poster" => actions.push(
            "把诊断结果交给 design_campaign_poster，明确活动主题、利益点、时间和门槛。".to_owned(),
        ),
        "screenshot" => actions.push(
            "如果这是后台或页面截图，先结合 material_file_understanding 提炼字段和异常位置。"
                .to_owned(),
        ),
        _ => {
            actions.push("先确认图片用途，再选择主图、详情页、海报或截图的专项 Skill。".to_owned())
        }
    }
    if contains_any(
        text,
        &["最", "第一", "全网最低", "治愈", "治疗", "官方认证", "100%"],
    ) {
        actions.push(
            "疑似合规风险需要继续调用 search_platform_policy 或人工复核平台规则。".to_owned(),
        );
    }
    actions
}

fn visible_summary_text(
    has_visual_summary: bool,
    has_ocr: bool,
    visual_text: &str,
    ocr_text: Option<&str>,
) -> String {
    if has_visual_summary {
        return compact_text(visual_text, MAX_DISPLAY_CHARS);
    }
    if has_ocr {
        return format!(
            "未收到画面摘要，仅收到 OCR 文字：{}",
            compact_text(ocr_text.unwrap_or_default(), 180)
        );
    }
    "未读取到视觉摘要、OCR 或图片说明。".to_owned()
}

fn infer_image_type(provided: Option<&str>, name: Option<&str>, question: Option<&str>) -> String {
    for value in [provided, name, question].into_iter().flatten() {
        let lower = value.to_ascii_lowercase();
        if lower.contains("main_image") || value.contains("主图") || value.contains("商品图") {
            return "main_image".to_owned();
        }
        if lower.contains("detail") || value.contains("详情页") || value.contains("详情图") {
            return "detail_page".to_owned();
        }
        if lower.contains("poster") || value.contains("海报") || value.contains("活动图") {
            return "poster".to_owned();
        }
        if lower.contains("screenshot") || value.contains("截图") || value.contains("页面截图")
        {
            return "screenshot".to_owned();
        }
        if matches!(
            lower.as_str(),
            "main_image" | "detail_page" | "poster" | "screenshot"
        ) {
            return lower;
        }
    }
    "unknown".to_owned()
}

fn join_parts(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    keywords.iter().any(|keyword| {
        let keyword_lower = keyword.to_ascii_lowercase();
        lower.contains(&keyword_lower) || text.contains(keyword)
    })
}

fn keyword_evidence(text: &str, keywords: &[&str], fallback: &str) -> Vec<String> {
    let mut evidence = meaningful_segments(text, 12)
        .into_iter()
        .filter(|segment| contains_any(segment, keywords))
        .take(3)
        .collect::<Vec<_>>();
    if evidence.is_empty() {
        evidence.push(fallback.to_owned());
    }
    evidence
}

fn display_image_name(name: Option<&str>) -> String {
    name.filter(|value| !value.trim().is_empty())
        .unwrap_or("未命名图片")
        .to_owned()
}

fn dedupe(items: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    items.retain(|item| seen.insert(item.clone()));
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::MaterialImageDiagnosis;
    use crate::skills::{Skill, SkillContext};

    #[tokio::test]
    async fn diagnoses_visual_summary_and_ocr() {
        let outcome = MaterialImageDiagnosis::new()
            .run(
                json!({
                    "image_name": "main-image.png",
                    "image_type": "main_image",
                    "vision_summary": "画面中心是冷萃咖啡瓶，背景偏暗，右上角有低糖0脂卖点，底部价格19.9元。",
                    "ocr_text": "低糖0脂 冷萃咖啡 19.9元",
                    "user_question": "请诊断这张主图的卖点和风险",
                    "context_refs": ["conversation_file:image-1"]
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["data_readiness"], "visual_summary_available");
        assert!(
            outcome.output["后续适配 Skill"]
                .as_array()
                .expect("skills should be array")
                .iter()
                .any(|item| item == "design_main_image")
        );
        assert!(
            outcome.output["诊断维度"]
                .as_array()
                .expect("dimensions should be array")
                .iter()
                .any(|item| item["dimension"] == "卖点呈现")
        );
    }

    #[tokio::test]
    async fn defers_when_only_image_metadata_exists() {
        let outcome = MaterialImageDiagnosis::new()
            .run(
                json!({
                    "image_name": "poster.png",
                    "image_count": 1
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "deferred");
        assert_eq!(outcome.output["data_readiness"], "metadata_only");
        assert_eq!(outcome.output["has_data"], false);
    }

    #[tokio::test]
    async fn handles_ocr_only_without_claiming_visual_reading() {
        let outcome = MaterialImageDiagnosis::new()
            .run(
                json!({
                    "image_name": "detail.jpg",
                    "image_type": "detail_page",
                    "ocr_text": "官方认证 全网最低价 敏感肌修护精华",
                    "user_question": "检查详情页截图文案风险"
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["data_readiness"], "ocr_only");
        assert!(
            outcome.output["风险与限制"]
                .as_array()
                .expect("risks should be array")
                .iter()
                .any(|item| item.as_str().is_some_and(|text| text.contains("只有 OCR")))
        );
        assert!(
            outcome.output["后续适配 Skill"]
                .as_array()
                .expect("skills should be array")
                .iter()
                .any(|item| item == "search_platform_policy")
        );
    }

    #[tokio::test]
    async fn reads_visual_summary_from_context_metadata() {
        let context = SkillContext::new().with_metadata(
            "request.attachments.image_summaries_json",
            json!([
                {
                    "summary": "主图里商品主体清晰，左侧有新品卖点，右下角有券后价39元。"
                }
            ]),
        );
        let outcome = MaterialImageDiagnosis::new()
            .run(
                json!({
                    "image_name": "main.png",
                    "user_question": "分析这张主图"
                }),
                context,
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["data_readiness"], "visual_summary_available");
    }
}
