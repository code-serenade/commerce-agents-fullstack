use utils::llm_json;

use super::{
    types::ParsedWorkerRoleOutput,
    util::{compact_event_text, push_unique},
};
use crate::{
    core::protocol::RoleWorkOutput,
    skills::{SkillExecutionResult, SkillExecutionStatus},
};

pub(super) fn parse_worker_role_output(output: &str) -> ParsedWorkerRoleOutput {
    let trimmed = output.trim();
    let Ok(value) = llm_json::parse_llm_json_value(trimmed) else {
        return ParsedWorkerRoleOutput {
            content: trimmed.to_string(),
            ..ParsedWorkerRoleOutput::default()
        };
    };

    let role_output = parse_role_work_output(&value);
    let evidence = role_output
        .as_ref()
        .map(|output| output.evidence.clone())
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| string_array_field(&value, "evidence"));
    let risks = role_output
        .as_ref()
        .map(|output| output.risks.clone())
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| string_array_field(&value, "risks"));
    let open_questions = role_output
        .as_ref()
        .map(|output| output.open_questions.clone())
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| string_array_field(&value, "open_questions"));
    let content = role_output
        .as_ref()
        .map(format_role_work_output_content)
        .filter(|content| !content.trim().is_empty())
        .unwrap_or_else(|| extract_worker_content(trimmed));

    ParsedWorkerRoleOutput {
        content,
        role_output,
        evidence,
        risks,
        open_questions,
    }
}

pub(super) fn merge_skill_results(
    mut parsed: ParsedWorkerRoleOutput,
    results: &[SkillExecutionResult],
) -> ParsedWorkerRoleOutput {
    if results.is_empty() {
        return parsed;
    }

    let role_output = parsed
        .role_output
        .get_or_insert_with(RoleWorkOutput::default);

    for result in results {
        match result.status {
            SkillExecutionStatus::Success => {
                let evidence = format_success_skill_evidence(result);
                push_unique(&mut parsed.evidence, evidence.clone());
                push_unique(&mut role_output.evidence, evidence);
                if let Some(search_evidence) = format_live_search_evidence(result) {
                    remove_success_skill_evidence(
                        &mut parsed.evidence,
                        result.skill_id.as_str(),
                        result.summary.as_deref(),
                    );
                    remove_success_skill_evidence(
                        &mut role_output.evidence,
                        result.skill_id.as_str(),
                        result.summary.as_deref(),
                    );
                    push_unique(&mut parsed.evidence, search_evidence.clone());
                    push_unique(&mut role_output.evidence, search_evidence);
                    normalize_content_after_live_search(role_output);
                }
                if let Some(evidence_packet) = format_search_evidence_packet(result) {
                    push_unique(&mut parsed.evidence, evidence_packet.clone());
                    push_unique(&mut role_output.evidence, evidence_packet);
                }
            }
            SkillExecutionStatus::Deferred | SkillExecutionStatus::PendingApproval => {
                let risk = format_incomplete_skill_risk(result);
                let question = format_incomplete_skill_question(result);
                push_unique(&mut parsed.risks, risk.clone());
                push_unique(&mut parsed.open_questions, question.clone());
                push_unique(&mut role_output.risks, risk);
                push_unique(&mut role_output.open_questions, question);
            }
            SkillExecutionStatus::NotFound
            | SkillExecutionStatus::ValidationError
            | SkillExecutionStatus::Failed => {
                let risk = format_failed_skill_risk(result);
                push_unique(&mut parsed.risks, risk.clone());
                push_unique(&mut parsed.open_questions, risk.clone());
                push_unique(&mut role_output.risks, risk.clone());
                push_unique(&mut role_output.open_questions, risk);
            }
        }
    }

    parsed.content = parsed
        .role_output
        .as_ref()
        .map(format_role_work_output_content)
        .filter(|content| !content.trim().is_empty())
        .unwrap_or(parsed.content);
    parsed
}

fn remove_success_skill_evidence(items: &mut Vec<String>, skill_id: &str, summary: Option<&str>) {
    let Some(summary) = summary.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    let expected = format!(
        "Skill {skill_id} 完成：{}",
        compact_event_text(summary.to_string())
    );
    items.retain(|item| item != &expected);
}

pub(super) fn format_role_work_output_content(output: &RoleWorkOutput) -> String {
    let mut lines = Vec::new();
    if !output.summary.trim().is_empty() {
        lines.push(output.summary.trim().to_string());
    }
    extend_section(&mut lines, "发现", &output.findings);
    extend_section(&mut lines, "建议", &output.recommendations);
    extend_section(&mut lines, "依据", &output.evidence);
    extend_section(&mut lines, "风险", &output.risks);
    extend_section(&mut lines, "待确认", &output.open_questions);
    lines.join("\n")
}

fn format_search_evidence_packet(result: &SkillExecutionResult) -> Option<String> {
    if !result.skill_id.as_str().starts_with("search_") {
        return None;
    }
    let packet = result.output.as_ref()?.get("evidence_packet")?;
    let serialized = serde_json::to_string(packet).ok()?;
    Some(format!(
        "search_evidence_packet:{}",
        compact_inline(&serialized, 2400)
    ))
}

fn format_live_search_evidence(result: &SkillExecutionResult) -> Option<String> {
    if !result.skill_id.as_str().starts_with("search_") {
        return None;
    }
    let output = result.output.as_ref()?;
    let adapter_status = output
        .get("adapter_status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if adapter_status != "live" {
        return None;
    }

    let sources = output.get("sources")?.as_array()?;
    if sources.is_empty() {
        return None;
    }

    let mut lines = vec![format!(
        "实时搜索来源（{}）：{} 个可采信来源",
        result.skill_id.as_str(),
        sources.len()
    )];
    for (index, source) in sources.iter().take(5).enumerate() {
        let title = source_text(source, "title").unwrap_or("未命名来源");
        let url = source_text(source, "url").unwrap_or("");
        let published_at = source_text(source, "published_at").unwrap_or("发布日期未标注");
        let risk_hint = source_text(source, "snippet").unwrap_or("");

        let mut line = format!(
            "{}. {} | {} | {}",
            index + 1,
            compact_inline(title, 120),
            compact_inline(published_at, 48),
            compact_inline(url, 220)
        );
        if !risk_hint.is_empty() {
            line.push_str(&format!(" | {}", compact_inline(risk_hint, 180)));
        }
        lines.push(line);
    }

    Some(lines.join("\n"))
}

fn normalize_content_after_live_search(output: &mut RoleWorkOutput) {
    const NO_SOURCE_MARKERS: [&str; 6] = [
        "本轮未拿到",
        "未拿到实时来源",
        "没有可采信实时来源",
        "缺少抖音电商官方发布",
        "无法给出具体的合规要求细则和对应来源信息",
        "不能直接给出对应规则明细",
    ];

    if NO_SOURCE_MARKERS
        .iter()
        .any(|marker| output.summary.contains(marker))
    {
        output.summary = "本轮已通过外部搜索取得实时来源；以下依据列表为后续判断的出处，\
                          若来源与通用经验冲突，以实时来源为准。"
            .to_string();
    }

    output
        .findings
        .retain(|item| !NO_SOURCE_MARKERS.iter().any(|marker| item.contains(marker)));
    output
        .recommendations
        .retain(|item| !NO_SOURCE_MARKERS.iter().any(|marker| item.contains(marker)));
    output
        .risks
        .retain(|item| !NO_SOURCE_MARKERS.iter().any(|marker| item.contains(marker)));
    output
        .open_questions
        .retain(|item| !NO_SOURCE_MARKERS.iter().any(|marker| item.contains(marker)));
}

fn source_text<'a>(source: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    source
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn compact_inline(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut text = value.chars().take(max_chars).collect::<String>();
    text.push_str("...");
    text
}

fn parse_role_work_output(value: &serde_json::Value) -> Option<RoleWorkOutput> {
    let source = value.get("role_output").unwrap_or(value);
    if let Some(text) = source.as_str() {
        let summary = text.trim().to_string();
        return (!summary.is_empty()).then_some(RoleWorkOutput {
            summary,
            evidence: string_array_field(value, "evidence"),
            risks: string_array_field(value, "risks"),
            open_questions: string_array_field(value, "open_questions"),
            ..RoleWorkOutput::default()
        });
    }

    let object = source.as_object()?;
    let output = RoleWorkOutput {
        summary: string_field_from_object(object, "summary").unwrap_or_default(),
        findings: string_array_field(source, "findings"),
        recommendations: string_array_field(source, "recommendations"),
        evidence: string_array_field(source, "evidence"),
        risks: string_array_field(source, "risks"),
        open_questions: string_array_field(source, "open_questions"),
    };

    has_role_work_output_content(&output).then_some(output)
}

fn has_role_work_output_content(output: &RoleWorkOutput) -> bool {
    !output.summary.trim().is_empty()
        || !output.findings.is_empty()
        || !output.recommendations.is_empty()
        || !output.evidence.is_empty()
        || !output.risks.is_empty()
        || !output.open_questions.is_empty()
}

fn extract_worker_content(output: &str) -> String {
    let trimmed = output.trim();
    let Ok(value) = llm_json::parse_llm_json_value(trimmed) else {
        return trimmed.to_string();
    };

    value
        .pointer("/decision/action/parameters/answer")
        .and_then(|value| value.as_str())
        .or_else(|| {
            value
                .pointer("/role_output")
                .and_then(|value| value.as_str())
        })
        .or_else(|| {
            value
                .pointer("/rationale/primary")
                .and_then(|value| value.as_str())
        })
        .map(ToString::to_string)
        .unwrap_or_else(|| trimmed.to_string())
}

fn extend_section(lines: &mut Vec<String>, title: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    lines.push(format!("{title}:"));
    lines.extend(items.iter().map(|item| format!("- {item}")));
}

fn string_field_from_object(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    object
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn string_array_field(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn format_success_skill_evidence(result: &SkillExecutionResult) -> String {
    let summary = result
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|summary| !summary.is_empty())
        .map(ToString::to_string)
        .or_else(|| result.output.as_ref().map(compact_json_preview))
        .unwrap_or_else(|| "completed".to_string());
    format!(
        "skill:{} status=success summary={}",
        result.skill_id.as_str(),
        summary
    )
}

fn format_incomplete_skill_risk(result: &SkillExecutionResult) -> String {
    format!(
        "skill:{} status={:?} reason={}",
        result.skill_id.as_str(),
        result.status,
        result
            .error
            .as_deref()
            .or(result.summary.as_deref())
            .unwrap_or("runtime adapter required")
    )
}

fn format_incomplete_skill_question(result: &SkillExecutionResult) -> String {
    format!(
        "skill:{} 尚未产出可用结果，需要补齐适配器、审批或外部执行后再判断。",
        result.skill_id.as_str()
    )
}

fn format_failed_skill_risk(result: &SkillExecutionResult) -> String {
    format!(
        "skill:{} status={:?} error={}",
        result.skill_id.as_str(),
        result.status,
        result.error.as_deref().unwrap_or("unknown error")
    )
}

fn compact_json_preview(value: &serde_json::Value) -> String {
    compact_event_text(value.to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};

    use super::{merge_skill_results, parse_worker_role_output};
    use crate::skills::{SkillExecutionResult, SkillExecutionStatus, SkillId};

    #[test]
    fn merge_skill_results_keeps_search_evidence_packet_for_downstream_workers() {
        let parsed = parse_worker_role_output(
            r#"{"role_output":{"summary":"本轮先整理搜索证据。","evidence":[]}}"#,
        );
        let results = vec![SkillExecutionResult {
            request_id: "req-search".to_string(),
            skill_id: SkillId::new("search_competitor"),
            status: SkillExecutionStatus::Success,
            output: Some(json!({
                "adapter_status": "live",
                "sources": [{
                    "title": "冷萃咖啡竞品价格参考",
                    "url": "https://example.com/cold-brew-price",
                    "published_at": "2026-05-20",
                    "snippet": "竞品价格集中在 12-19 元。"
                }],
                "evidence_packet": {
                    "intent": "competitor_price_or_benchmark",
                    "usable_claims": ["竞品价格集中在 12-19 元。"],
                    "gaps": [],
                    "confidence": 0.72
                }
            })),
            summary: Some("外部搜索已返回 1 个可采信来源".to_string()),
            metadata: Map::new(),
            error: None,
        }];

        let merged = merge_skill_results(parsed, &results);
        let evidence = merged
            .role_output
            .expect("role output should exist")
            .evidence;

        assert!(
            evidence
                .iter()
                .any(|item| item.starts_with("search_evidence_packet:")),
            "search evidence packet should be preserved in report evidence"
        );
        assert!(
            evidence
                .iter()
                .any(|item| item.contains("实时搜索来源（search_competitor）")),
            "human-readable live source list should still be preserved"
        );
    }
}
