use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::{
        compact_text, evidence_segments, infer_material_type, material_scenarios,
        meaningful_segments, metadata_refs, metadata_text, metadata_usize, missing_info,
        normalized_optional_text, risks,
    },
    specs::material_file_understanding_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_i64_param,
    optional_string_param, optional_string_vec_param,
};

const MAX_ANALYSIS_CHARS: usize = 8_000;

pub struct MaterialFileUnderstanding {
    spec: SkillSpec,
}

impl MaterialFileUnderstanding {
    pub fn new() -> Self {
        Self {
            spec: material_file_understanding_spec(),
        }
    }
}

impl Default for MaterialFileUnderstanding {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for MaterialFileUnderstanding {
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
        let provided_material_type =
            normalized_optional_text(optional_string_param(&params, "material_type")?);
        let content_text =
            normalized_optional_text(optional_string_param(&params, "content_text")?).or_else(
                || {
                    metadata_text(
                        &context,
                        &[
                            "request.context.combined_summary",
                            "context.combined_summary",
                            "request.workspaceContextPack.summary",
                            "workspaceContextPack.summary",
                            "request.workspaceContext.summary",
                            "workspaceContext.summary",
                        ],
                    )
                },
            );
        let ocr_text = normalized_optional_text(optional_string_param(&params, "ocr_text")?)
            .or_else(|| {
                metadata_text(
                    &context,
                    &[
                        "request.attachments.image_summaries_json",
                        "attachments.image_summaries_json",
                    ],
                )
            });
        let summary =
            normalized_optional_text(optional_string_param(&params, "summary")?).or_else(|| {
                metadata_text(
                    &context,
                    &[
                        "request.context.selected_summary",
                        "context.selected_summary",
                        "request.context.summary",
                        "context.summary",
                    ],
                )
            });
        let user_question =
            normalized_optional_text(optional_string_param(&params, "user_question")?)
                .or_else(|| metadata_text(&context, &["task_description"]));
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
        let file_count = optional_i64_param(&params, "file_count")?
            .and_then(|value| usize::try_from(value.max(0)).ok())
            .or_else(|| {
                metadata_usize(
                    &context,
                    &[
                        "request.context.source_count",
                        "context.source_count",
                        "request.source_count",
                    ],
                )
            })
            .unwrap_or(context_refs.len());

        let material_type =
            infer_material_type(material_name.as_deref(), provided_material_type.as_deref());
        let available_parts = [
            summary.as_deref().unwrap_or_default(),
            content_text.as_deref().unwrap_or_default(),
            ocr_text.as_deref().unwrap_or_default(),
        ]
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
        let combined = available_parts.join("\n\n");
        let has_content = !combined.trim().is_empty();
        let truncated = combined.chars().count() > MAX_ANALYSIS_CHARS;
        let analysis_text = compact_text(&combined, MAX_ANALYSIS_CHARS);
        let question = user_question.as_deref().unwrap_or_default();
        let key_facts = if has_content {
            meaningful_segments(&analysis_text, 6)
        } else {
            Vec::new()
        };
        let evidence = if has_content {
            evidence_segments(&analysis_text, question, 5)
        } else {
            Vec::new()
        };
        let data_readiness = if has_content {
            if truncated {
                "readable_truncated"
            } else {
                "readable"
            }
        } else if material_name.is_some() || file_count > 0 || !context_refs.is_empty() {
            "metadata_only"
        } else {
            "missing_material"
        };
        let status = if has_content { "success" } else { "deferred" };
        let summary_line = match data_readiness {
            "readable" | "readable_truncated" => format!(
                "已形成资料理解：{}，抽取 {} 条关键事实、{} 条可引用证据。",
                display_material_name(material_name.as_deref()),
                key_facts.len(),
                evidence.len()
            ),
            "metadata_only" => format!(
                "只收到资料元数据：{}，尚未收到可读取正文或 OCR。",
                display_material_name(material_name.as_deref())
            ),
            _ => "未收到可理解的资料内容或资料引用。".to_owned(),
        };
        let relevance = relevance_note(has_content, question);
        let material_summary = if has_content {
            compact_text(&analysis_text, 420)
        } else {
            "未读取到正文、OCR 或摘要。".to_owned()
        };
        let usable_for_answer = material_scenarios(&analysis_text, &material_type);
        let missing = missing_info(
            has_content,
            !question.trim().is_empty(),
            &context_refs,
            &material_type,
        );
        let risk_items = risks(has_content, &material_type, truncated);
        let next_action_items = next_actions(has_content, &material_type);

        let output = json!({
            "status": status,
            "has_data": has_content,
            "adapter_status": "manual_or_context_input",
            "data_readiness": data_readiness,
            "summary": summary_line.clone(),
            "skill": self.spec.id,
            "资料名称": material_name,
            "资料类型": material_type,
            "资料数量": file_count,
            "资料引用": context_refs,
            "问题相关性": relevance,
            "资料摘要": material_summary,
            "关键事实": key_facts,
            "可引用证据": evidence,
            "可用于回答": usable_for_answer,
            "缺失信息": missing,
            "风险与限制": risk_items,
            "下一步建议": next_action_items,
            "治理说明": [
                "本 skill 只理解本轮显式传入的资料正文、OCR、摘要或上下文元数据，不会自行读取磁盘文件或伪造文件内容。",
                "如果需要最新外部资料，应另行触发 search_* skill；如果需要表格字段剖析、图片视觉诊断或跨文件证据链，应继续调用专项 material/data/design skill。",
                "当 status=deferred 且 data_readiness=metadata_only 时，调用方应先完成文件抽取、OCR 或上下文打包，再让业务岗位继续回答。"
            ]
        });

        Ok(SkillOutcome::new(output).with_summary(summary_line))
    }
}

fn relevance_note(has_content: bool, question: &str) -> String {
    if !has_content {
        return "未读取到资料正文，无法判断与问题的真实相关性。".to_owned();
    }
    if question.trim().is_empty() {
        return "未提供具体问题，当前只做通用资料理解。".to_owned();
    }
    "已按用户问题优先抽取含关键词或数字的片段，后续岗位回答应引用“可引用证据”。".to_owned()
}

fn next_actions(has_content: bool, material_type: &str) -> Vec<String> {
    if !has_content {
        return vec![
            "先触发文件文本抽取、图片 OCR/视觉预处理，或把资料正文传入 content_text/ocr_text。"
                .to_owned(),
            "让前端在资料面板显示读取状态，避免用户误以为 Agent 已读完整文件。".to_owned(),
        ];
    }

    let mut actions = vec![
        "把“关键事实”和“可引用证据”交给被选中的业务岗位 Agent，要求回答中引用资料依据。".to_owned(),
        "如果用户要结论或方案，让岗位 Skill 基于这些资料继续做运营、财务、客服、设计或教学分析。"
            .to_owned(),
    ];
    if matches!(material_type, "excel" | "csv" | "xlsx" | "xls") {
        actions.push(
            "继续调用表格剖析或 data_* Skill，识别字段、指标口径、异常值和可计算指标。".to_owned(),
        );
    }
    if matches!(material_type, "image" | "png" | "jpg" | "jpeg") {
        actions.push(
            "继续调用图片诊断或视觉模型，补充主体、构图、可读性、卖点呈现和合规风险。".to_owned(),
        );
    }
    actions
}

fn display_material_name(name: Option<&str>) -> String {
    name.filter(|value| !value.trim().is_empty())
        .unwrap_or("未命名资料")
        .to_owned()
}

fn dedupe(items: &mut Vec<String>) {
    let mut seen = std::collections::BTreeSet::new();
    items.retain(|item| seen.insert(item.clone()));
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::MaterialFileUnderstanding;
    use crate::skills::{Skill, SkillContext};

    #[tokio::test]
    async fn returns_readable_evidence_when_content_is_present() {
        let outcome = MaterialFileUnderstanding::new()
            .run(
                json!({
                    "material_name": "profit.xlsx",
                    "content_text": "商品A 成本 12 元，售价 29 元，月销量 300 件。\n差评集中在物流慢。",
                    "user_question": "请分析成本利润和用户反馈",
                    "context_refs": ["conversation_file:file-1"]
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["资料类型"], "excel");
        assert!(
            outcome.output["可引用证据"]
                .as_array()
                .expect("evidence should be array")
                .iter()
                .any(|item| item.as_str().is_some_and(|text| text.contains("成本")))
        );
    }

    #[tokio::test]
    async fn does_not_claim_file_content_when_only_metadata_exists() {
        let outcome = MaterialFileUnderstanding::new()
            .run(
                json!({
                    "material_name": "brief.pdf",
                    "file_count": 1,
                    "user_question": "总结这个 PDF"
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "deferred");
        assert_eq!(outcome.output["data_readiness"], "metadata_only");
        assert_eq!(outcome.output["has_data"], false);
        assert!(
            outcome.output["资料摘要"]
                .as_str()
                .unwrap()
                .contains("未读取到")
        );
    }

    #[tokio::test]
    async fn can_use_context_summary_from_skill_context() {
        let context = SkillContext::new().with_metadata(
            "request.context.combined_summary",
            json!("图片摘要：主图上有价格 19.9 元，卖点是低糖和便携。"),
        );
        let outcome = MaterialFileUnderstanding::new()
            .run(
                json!({"material_type": "image", "user_question": "看这张图有什么卖点"}),
                context,
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["资料类型"], "image");
        assert!(
            outcome.output["关键事实"]
                .as_array()
                .expect("facts should be array")
                .iter()
                .any(|item| item.as_str().is_some_and(|text| text.contains("低糖")))
        );
    }
}
