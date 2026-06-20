use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::{
        compact_text, metadata_refs, metadata_text, normalized_optional_text, question_terms,
    },
    specs::material_multi_file_compare_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_i64_param,
    optional_string_param, optional_string_vec_param,
};

const DEFAULT_MAX_POINTS: usize = 6;
const MAX_POINTS: usize = 20;
const MAX_TEXT_CHARS: usize = 8_000;
const MAX_SEGMENT_CHARS: usize = 220;

#[derive(Clone, Debug)]
struct MaterialDoc {
    ref_id: Option<String>,
    title: String,
    material_type: String,
    version: Option<String>,
    updated_at: Option<String>,
    text: String,
}

#[derive(Clone, Debug)]
struct MaterialSignal {
    label: &'static str,
    value: String,
    ref_id: Option<String>,
    title: String,
}

pub struct MaterialMultiFileCompare {
    spec: SkillSpec,
}

impl MaterialMultiFileCompare {
    pub fn new() -> Self {
        Self {
            spec: material_multi_file_compare_spec(),
        }
    }
}

impl Default for MaterialMultiFileCompare {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for MaterialMultiFileCompare {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let user_question =
            normalized_optional_text(optional_string_param(&params, "user_question")?)
                .or_else(|| metadata_text(&context, &["task_description"]));
        let compare_focus =
            normalized_optional_text(optional_string_param(&params, "compare_focus")?)
                .unwrap_or_else(|| infer_compare_focus(user_question.as_deref()));
        let max_points = optional_i64_param(&params, "max_points")?
            .and_then(|value| usize::try_from(value.clamp(1, MAX_POINTS as i64)).ok())
            .unwrap_or(DEFAULT_MAX_POINTS);
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
            ],
        ));
        dedupe(&mut context_refs);

        let mut materials = material_docs_from_value(params.get("materials"));
        if materials.is_empty()
            && let Some(text) = metadata_text(
                &context,
                &[
                    "request.context.source_summaries",
                    "context.source_summaries",
                    "request.context.combined_summary",
                    "context.combined_summary",
                ],
            )
        {
            materials = fallback_docs_from_combined_text(&text, &context_refs);
        }
        materials.retain(|item| !item.text.trim().is_empty());

        if materials.len() < 2 {
            let data_readiness = if materials.len() == 1 {
                "single_material_only"
            } else if !context_refs.is_empty() {
                "metadata_only"
            } else {
                "missing_materials"
            };
            let summary = match data_readiness {
                "single_material_only" => "只收到 1 份可读资料，无法做多资料对比。".to_owned(),
                "metadata_only" => "只收到资料引用或元数据，尚未收到可对比正文/摘要。".to_owned(),
                _ => "未收到可对比的多份资料内容或资料引用。".to_owned(),
            };
            let output = json!({
                "status": "deferred",
                "has_data": false,
                "adapter_status": "manual_or_context_input",
                "data_readiness": data_readiness,
                "summary": summary,
                "skill": self.spec.id,
                "资料引用": context_refs,
                "资料数量": materials.len(),
                "对比重点": compare_focus,
                "资料清单": material_list_json(&materials),
                "共同点": [],
                "差异点": [],
                "冲突点": [],
                "版本变化": [],
                "可合并结论": [],
                "缺失信息": missing_when_deferred(data_readiness),
                "后续适配 Skill": ["material_file_understanding"],
                "下一步建议": [
                    "至少传入两份包含正文、OCR、表格摘要或图片摘要的资料。",
                    "如果资料来自上传文件，先完成文件抽取或资料上下文打包，再重新做多资料对比。"
                ],
                "治理说明": [
                    "本 skill 不读取磁盘文件，只比较本轮显式传入的 materials 或上下文摘要。",
                    "资料少于两份时不得生成差异、冲突或共同结论。"
                ]
            });
            return Ok(SkillOutcome::new(output).with_summary(summary));
        }

        let terms = question_terms(user_question.as_deref().unwrap_or_default());
        let signals = collect_signals(&materials);
        let common_points = common_points(&signals, max_points);
        let difference_points = difference_points(&signals, &materials, max_points);
        let conflict_points = conflict_points(&signals, max_points);
        let version_changes = version_changes(&materials, &signals, max_points);
        let mergeable_conclusions = mergeable_conclusions(
            &common_points,
            &difference_points,
            &conflict_points,
            max_points,
        );
        let missing = missing_info(&materials, &conflict_points, &terms);
        let downstream_skills =
            downstream_skill_suggestions(&signals, &conflict_points, &compare_focus);
        let summary = format!(
            "已对比 {} 份资料：发现 {} 条共同点、{} 条差异点、{} 条冲突点。",
            materials.len(),
            common_points.len(),
            difference_points.len(),
            conflict_points.len()
        );
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_or_context_input",
            "data_readiness": "multi_material_compared",
            "summary": summary.clone(),
            "skill": self.spec.id,
            "资料引用": context_refs,
            "资料数量": materials.len(),
            "资料清单": material_list_json(&materials),
            "对比重点": compare_focus,
            "问题关键词": terms,
            "共同点": common_points,
            "差异点": difference_points,
            "冲突点": conflict_points,
            "版本变化": version_changes,
            "可合并结论": mergeable_conclusions,
            "缺失信息": missing,
            "后续适配 Skill": downstream_skills,
            "下一步建议": next_actions(&materials),
            "治理说明": [
                "本 skill 只做多资料文本/摘要层面对比，不替代完整文件审阅、真实表格计算或外部搜索。",
                "冲突点只是基于当前传入片段的初筛，最终结论应回到原文件或业务后台核对。",
                "如果对比涉及最新平台规则、行业数据或竞品市场，应继续调用 search_* skill 获取外部来源。"
            ]
        });

        Ok(SkillOutcome::new(output).with_summary(summary))
    }
}

fn material_docs_from_value(value: Option<&Value>) -> Vec<MaterialDoc> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::Array(items) => items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| material_doc_from_value(item, index))
            .collect(),
        _ => material_doc_from_value(value, 0).into_iter().collect(),
    }
}

fn material_doc_from_value(value: &Value, index: usize) -> Option<MaterialDoc> {
    match value {
        Value::String(text) => {
            normalized_optional_text(Some(text.clone())).map(|text| MaterialDoc {
                ref_id: None,
                title: format!("资料{}", index + 1),
                material_type: "text".to_owned(),
                version: None,
                updated_at: None,
                text,
            })
        }
        Value::Object(object) => {
            let text = ["content", "text", "summary", "combined_summary", "ocr_text"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)))?;
            let title = ["title", "name", "fileName", "file_name"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)))
                .unwrap_or_else(|| format!("资料{}", index + 1));
            let ref_id = ["ref", "sourceRef", "source_ref", "id", "fileId", "file_id"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)));
            let material_type = ["type", "material_type", "fileType", "file_type"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)))
                .unwrap_or_else(|| "unknown".to_owned());
            let version = ["version", "revision", "batch"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)));
            let updated_at = ["updated_at", "updatedAt", "date", "time"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)));
            Some(MaterialDoc {
                ref_id,
                title,
                material_type,
                version,
                updated_at,
                text,
            })
        }
        Value::Array(items) => {
            let text = items
                .iter()
                .filter_map(text_from_value)
                .collect::<Vec<_>>()
                .join("\n");
            normalized_optional_text(Some(text)).map(|text| MaterialDoc {
                ref_id: None,
                title: format!("资料{}", index + 1),
                material_type: "text".to_owned(),
                version: None,
                updated_at: None,
                text,
            })
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => None,
    }
}

fn fallback_docs_from_combined_text(text: &str, refs: &[String]) -> Vec<MaterialDoc> {
    let normalized = text
        .replace("\n---\n", "\n@@MATERIAL_SPLIT@@\n")
        .replace("\n### ", "\n@@MATERIAL_SPLIT@@")
        .replace("\n## ", "\n@@MATERIAL_SPLIT@@");
    let parts = normalized
        .split("@@MATERIAL_SPLIT@@")
        .map(str::trim)
        .filter(|value| value.chars().count() >= 8)
        .collect::<Vec<_>>();
    if parts.len() < 2 {
        return Vec::new();
    }
    parts
        .into_iter()
        .enumerate()
        .map(|(index, part)| MaterialDoc {
            ref_id: refs.get(index).cloned(),
            title: format!("上下文资料{}", index + 1),
            material_type: "context".to_owned(),
            version: None,
            updated_at: None,
            text: part.to_owned(),
        })
        .collect()
}

fn collect_signals(materials: &[MaterialDoc]) -> Vec<MaterialSignal> {
    let mut signals = Vec::new();
    for material in materials {
        let text = compact_text(&material.text, MAX_TEXT_CHARS);
        for segment in split_segments(&text) {
            for (label, keywords) in signal_keywords() {
                if contains_any(&segment, keywords) {
                    signals.push(MaterialSignal {
                        label,
                        value: compact_text(&segment, MAX_SEGMENT_CHARS),
                        ref_id: material.ref_id.clone(),
                        title: material.title.clone(),
                    });
                    break;
                }
            }
        }
    }
    signals
}

fn common_points(signals: &[MaterialSignal], max_points: usize) -> Vec<Value> {
    let mut by_label: BTreeMap<&str, Vec<&MaterialSignal>> = BTreeMap::new();
    for signal in signals {
        by_label.entry(signal.label).or_default().push(signal);
    }
    by_label
        .into_iter()
        .filter_map(|(label, items)| {
            let titles = unique_titles(&items);
            if titles.len() < 2 {
                return None;
            }
            Some(json!({
                "topic": label,
                "sources": titles,
                "evidence": items.iter().take(3).map(|item| signal_json(item)).collect::<Vec<_>>(),
                "summary": format!("多份资料都提到 {label} 相关信息。")
            }))
        })
        .take(max_points)
        .collect()
}

fn difference_points(
    signals: &[MaterialSignal],
    materials: &[MaterialDoc],
    max_points: usize,
) -> Vec<Value> {
    let mut by_label: BTreeMap<&str, Vec<&MaterialSignal>> = BTreeMap::new();
    for signal in signals {
        by_label.entry(signal.label).or_default().push(signal);
    }
    let total_sources = materials.len();
    by_label
        .into_iter()
        .filter_map(|(label, items)| {
            let titles = unique_titles(&items);
            if titles.len() == total_sources || titles.is_empty() {
                return None;
            }
            let missing_sources = materials
                .iter()
                .filter(|material| !titles.iter().any(|title| title == &material.title))
                .map(|material| material.title.clone())
                .collect::<Vec<_>>();
            Some(json!({
                "topic": label,
                "presentIn": titles,
                "missingIn": missing_sources,
                "evidence": items.iter().take(3).map(|item| signal_json(item)).collect::<Vec<_>>(),
                "summary": format!("{label} 只出现在部分资料中，需确认是否是算法差异、资料缺失或版本差异。")
            }))
        })
        .take(max_points)
        .collect()
}

fn conflict_points(signals: &[MaterialSignal], max_points: usize) -> Vec<Value> {
    let mut conflicts = Vec::new();
    let mut by_label: BTreeMap<&str, Vec<&MaterialSignal>> = BTreeMap::new();
    for signal in signals {
        by_label.entry(signal.label).or_default().push(signal);
    }
    for (label, items) in by_label {
        if unique_titles(&items).len() < 2 {
            continue;
        }
        let numbers = items
            .iter()
            .flat_map(|item| numbers_in_text(&item.value))
            .collect::<Vec<_>>();
        let numeric_conflict = numeric_conflict(&numbers);
        let polarity_conflict = polarity_conflict(items.iter().map(|item| item.value.as_str()));
        if !numeric_conflict && !polarity_conflict {
            continue;
        }
        conflicts.push(json!({
            "topic": label,
            "conflictType": if numeric_conflict { "numeric_or_metric_gap" } else { "opposite_polarity" },
            "sources": items.iter().take(4).map(|item| signal_json(item)).collect::<Vec<_>>(),
            "risk": "不同资料对同一主题给出的数字或判断不一致，最终回答需要先标注计算假设并回到原资料核对。"
        }));
        if conflicts.len() >= max_points {
            break;
        }
    }
    conflicts
}

fn version_changes(
    materials: &[MaterialDoc],
    signals: &[MaterialSignal],
    max_points: usize,
) -> Vec<Value> {
    let versioned = materials
        .iter()
        .filter(|item| item.version.is_some() || item.updated_at.is_some())
        .collect::<Vec<_>>();
    if versioned.len() < 2 {
        return Vec::new();
    }
    let mut points = Vec::new();
    for material in versioned.into_iter().take(max_points) {
        let material_signals = signals
            .iter()
            .filter(|signal| signal.title == material.title)
            .take(3)
            .map(signal_json)
            .collect::<Vec<_>>();
        points.push(json!({
            "source": material.title,
            "sourceRef": material.ref_id,
            "version": material.version,
            "updatedAt": material.updated_at,
            "notableSignals": material_signals,
            "summary": "该资料包含版本或更新时间信息，可用于判断版本先后，但仍需业务确认哪个版本有效。"
        }));
    }
    points
}

fn mergeable_conclusions(
    common_points: &[Value],
    difference_points: &[Value],
    conflict_points: &[Value],
    max_points: usize,
) -> Vec<String> {
    let mut conclusions = Vec::new();
    if !common_points.is_empty() {
        conclusions
            .push("多份资料的共同点可以作为相对稳定依据，但回答仍应引用具体来源。".to_owned());
    }
    if !difference_points.is_empty() {
        conclusions
            .push("只出现在部分资料中的信息应标注来源范围，不能当成所有资料共同结论。".to_owned());
    }
    if !conflict_points.is_empty() {
        conclusions.push(
            "存在冲突点时，应先核对版本、时间范围、统计方式或文件来源，再给最终结论。".to_owned(),
        );
    }
    if conclusions.is_empty() {
        conclusions.push(
            "当前资料之间没有抽到明显共同/差异/冲突信号，建议补充更清晰的摘要或原文。".to_owned(),
        );
    }
    conclusions.into_iter().take(max_points).collect()
}

fn material_list_json(materials: &[MaterialDoc]) -> Vec<Value> {
    materials
        .iter()
        .map(|material| {
            json!({
                "sourceRef": material.ref_id,
                "title": material.title,
                "materialType": material.material_type,
                "version": material.version,
                "updatedAt": material.updated_at,
                "textLength": material.text.chars().count()
            })
        })
        .collect()
}

fn missing_info(materials: &[MaterialDoc], conflicts: &[Value], terms: &[String]) -> Vec<String> {
    let mut missing = Vec::new();
    if materials.iter().any(|item| item.ref_id.is_none()) {
        missing.push("部分资料缺少 sourceRef，后续回答无法稳定回链到文件或资料库条目。".to_owned());
    }
    if terms.is_empty() {
        missing.push(
            "缺少用户问题关键词，本次按通用电商信号对比，可能不完全贴合用户意图。".to_owned(),
        );
    }
    if !conflicts.is_empty() {
        missing
            .push("存在冲突点，需要补充版本、时间范围、统计方式或原始表格后再定结论。".to_owned());
    }
    if materials.len() < 3 {
        missing.push("当前只有两份资料，只能做两两对比，不能判断更广泛的共性。".to_owned());
    }
    missing
}

fn missing_when_deferred(data_readiness: &str) -> Vec<String> {
    match data_readiness {
        "single_material_only" => {
            vec!["至少需要两份可读资料才能比较共同点、差异点和冲突点。".to_owned()]
        }
        "metadata_only" => vec!["缺少可读正文、OCR、表格摘要、图片摘要或资料合并文本。".to_owned()],
        _ => vec!["缺少多份资料内容或资料引用。".to_owned()],
    }
}

fn downstream_skill_suggestions(
    signals: &[MaterialSignal],
    conflicts: &[Value],
    compare_focus: &str,
) -> Vec<String> {
    let mut skills = Vec::new();
    if signals.iter().any(|signal| {
        matches!(
            signal.label,
            "经营指标" | "价格促销" | "库存履约" | "成本利润"
        )
    }) {
        skills.push("material_table_profiler".to_owned());
        skills.push("data_funnel_analysis".to_owned());
        skills.push("accounting_profit_calc".to_owned());
    }
    if signals
        .iter()
        .any(|signal| matches!(signal.label, "规则合规" | "风险问题"))
        || compare_focus == "risk"
    {
        skills.push("search_platform_policy".to_owned());
    }
    if signals.iter().any(|signal| signal.label == "客服反馈") {
        skills.push("service_sentiment_analyzer".to_owned());
    }
    if signals
        .iter()
        .any(|signal| matches!(signal.label, "商品卖点" | "视觉素材"))
    {
        skills.push("material_image_diagnosis".to_owned());
        skills.push("creative_title_ctr_scorer".to_owned());
    }
    if !conflicts.is_empty() {
        skills.push("material_evidence_extractor".to_owned());
    }
    if skills.is_empty() {
        skills.push("material_file_understanding".to_owned());
    }
    dedupe(&mut skills);
    skills
}

fn next_actions(materials: &[MaterialDoc]) -> Vec<String> {
    let mut actions = vec![
        "把共同点作为相对稳定依据，把差异点和冲突点交给对应岗位继续核对。".to_owned(),
        "最终回答引用多资料对比结果时，应标注具体来源，避免把单一资料结论泛化。".to_owned(),
    ];
    if materials
        .iter()
        .any(|material| material.material_type.contains("excel"))
    {
        actions.push(
            "涉及 Excel/表格资料时，继续调用表格剖析或数据/财务 Skill 校验字段含义。".to_owned(),
        );
    }
    if materials
        .iter()
        .any(|material| material.material_type.contains("image"))
    {
        actions.push("涉及图片资料时，继续调用图片诊断补充视觉、OCR 和合规风险。".to_owned());
    }
    actions
}

fn split_segments(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.replace('\r', "\n").lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        for raw in line
            .split(['。', '；', ';'])
            .map(str::trim)
            .filter(|value| value.chars().count() >= 6)
        {
            push_unique(&mut out, compact_text(raw, MAX_SEGMENT_CHARS));
        }
    }
    out
}

fn signal_keywords() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        (
            "经营指标",
            &[
                "GMV",
                "gmv",
                "ROI",
                "roi",
                "ROAS",
                "roas",
                "转化",
                "点击",
                "访客",
                "订单",
                "退款率",
            ],
        ),
        ("成本利润", &["成本", "毛利", "利润", "净利", "费用"]),
        (
            "价格促销",
            &["价格", "售价", "元", "￥", "¥", "券", "满减", "折", "包邮"],
        ),
        (
            "库存履约",
            &["库存", "缺货", "发货", "履约", "到货", "物流"],
        ),
        (
            "客服反馈",
            &[
                "评价", "反馈", "客服", "售后", "投诉", "差评", "买家", "用户",
            ],
        ),
        (
            "规则合规",
            &["规则", "合规", "禁止", "不得", "审核", "违禁", "平台"],
        ),
        (
            "风险问题",
            &["风险", "异常", "下滑", "问题", "缺失", "破损", "延迟"],
        ),
        (
            "商品卖点",
            &["卖点", "规格", "成分", "材质", "功能", "适合", "人群"],
        ),
        (
            "视觉素材",
            &[
                "主图",
                "详情页",
                "海报",
                "图片",
                "视觉",
                "构图",
                "OCR",
                "文案",
            ],
        ),
    ]
}

fn signal_json(signal: &MaterialSignal) -> Value {
    json!({
        "sourceRef": signal.ref_id,
        "sourceTitle": signal.title,
        "quote": signal.value
    })
}

fn unique_titles(items: &[&MaterialSignal]) -> Vec<String> {
    let mut titles = Vec::new();
    for item in items {
        push_unique(&mut titles, item.title.clone());
    }
    titles
}

fn numbers_in_text(text: &str) -> Vec<f64> {
    text.split(|char: char| {
        !(char.is_ascii_digit() || char == '.' || char == '-' || char == '%' || char == ',')
    })
    .filter_map(|token| {
        token
            .trim_matches('%')
            .replace(',', "")
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
    })
    .collect()
}

fn numeric_conflict(numbers: &[f64]) -> bool {
    if numbers.len() < 2 {
        return false;
    }
    let min = numbers
        .iter()
        .copied()
        .reduce(f64::min)
        .unwrap_or(0.0)
        .abs();
    let max = numbers
        .iter()
        .copied()
        .reduce(f64::max)
        .unwrap_or(0.0)
        .abs();
    max - min >= 5.0 && max / min.max(1.0) >= 1.3
}

fn polarity_conflict<'a>(items: impl Iterator<Item = &'a str>) -> bool {
    let mut positive = false;
    let mut negative = false;
    for item in items {
        if contains_any(item, &["增长", "提升", "改善", "好评", "稳定", "充足"]) {
            positive = true;
        }
        if contains_any(
            item,
            &["下滑", "下降", "差评", "投诉", "不足", "缺货", "风险"],
        ) {
            negative = true;
        }
    }
    positive && negative
}

fn infer_compare_focus(question: Option<&str>) -> String {
    let question = question.unwrap_or_default();
    if contains_any(question, &["冲突", "矛盾", "不一致"]) {
        "conflict".to_owned()
    } else if contains_any(question, &["版本", "前后", "变化", "更新"]) {
        "version".to_owned()
    } else if contains_any(question, &["指标", "数据", "GMV", "转化", "利润"]) {
        "metrics".to_owned()
    } else if contains_any(question, &["商品", "卖点", "详情", "主图"]) {
        "product".to_owned()
    } else if contains_any(question, &["风险", "规则", "合规"]) {
        "risk".to_owned()
    } else {
        "general".to_owned()
    }
}

fn text_from_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Array(items) => {
            let text = items
                .iter()
                .filter_map(text_from_value)
                .collect::<Vec<_>>()
                .join("\n");
            (!text.trim().is_empty()).then_some(text)
        }
        Value::Object(object) => ["text", "content", "summary", "title", "name", "ref", "id"]
            .iter()
            .find_map(|key| object.get(*key).and_then(text_from_value)),
        Value::Null => None,
    }
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    keywords.iter().any(|keyword| {
        let keyword_lower = keyword.to_ascii_lowercase();
        lower.contains(&keyword_lower) || text.contains(keyword)
    })
}

fn push_unique(out: &mut Vec<String>, value: String) {
    let value = value.trim();
    if value.is_empty() || out.iter().any(|item| item == value) {
        return;
    }
    out.push(value.to_owned());
}

fn dedupe(items: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    items.retain(|item| seen.insert(item.clone()));
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::MaterialMultiFileCompare;
    use crate::skills::{Skill, SkillContext};

    #[tokio::test]
    async fn compares_common_differences_and_conflicts() {
        let outcome = MaterialMultiFileCompare::new()
            .run(
                json!({
                    "materials": [
                        {
                            "ref": "conversation_file:file-1",
                            "title": "活动前复盘",
                            "type": "text",
                            "summary": "GMV 12000 元，转化率 3.2%，客服反馈发货慢，库存充足。"
                        },
                        {
                            "ref": "conversation_file:file-2",
                            "title": "活动后复盘",
                            "type": "text",
                            "summary": "GMV 18000 元，转化率 4.5%，客服反馈发货慢，库存不足存在缺货风险。"
                        }
                    ],
                    "user_question": "对比两份复盘资料的共同点、差异和冲突"
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["data_readiness"], "multi_material_compared");
        assert!(
            !outcome.output["共同点"]
                .as_array()
                .expect("common points should be array")
                .is_empty()
        );
        assert!(
            !outcome.output["冲突点"]
                .as_array()
                .expect("conflicts should be array")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn detects_version_metadata() {
        let outcome = MaterialMultiFileCompare::new()
            .run(
                json!({
                    "materials": [
                        {
                            "ref": "workspace_source:v1",
                            "title": "详情页脚本",
                            "version": "v1",
                            "updated_at": "2026-05-01",
                            "summary": "主打低糖卖点，价格 39 元。"
                        },
                        {
                            "ref": "workspace_source:v2",
                            "title": "详情页脚本",
                            "version": "v2",
                            "updated_at": "2026-05-20",
                            "summary": "主打低糖和高蛋白卖点，价格 45 元。"
                        }
                    ],
                    "user_question": "比较版本变化"
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert!(
            !outcome.output["版本变化"]
                .as_array()
                .expect("version changes should be array")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn defers_with_single_material_only() {
        let outcome = MaterialMultiFileCompare::new()
            .run(
                json!({
                    "materials": [
                        {
                            "title": "only",
                            "summary": "只有一份资料，不能对比。"
                        }
                    ]
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "deferred");
        assert_eq!(outcome.output["data_readiness"], "single_material_only");
    }

    #[tokio::test]
    async fn defers_without_materials() {
        let outcome = MaterialMultiFileCompare::new()
            .run(
                json!({
                    "context_refs": ["conversation_file:file-1", "conversation_file:file-2"]
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "deferred");
        assert_eq!(outcome.output["data_readiness"], "metadata_only");
    }
}
