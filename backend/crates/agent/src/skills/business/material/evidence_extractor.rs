use std::collections::BTreeSet;

use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::{
        compact_text, metadata_refs, metadata_text, normalized_optional_text, question_terms,
    },
    specs::material_evidence_extractor_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_i64_param,
    optional_string_param, optional_string_vec_param,
};

const DEFAULT_MAX_EVIDENCE: usize = 8;
const MAX_EVIDENCE: usize = 30;
const MAX_ANALYSIS_CHARS: usize = 18_000;
const MAX_SEGMENT_CHARS: usize = 240;

#[derive(Clone, Debug)]
struct SourceDocument {
    ref_id: Option<String>,
    title: Option<String>,
    text: String,
}

#[derive(Clone, Debug)]
struct EvidenceCandidate {
    source_ref: Option<String>,
    source_title: Option<String>,
    quote: String,
    evidence_type: &'static str,
    relevance_score: i64,
    contains_number: bool,
    matched_terms: Vec<String>,
}

pub struct MaterialEvidenceExtractor {
    spec: SkillSpec,
}

impl MaterialEvidenceExtractor {
    pub fn new() -> Self {
        Self {
            spec: material_evidence_extractor_spec(),
        }
    }
}

impl Default for MaterialEvidenceExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for MaterialEvidenceExtractor {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let material_name =
            normalized_optional_text(optional_string_param(&params, "material_name")?).or_else(
                || {
                    metadata_text(
                        &context,
                        &["request.context.source_titles", "context.source_titles"],
                    )
                },
            );
        let user_question =
            normalized_optional_text(optional_string_param(&params, "user_question")?)
                .or_else(|| metadata_text(&context, &["task_description"]));
        let max_evidence = optional_i64_param(&params, "max_evidence")?
            .and_then(|value| usize::try_from(value.clamp(1, MAX_EVIDENCE as i64)).ok())
            .unwrap_or(DEFAULT_MAX_EVIDENCE);
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

        let mut sources = source_documents_from_value(params.get("source_summaries"));
        if let Some(text) =
            normalized_optional_text(optional_string_param(&params, "content_text")?).or_else(
                || {
                    metadata_text(
                        &context,
                        &[
                            "request.context.combined_summary",
                            "context.combined_summary",
                            "request.context.selected_summary",
                            "context.selected_summary",
                            "request.workspaceContextPack.summary",
                            "workspaceContextPack.summary",
                            "request.workspaceContext.summary",
                            "workspaceContext.summary",
                            "request.attachments.image_summaries_json",
                            "attachments.image_summaries_json",
                        ],
                    )
                },
            )
        {
            sources.insert(
                0,
                SourceDocument {
                    ref_id: context_refs.first().cloned(),
                    title: material_name.clone(),
                    text,
                },
            );
        }
        sources.retain(|source| !source.text.trim().is_empty());
        if sources.is_empty() {
            let data_readiness = if material_name.is_some() || !context_refs.is_empty() {
                "metadata_only"
            } else {
                "missing_material"
            };
            let summary = match data_readiness {
                "metadata_only" => format!(
                    "只收到资料元数据：{}，尚未收到可提取证据的正文、OCR 或摘要。",
                    display_material_name(material_name.as_deref())
                ),
                _ => "未收到可提取证据的资料内容或资料引用。".to_owned(),
            };
            let output = json!({
                "status": "deferred",
                "has_data": false,
                "adapter_status": "manual_or_context_input",
                "data_readiness": data_readiness,
                "summary": summary,
                "skill": self.spec.id,
                "资料名称": material_name,
                "资料引用": context_refs,
                "证据包": [],
                "证据主题": [],
                "证据覆盖": coverage_json(false, &[], 0),
                "缺失信息": [
                    "缺少可读取正文、OCR、图片摘要、表格摘要或多资料合并文本。",
                    "缺少证据内容时，后续岗位不得把文件名、资料标题或附件数量当作事实依据。"
                ],
                "后续适配 Skill": ["material_file_understanding"],
                "下一步建议": [
                    "先完成文件文本抽取、OCR、视觉摘要或资料上下文打包，再重新提取证据。",
                    "前端应展示资料读取状态和来源选择，避免用户误以为 Agent 已读取完整文件。"
                ],
                "治理说明": [
                    "本 skill 不读取磁盘文件，只处理本轮显式传入的正文、OCR、摘要或上下文元数据。",
                    "没有可读内容时必须 deferred，不能生成证据句或来源引用。"
                ]
            });
            return Ok(SkillOutcome::new(output).with_summary(summary));
        }

        let terms = question_terms(user_question.as_deref().unwrap_or_default());
        let total_chars = sources
            .iter()
            .map(|source| source.text.chars().count())
            .sum::<usize>();
        let candidates = collect_candidates(&sources, &terms);
        let selected = select_candidates(candidates, max_evidence);
        let evidence_json = selected
            .iter()
            .enumerate()
            .map(|(index, item)| evidence_json(index + 1, item))
            .collect::<Vec<_>>();
        let topics = evidence_topics(&selected);
        let missing = missing_info(&selected, &terms, &context_refs);
        let downstream_skills = downstream_skill_suggestions(&selected, &terms);
        let summary = format!(
            "已从 {} 份资料中抽取 {} 条可引用证据，覆盖 {} 类证据主题。",
            sources.len(),
            selected.len(),
            topics.len()
        );
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_or_context_input",
            "data_readiness": if total_chars > MAX_ANALYSIS_CHARS { "evidence_extracted_truncated" } else { "evidence_extracted" },
            "summary": summary.clone(),
            "skill": self.spec.id,
            "资料名称": material_name,
            "资料引用": context_refs,
            "资料数量": sources.len(),
            "分析字符数": total_chars.min(MAX_ANALYSIS_CHARS),
            "问题关键词": terms,
            "证据包": evidence_json,
            "证据主题": topics,
            "证据覆盖": coverage_json(true, &selected, sources.len()),
            "缺失信息": missing,
            "后续适配 Skill": downstream_skills,
            "下一步建议": next_actions(&selected),
            "治理说明": [
                "本 skill 只抽取可引用证据和证据使用边界，不替代最终业务判断、外部搜索或平台规则核验。",
                "后续岗位回答应优先引用“证据包”里的 sourceRef/sourceTitle/quote，并区分资料内事实、外部搜索证据和模型推理建议。",
                "如果 data_readiness=evidence_extracted_truncated，应继续分段提取或按来源逐份提取，避免长文档证据遗漏。"
            ]
        });

        Ok(SkillOutcome::new(output).with_summary(summary))
    }
}

fn source_documents_from_value(value: Option<&Value>) -> Vec<SourceDocument> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(source_document_from_value)
            .collect::<Vec<_>>(),
        _ => source_document_from_value(value).into_iter().collect(),
    }
}

fn source_document_from_value(value: &Value) -> Option<SourceDocument> {
    match value {
        Value::String(text) => {
            normalized_optional_text(Some(text.clone())).map(|text| SourceDocument {
                ref_id: None,
                title: None,
                text,
            })
        }
        Value::Object(object) => {
            let text = ["content", "text", "summary", "combined_summary", "ocr_text"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)))?;
            let ref_id = ["ref", "sourceRef", "source_ref", "id", "fileId", "file_id"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)));
            let title = ["title", "name", "fileName", "file_name"]
                .iter()
                .find_map(|key| object.get(*key).and_then(text_from_value))
                .and_then(|value| normalized_optional_text(Some(value)));
            Some(SourceDocument {
                ref_id,
                title,
                text,
            })
        }
        Value::Array(items) => {
            let text = items
                .iter()
                .filter_map(text_from_value)
                .collect::<Vec<_>>()
                .join("\n");
            normalized_optional_text(Some(text)).map(|text| SourceDocument {
                ref_id: None,
                title: None,
                text,
            })
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => None,
    }
}

fn collect_candidates(sources: &[SourceDocument], terms: &[String]) -> Vec<EvidenceCandidate> {
    let mut candidates = Vec::new();
    for source in sources {
        let text = compact_text(&source.text, MAX_ANALYSIS_CHARS);
        for segment in split_segments(&text) {
            let matched_terms = terms
                .iter()
                .filter(|term| segment.contains(term.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            let contains_number = segment.chars().any(|char| char.is_ascii_digit());
            let evidence_type = evidence_type(&segment);
            let score = relevance_score(&segment, &matched_terms, contains_number, evidence_type);
            if score <= 0 {
                continue;
            }
            candidates.push(EvidenceCandidate {
                source_ref: source.ref_id.clone(),
                source_title: source.title.clone(),
                quote: compact_text(&segment, MAX_SEGMENT_CHARS),
                evidence_type,
                relevance_score: score,
                contains_number,
                matched_terms,
            });
        }
    }
    candidates
}

fn split_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
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
            push_unique(&mut segments, compact_text(raw, MAX_SEGMENT_CHARS));
        }
    }
    segments
}

fn relevance_score(
    segment: &str,
    matched_terms: &[String],
    contains_number: bool,
    evidence_type: &str,
) -> i64 {
    let mut score = 0;
    score += (matched_terms.len() as i64) * 4;
    if contains_number {
        score += 3;
    }
    if matches!(
        evidence_type,
        "metric" | "risk" | "rule" | "customer_feedback" | "price" | "product_fact"
    ) {
        score += 2;
    }
    if segment.contains("因为") || segment.contains("导致") || segment.contains("原因") {
        score += 1;
    }
    if score == 0 && segment.chars().count() >= 12 {
        score = 1;
    }
    score
}

fn select_candidates(
    mut candidates: Vec<EvidenceCandidate>,
    max_evidence: usize,
) -> Vec<EvidenceCandidate> {
    candidates.sort_by(|left, right| {
        right
            .relevance_score
            .cmp(&left.relevance_score)
            .then_with(|| left.quote.cmp(&right.quote))
    });
    let mut seen = BTreeSet::new();
    let mut selected = Vec::new();
    for candidate in candidates {
        let key = normalize_dedupe_key(&candidate.quote);
        if !seen.insert(key) {
            continue;
        }
        selected.push(candidate);
        if selected.len() >= max_evidence {
            break;
        }
    }
    selected
}

fn evidence_json(index: usize, candidate: &EvidenceCandidate) -> Value {
    json!({
        "id": format!("evidence-{index}"),
        "sourceRef": candidate.source_ref,
        "sourceTitle": candidate.source_title,
        "quote": candidate.quote,
        "evidenceType": candidate.evidence_type,
        "relevanceScore": candidate.relevance_score,
        "matchedTerms": candidate.matched_terms,
        "containsNumber": candidate.contains_number,
        "usage": usage_note(candidate),
        "limits": limits_note(candidate)
    })
}

fn evidence_type(segment: &str) -> &'static str {
    if contains_any(
        segment,
        &["规则", "合规", "禁止", "不得", "平台", "审核", "违禁"],
    ) {
        return "rule";
    }
    if contains_any(
        segment,
        &[
            "风险", "异常", "下滑", "投诉", "差评", "退款", "缺失", "问题",
        ],
    ) {
        return "risk";
    }
    if contains_any(
        segment,
        &["评价", "反馈", "客户", "客服", "售后", "买家", "用户"],
    ) {
        return "customer_feedback";
    }
    if contains_any(
        segment,
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
            "库存",
            "毛利",
            "利润",
            "成本",
            "退款率",
        ],
    ) {
        return "metric";
    }
    if contains_any(
        segment,
        &[
            "价格",
            "售价",
            "客单价",
            "券",
            "满减",
            "折",
            "元",
            "￥",
            "¥",
        ],
    ) {
        return "price";
    }
    if contains_any(
        segment,
        &[
            "卖点", "规格", "成分", "材质", "功能", "功效", "适合", "人群",
        ],
    ) {
        return "product_fact";
    }
    "claim"
}

fn evidence_topics(candidates: &[EvidenceCandidate]) -> Vec<String> {
    let mut topics = Vec::new();
    for candidate in candidates {
        let label = match candidate.evidence_type {
            "metric" => "经营/指标证据",
            "risk" => "风险/异常证据",
            "rule" => "规则/合规证据",
            "customer_feedback" => "用户反馈证据",
            "price" => "价格/促销证据",
            "product_fact" => "商品事实证据",
            _ => "一般事实证据",
        };
        push_unique(&mut topics, label.to_owned());
    }
    topics
}

fn coverage_json(has_data: bool, candidates: &[EvidenceCandidate], source_count: usize) -> Value {
    if !has_data {
        return json!({
            "sourceCount": 0,
            "evidenceCount": 0,
            "hasMetricEvidence": false,
            "hasRiskEvidence": false,
            "hasRuleEvidence": false,
            "hasCustomerEvidence": false
        });
    }
    json!({
        "sourceCount": source_count,
        "evidenceCount": candidates.len(),
        "hasMetricEvidence": candidates.iter().any(|item| item.evidence_type == "metric"),
        "hasRiskEvidence": candidates.iter().any(|item| item.evidence_type == "risk"),
        "hasRuleEvidence": candidates.iter().any(|item| item.evidence_type == "rule"),
        "hasCustomerEvidence": candidates.iter().any(|item| item.evidence_type == "customer_feedback")
    })
}

fn missing_info(
    candidates: &[EvidenceCandidate],
    terms: &[String],
    context_refs: &[String],
) -> Vec<String> {
    let mut missing = Vec::new();
    if candidates.is_empty() {
        missing.push("可读内容存在，但未抽到足够清晰的证据片段。".to_owned());
    }
    if terms.is_empty() {
        missing.push(
            "缺少用户问题关键词，证据只能按数字、风险、规则、价格等通用信号排序。".to_owned(),
        );
    }
    if context_refs.is_empty()
        && candidates
            .iter()
            .all(|candidate| candidate.source_ref.is_none() && candidate.source_title.is_none())
    {
        missing.push(
            "缺少明确来源引用，后续回答只能引用片段，无法稳定回链到文件或资料库条目。".to_owned(),
        );
    }
    if !candidates.iter().any(|candidate| candidate.contains_number) {
        missing.push(
            "未识别到数字型证据；涉及利润、投放、库存、转化时还需要表格或后台数据。".to_owned(),
        );
    }
    missing
}

fn downstream_skill_suggestions(candidates: &[EvidenceCandidate], terms: &[String]) -> Vec<String> {
    let mut skills = Vec::new();
    if candidates.iter().any(|item| item.evidence_type == "metric")
        || terms
            .iter()
            .any(|term| contains_any(term, &["利润", "成本", "毛利", "ROI", "转化", "库存"]))
    {
        skills.push("material_table_profiler".to_owned());
        skills.push("data_funnel_analysis".to_owned());
        skills.push("accounting_profit_calc".to_owned());
    }
    if candidates.iter().any(|item| item.evidence_type == "rule") {
        skills.push("search_platform_policy".to_owned());
    }
    if candidates
        .iter()
        .any(|item| item.evidence_type == "customer_feedback")
    {
        skills.push("service_sentiment_analyzer".to_owned());
        skills.push("service_nps_driver_analysis".to_owned());
    }
    if candidates
        .iter()
        .any(|item| matches!(item.evidence_type, "price" | "product_fact"))
    {
        skills.push("creative_title_ctr_scorer".to_owned());
        skills.push("design_main_image".to_owned());
    }
    if skills.is_empty() {
        skills.push("material_file_understanding".to_owned());
    }
    dedupe(&mut skills);
    skills
}

fn next_actions(candidates: &[EvidenceCandidate]) -> Vec<String> {
    let mut actions = vec![
        "把“证据包”交给最终综合或被选中的岗位 Agent，要求回答中引用 sourceRef/sourceTitle/quote。"
            .to_owned(),
        "最终回答应分清资料内事实、外部搜索证据和模型推理建议。".to_owned(),
    ];
    if candidates.iter().any(|item| item.evidence_type == "metric") {
        actions.push(
            "涉及指标或金额时，继续调用表格/数据/财务 Skill 校验字段含义和计算假设。".to_owned(),
        );
    }
    if candidates.iter().any(|item| item.evidence_type == "rule") {
        actions.push(
            "涉及规则或合规时，继续调用 search_platform_policy 获取最新公开来源。".to_owned(),
        );
    }
    actions
}

fn usage_note(candidate: &EvidenceCandidate) -> &'static str {
    match candidate.evidence_type {
        "metric" => "可作为指标、测算或经营判断的资料内依据；使用前需确认怎么算和时间范围。",
        "risk" => "可作为问题诊断或风险提示依据；不要直接外推到全部业务。",
        "rule" => "可作为资料内规则线索；如需最新平台规则必须再走搜索或人工复核。",
        "customer_feedback" => "可作为用户声音或客服问题线索；需要样本量和渠道信息辅助判断。",
        "price" => "可作为价格/活动表达依据；需要确认是否含券、满减门槛和有效期。",
        "product_fact" => "可作为商品事实或卖点依据；需要与商品资料、详情页或平台后台核对。",
        _ => "可作为一般事实片段引用；不要超出原文含义。",
    }
}

fn limits_note(candidate: &EvidenceCandidate) -> &'static str {
    if candidate.source_ref.is_none() && candidate.source_title.is_none() {
        return "缺少稳定来源引用，回答中应说明来自本轮上下文片段。";
    }
    if candidate.quote.chars().count() >= MAX_SEGMENT_CHARS {
        return "片段已截断，长文档应回到原资料确认完整上下文。";
    }
    "仅代表本轮传入资料中的局部证据，不能替代完整文件审阅。"
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

fn normalize_dedupe_key(value: &str) -> String {
    value
        .chars()
        .filter(|char| !char.is_whitespace())
        .take(80)
        .collect::<String>()
}

fn display_material_name(name: Option<&str>) -> String {
    name.filter(|value| !value.trim().is_empty())
        .unwrap_or("未命名资料")
        .to_owned()
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

    use super::MaterialEvidenceExtractor;
    use crate::skills::{Skill, SkillContext};

    #[tokio::test]
    async fn extracts_metric_and_risk_evidence_from_text() {
        let outcome = MaterialEvidenceExtractor::new()
            .run(
                json!({
                    "material_name": "review.md",
                    "content_text": "商品A 成本 12 元，售价 29 元，月销量 300 件。\n客服评价集中在发货慢和包装破损，退款率 8%。\n平台规则提示不得使用全网最低等绝对化表述。",
                    "user_question": "请找出利润、客服和合规风险证据",
                    "context_refs": ["conversation_file:file-1"],
                    "max_evidence": 5
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["data_readiness"], "evidence_extracted");
        let evidence = outcome.output["证据包"]
            .as_array()
            .expect("evidence should be array");
        assert!(evidence.iter().any(|item| item["evidenceType"] == "metric"));
        assert!(evidence.iter().any(|item| item["evidenceType"] == "risk"));
        assert!(evidence.iter().any(|item| item["evidenceType"] == "rule"));
    }

    #[tokio::test]
    async fn extracts_evidence_from_source_summaries() {
        let outcome = MaterialEvidenceExtractor::new()
            .run(
                json!({
                    "source_summaries": [
                        {
                            "ref": "conversation_file:file-1",
                            "title": "profit.xlsx",
                            "summary": "GMV 12000 元，成本 7000 元，广告费 1200 元，订单 180。"
                        },
                        {
                            "ref": "conversation_file:file-2",
                            "title": "reviews.docx",
                            "summary": "用户反馈发货慢，客服响应时间偏长。"
                        }
                    ],
                    "user_question": "提取经营和客服证据"
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["资料数量"], 2);
        assert!(
            outcome.output["证据包"]
                .as_array()
                .expect("evidence should be array")
                .iter()
                .any(|item| item["sourceRef"] == "conversation_file:file-1")
        );
    }

    #[tokio::test]
    async fn defers_without_readable_content() {
        let outcome = MaterialEvidenceExtractor::new()
            .run(
                json!({
                    "material_name": "brief.pdf",
                    "context_refs": ["conversation_file:file-1"]
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
    async fn reads_context_combined_summary() {
        let context = SkillContext::new().with_metadata(
            "request.context.combined_summary",
            json!("活动复盘显示点击率 3.2%，加购率 8%，但详情页停留时间下滑。"),
        );
        let outcome = MaterialEvidenceExtractor::new()
            .run(json!({"user_question": "提取转化证据"}), context)
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert!(
            outcome.output["证据包"]
                .as_array()
                .expect("evidence should be array")
                .iter()
                .any(|item| item["containsNumber"] == true)
        );
    }
}
