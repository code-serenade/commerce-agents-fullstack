use utils::llm_json;

use super::{
    types::ParsedWorkerRoleOutput,
    util::{compact_event_text, push_unique},
};
use crate::{
    core::protocol::{RoleWorkOutput, WorkerDiagnostic},
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
    let diagnostics = search_diagnostics_from_evidence(&evidence);

    ParsedWorkerRoleOutput {
        content,
        role_output,
        evidence,
        risks,
        open_questions,
        diagnostics,
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

        if !matches!(result.status, SkillExecutionStatus::Success)
            && let Some(evidence_packet) = format_search_evidence_packet(result)
        {
            push_unique(&mut parsed.evidence, evidence_packet.clone());
            push_unique(&mut role_output.evidence, evidence_packet);
        }
        if let Some(trace) = format_search_evidence_trace(result) {
            push_unique(&mut parsed.evidence, trace.clone());
            push_unique(&mut role_output.evidence, trace);
        }
        if let Some(diagnostic) = format_search_evidence_diagnostic(result) {
            push_unique_diagnostic(&mut parsed.diagnostics, diagnostic);
        }
        if let Some(diagnostic) = format_skill_failure_diagnostic(result) {
            push_unique_diagnostic(&mut parsed.diagnostics, diagnostic);
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

fn push_unique_diagnostic(items: &mut Vec<WorkerDiagnostic>, diagnostic: WorkerDiagnostic) {
    if !items.contains(&diagnostic) {
        items.push(diagnostic);
    }
}

fn search_diagnostics_from_evidence(evidence: &[String]) -> Vec<WorkerDiagnostic> {
    let mut diagnostics = Vec::new();
    for item in evidence {
        if let Some(diagnostic) = search_trace_diagnostic(item) {
            push_unique_diagnostic(&mut diagnostics, diagnostic);
        }
    }
    diagnostics
}

fn search_trace_diagnostic(trace: &str) -> Option<WorkerDiagnostic> {
    let mut fields = parse_search_trace_fields(trace)?;
    normalize_search_trace_filter_fields(&mut fields);
    let source = fields
        .get("skill")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("search")
        .to_string();
    Some(WorkerDiagnostic {
        kind: "search_evidence".to_string(),
        source,
        fields,
    })
}

fn normalize_search_trace_filter_fields(fields: &mut std::collections::BTreeMap<String, String>) {
    for (alias, canonical) in [
        ("filtered", "total_filtered_count"),
        ("stale", "stale_filtered_count"),
        ("missing_url", "missing_url_filtered_count"),
        ("untraceable", "untraceable_domain_filtered_count"),
        ("duplicate", "duplicate_filtered_count"),
    ] {
        if fields.contains_key(canonical) {
            continue;
        }
        if let Some(value) = fields.get(alias).cloned() {
            fields.insert(canonical.to_string(), value);
        }
    }

    if !fields.contains_key("total_filtered_count") {
        let total = [
            "stale_filtered_count",
            "missing_url_filtered_count",
            "untraceable_domain_filtered_count",
            "duplicate_filtered_count",
        ]
        .into_iter()
        .filter_map(|key| {
            fields
                .get(key)
                .and_then(|value| value.parse::<usize>().ok())
        })
        .sum::<usize>();
        if total > 0 {
            fields.insert("total_filtered_count".to_string(), total.to_string());
        }
    }
}

fn parse_search_trace_fields(trace: &str) -> Option<std::collections::BTreeMap<String, String>> {
    let rest = trace.trim().strip_prefix("search_evidence_trace:")?;
    let fields = rest
        .split_whitespace()
        .filter_map(|part| part.split_once('='))
        .map(|(key, value)| {
            (
                key.trim().to_string(),
                value.trim().trim_matches('"').to_string(),
            )
        })
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
        .collect::<std::collections::BTreeMap<_, _>>();
    (!fields.is_empty()).then_some(fields)
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

fn format_search_evidence_diagnostic(result: &SkillExecutionResult) -> Option<WorkerDiagnostic> {
    if !result.skill_id.as_str().starts_with("search_") {
        return None;
    }
    let output = result.output.as_ref()?;
    let packet = output.get("evidence_packet")?;
    let mut fields = std::collections::BTreeMap::new();

    push_field(&mut fields, "skill", result.skill_id.as_str());
    push_field(
        &mut fields,
        "status",
        output
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| skill_status_label(result.status)),
    );
    push_field(
        &mut fields,
        "adapter_status",
        output
            .get("adapter_status")
            .and_then(serde_json::Value::as_str)
            .or_else(|| packet_text(packet, "adapter_status"))
            .or_else(|| {
                result
                    .metadata
                    .get("adapter_status")
                    .and_then(serde_json::Value::as_str)
            })
            .unwrap_or("unknown"),
    );
    push_field(
        &mut fields,
        "source_state",
        packet_text(packet, "source_state").unwrap_or("unknown"),
    );
    push_field(
        &mut fields,
        "intent",
        packet_text(packet, "intent").unwrap_or("general_search_evidence"),
    );
    push_field(
        &mut fields,
        "strength",
        packet_text(packet, "search_strength").unwrap_or("unknown"),
    );
    push_field(
        &mut fields,
        "sources",
        search_source_count(output, packet).to_string(),
    );
    push_filter_stat_fields(&mut fields, output, packet);
    push_field(
        &mut fields,
        "usable_claims",
        packet
            .get("usable_claims")
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len)
            .to_string(),
    );

    let gaps = packet
        .get("gaps")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    push_field(&mut fields, "gaps", gaps.len().to_string());
    if let Some(confidence) = packet.get("confidence").and_then(serde_json::Value::as_f64) {
        push_field(&mut fields, "confidence", format!("{confidence:.2}"));
    }
    if let Some(query) = packet_text(packet, "query").or_else(|| {
        output
            .get("planned_query")
            .and_then(serde_json::Value::as_str)
    }) {
        push_field(&mut fields, "query", compact_inline(query, 180));
    }

    let gap_preview = gaps
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(2)
        .collect::<Vec<_>>()
        .join(" | ");
    if !gap_preview.is_empty() {
        push_field(
            &mut fields,
            "gap_preview",
            compact_inline(&gap_preview, 220),
        );
    }

    Some(WorkerDiagnostic {
        kind: "search_evidence".to_string(),
        source: result.skill_id.as_str().to_string(),
        fields,
    })
}

fn format_skill_failure_diagnostic(result: &SkillExecutionResult) -> Option<WorkerDiagnostic> {
    if matches!(result.status, SkillExecutionStatus::Success) {
        return None;
    }

    let failure_kind = skill_failure_kind(result)?;
    let mut fields = std::collections::BTreeMap::new();
    push_field(&mut fields, "failure_kind", failure_kind);
    push_field(&mut fields, "status", skill_status_label(result.status));
    push_field(&mut fields, "skill", result.skill_id.as_str());
    push_field(&mut fields, "request_id", &result.request_id);
    if let Some(reason) = skill_failure_reason(result) {
        push_field(&mut fields, "reason", compact_inline(&reason, 220));
    }
    for key in [
        "execution_mode",
        "risk_level",
        "adapter_status",
        "data_readiness",
    ] {
        if let Some(value) = result.metadata.get(key).and_then(serde_json::Value::as_str) {
            push_field(&mut fields, key, value);
        }
    }
    if let Some(value) = result
        .metadata
        .get("approval_required")
        .and_then(serde_json::Value::as_bool)
    {
        push_field(&mut fields, "approval_required", value.to_string());
    }
    if let Some(output) = result.output.as_ref() {
        if let Some(value) = output
            .get("adapter_status")
            .and_then(serde_json::Value::as_str)
        {
            push_field(&mut fields, "adapter_status", value);
        }
        if let Some(value) = output
            .get("data_readiness")
            .and_then(serde_json::Value::as_str)
        {
            push_field(&mut fields, "data_readiness", value);
        }
    }

    Some(WorkerDiagnostic {
        kind: "failure".to_string(),
        source: failure_kind.to_string(),
        fields,
    })
}

fn skill_failure_kind(result: &SkillExecutionResult) -> Option<&'static str> {
    let reason = skill_failure_reason(result).unwrap_or_default();
    let lower = reason.to_ascii_lowercase();
    if lower.contains("timed out") || lower.contains("timeout") {
        return Some("skill_adapter_timeout");
    }

    match result.status {
        SkillExecutionStatus::Success => None,
        SkillExecutionStatus::Deferred => {
            if lower.contains("adapter")
                || lower.contains("executor unavailable")
                || lower.contains("not implemented")
                || lower.contains("尚未接入")
                || lower.contains("未接入")
            {
                Some("skill_adapter_unavailable")
            } else {
                Some("skill_deferred")
            }
        }
        SkillExecutionStatus::PendingApproval => Some("skill_pending_approval"),
        SkillExecutionStatus::NotFound => Some("skill_not_found"),
        SkillExecutionStatus::ValidationError => Some("skill_validation_error"),
        SkillExecutionStatus::Failed => Some("skill_execution_failed"),
    }
}

fn skill_failure_reason(result: &SkillExecutionResult) -> Option<String> {
    result
        .error
        .as_deref()
        .or(result.summary.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            result
                .output
                .as_ref()
                .and_then(|output| {
                    output
                        .get("message")
                        .or_else(|| output.get("reason"))
                        .or_else(|| output.get("error"))
                })
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
}

fn push_field(
    fields: &mut std::collections::BTreeMap<String, String>,
    key: impl Into<String>,
    value: impl Into<String>,
) {
    let value = value.into();
    if !value.trim().is_empty() {
        fields.insert(key.into(), value);
    }
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

fn format_search_evidence_trace(result: &SkillExecutionResult) -> Option<String> {
    if !result.skill_id.as_str().starts_with("search_") {
        return None;
    }
    let output = result.output.as_ref()?;
    let packet = output.get("evidence_packet")?;
    let intent = packet_text(packet, "intent").unwrap_or("general_search_evidence");
    let query = packet_text(packet, "query")
        .or_else(|| {
            output
                .get("planned_query")
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("");
    let strength = packet_text(packet, "search_strength").unwrap_or("unknown");
    let adapter_status = output
        .get("adapter_status")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            result
                .metadata
                .get("adapter_status")
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("unknown");
    let output_status = output
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| skill_status_label(result.status));
    let source_count = output
        .pointer("/provenance/source_count")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| source_array_count(output, "sources"))
        .or_else(|| source_array_count(packet, "sources"))
        .unwrap_or(0);
    let claim_count = packet
        .get("usable_claims")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let gaps = packet
        .get("gaps")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let gap_count = gaps.len();
    let confidence = packet
        .get("confidence")
        .and_then(serde_json::Value::as_f64)
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "unknown".to_string());
    let gap_preview = gaps
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(2)
        .collect::<Vec<_>>()
        .join(" | ");
    let filter_stats = search_filter_stats(output, packet);

    let mut trace = format!(
        "search_evidence_trace: skill={} status={} adapter_status={} intent={} strength={} \
         sources={} usable_claims={} gaps={} confidence={} filtered={} stale={} missing_url={} \
         untraceable={} duplicate={} query=\"{}\"",
        result.skill_id.as_str(),
        output_status,
        adapter_status,
        intent,
        strength,
        source_count,
        claim_count,
        gap_count,
        confidence,
        filter_stats.total_filtered_count,
        filter_stats.stale_filtered_count,
        filter_stats.missing_url_filtered_count,
        filter_stats.untraceable_domain_filtered_count,
        filter_stats.duplicate_filtered_count,
        compact_inline(query, 180)
    );
    if !gap_preview.is_empty() {
        trace.push_str(&format!(
            " gap_preview=\"{}\"",
            compact_inline(&gap_preview, 220)
        ));
    }
    Some(trace)
}

fn search_source_count(output: &serde_json::Value, packet: &serde_json::Value) -> u64 {
    output
        .pointer("/provenance/source_count")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| source_array_count(output, "sources"))
        .or_else(|| source_array_count(packet, "sources"))
        .unwrap_or(0)
}

#[derive(Debug, Default)]
struct SearchFilterStats {
    missing_url_filtered_count: u64,
    untraceable_domain_filtered_count: u64,
    duplicate_filtered_count: u64,
    stale_filtered_count: u64,
    total_filtered_count: u64,
}

fn push_filter_stat_fields(
    fields: &mut std::collections::BTreeMap<String, String>,
    output: &serde_json::Value,
    packet: &serde_json::Value,
) {
    let stats = search_filter_stats(output, packet);
    push_field(
        fields,
        "missing_url_filtered_count",
        stats.missing_url_filtered_count.to_string(),
    );
    push_field(
        fields,
        "untraceable_domain_filtered_count",
        stats.untraceable_domain_filtered_count.to_string(),
    );
    push_field(
        fields,
        "duplicate_filtered_count",
        stats.duplicate_filtered_count.to_string(),
    );
    push_field(
        fields,
        "stale_filtered_count",
        stats.stale_filtered_count.to_string(),
    );
    push_field(
        fields,
        "total_filtered_count",
        stats.total_filtered_count.to_string(),
    );
}

fn search_filter_stats(
    output: &serde_json::Value,
    packet: &serde_json::Value,
) -> SearchFilterStats {
    let missing_url_filtered_count =
        search_filter_stat(output, packet, "missing_url_filtered_count");
    let untraceable_domain_filtered_count =
        search_filter_stat(output, packet, "untraceable_domain_filtered_count");
    let duplicate_filtered_count = search_filter_stat(output, packet, "duplicate_filtered_count");
    let stale_filtered_count = search_filter_stat(output, packet, "stale_filtered_count");
    let total_filtered_count = search_filter_stat(output, packet, "total_filtered_count")
        .unwrap_or_else(|| {
            missing_url_filtered_count.unwrap_or(0)
                + untraceable_domain_filtered_count.unwrap_or(0)
                + duplicate_filtered_count.unwrap_or(0)
                + stale_filtered_count.unwrap_or(0)
        });

    SearchFilterStats {
        missing_url_filtered_count: missing_url_filtered_count.unwrap_or(0),
        untraceable_domain_filtered_count: untraceable_domain_filtered_count.unwrap_or(0),
        duplicate_filtered_count: duplicate_filtered_count.unwrap_or(0),
        stale_filtered_count: stale_filtered_count.unwrap_or(0),
        total_filtered_count,
    }
}

fn search_filter_stat(
    output: &serde_json::Value,
    packet: &serde_json::Value,
    key: &str,
) -> Option<u64> {
    [
        packet.pointer(&format!("/filter_stats/{key}")),
        output.pointer(&format!("/filter_stats/{key}")),
        output.pointer(&format!("/provenance/filter_stats/{key}")),
        output.pointer(&format!("/provenance/{key}")),
        output.pointer(&format!("/freshness/{key}")),
    ]
    .into_iter()
    .flatten()
    .find_map(serde_json::Value::as_u64)
}

fn source_array_count(value: &serde_json::Value, key: &str) -> Option<u64> {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| items.len() as u64)
}

fn packet_text<'a>(packet: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    packet
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn skill_status_label(status: SkillExecutionStatus) -> &'static str {
    match status {
        SkillExecutionStatus::Success => "success",
        SkillExecutionStatus::Deferred => "deferred",
        SkillExecutionStatus::PendingApproval => "pending_approval",
        SkillExecutionStatus::NotFound => "not_found",
        SkillExecutionStatus::ValidationError => "validation_error",
        SkillExecutionStatus::Failed => "failed",
    }
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
                    "adapter_status": "live",
                    "source_state": "has_reliable_sources",
                    "search_strength": "standard",
                    "filter_stats": {
                        "missing_url_filtered_count": 1,
                        "untraceable_domain_filtered_count": 0,
                        "duplicate_filtered_count": 1,
                        "stale_filtered_count": 0,
                        "total_filtered_count": 2
                    },
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
        let diagnostics = merged.diagnostics.clone();
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
        assert!(
            evidence.iter().any(|item| {
                item.starts_with("search_evidence_trace:")
                    && item.contains("skill=search_competitor")
                    && item.contains("adapter_status=live")
                    && item.contains("intent=competitor_price_or_benchmark")
                    && item.contains("filtered=2")
                    && item.contains("missing_url=1")
                    && item.contains("duplicate=1")
            }),
            "search evidence trace should summarize the live search path"
        );
        assert!(
            diagnostics.iter().any(|item| {
                item.kind == "search_evidence"
                    && item.source == "search_competitor"
                    && item.fields.get("adapter_status").map(String::as_str) == Some("live")
                    && item.fields.get("source_state").map(String::as_str)
                        == Some("has_reliable_sources")
                    && item.fields.get("sources").map(String::as_str) == Some("1")
                    && item.fields.get("usable_claims").map(String::as_str) == Some("1")
                    && item.fields.get("gaps").map(String::as_str) == Some("0")
                    && item.fields.get("confidence").map(String::as_str) == Some("0.72")
                    && item
                        .fields
                        .get("missing_url_filtered_count")
                        .map(String::as_str)
                        == Some("1")
                    && item
                        .fields
                        .get("duplicate_filtered_count")
                        .map(String::as_str)
                        == Some("1")
                    && item.fields.get("total_filtered_count").map(String::as_str) == Some("2")
            }),
            "search evidence diagnostic should expose typed live adapter state"
        );
    }

    #[test]
    fn merge_skill_results_keeps_degraded_search_evidence_packet_for_downstream_workers() {
        let parsed = parse_worker_role_output(
            r#"{"role_output":{"summary":"本轮搜索未拿到来源，但需要把缺口交给下游。","evidence":[]}}"#,
        );
        let results = vec![SkillExecutionResult {
            request_id: "req-search".to_string(),
            skill_id: SkillId::new("search_platform_policy"),
            status: SkillExecutionStatus::Deferred,
            output: Some(json!({
                "status": "deferred",
                "adapter_status": "degraded",
                "planned_query": "抖音 价格合规 官方规则 最新 2026",
                "provenance": {
                    "source_count": 0
                },
                    "evidence_packet": {
                        "intent": "platform_policy_or_compliance",
                        "skill": "search_platform_policy",
                        "adapter_status": "degraded",
                        "source_state": "no_reliable_sources",
                        "query": "抖音 价格合规 官方规则 最新 2026",
                        "search_strength": "standard",
                        "filter_stats": {
                            "missing_url_filtered_count": 0,
                            "untraceable_domain_filtered_count": 1,
                            "duplicate_filtered_count": 0,
                            "stale_filtered_count": 2,
                            "total_filtered_count": 3
                        },
                        "sources": [],
                    "usable_claims": [],
                    "gaps": ["provider timeout"],
                    "confidence": 0.0
                }
            })),
            summary: Some("外部搜索 adapter 未返回可采信来源，已按降级结果处理".to_string()),
            metadata: Map::new(),
            error: None,
        }];

        let merged = merge_skill_results(parsed, &results);
        let diagnostics = merged.diagnostics.clone();
        let role_output = merged.role_output.expect("role output should exist");

        assert!(
            role_output
                .evidence
                .iter()
                .any(|item| item.starts_with("search_evidence_packet:")),
            "degraded search evidence packet should still be preserved in report evidence"
        );
        assert!(
            role_output.evidence.iter().any(|item| {
                item.starts_with("search_evidence_trace:")
                    && item.contains("skill=search_platform_policy")
                    && item.contains("adapter_status=degraded")
                    && item.contains("sources=0")
                    && item.contains("gaps=1")
                    && item.contains("filtered=3")
                    && item.contains("stale=2")
                    && item.contains("untraceable=1")
                    && item.contains("provider timeout")
            }),
            "degraded search trace should expose status, source count, gaps and query"
        );
        assert!(
            role_output
                .risks
                .iter()
                .any(|item| item.contains("search_platform_policy")),
            "degraded search should still keep the incomplete skill risk"
        );
        assert!(
            diagnostics.iter().any(|item| {
                item.kind == "search_evidence"
                    && item.source == "search_platform_policy"
                    && item.fields.get("adapter_status").map(String::as_str) == Some("degraded")
                    && item.fields.get("source_state").map(String::as_str)
                        == Some("no_reliable_sources")
                    && item.fields.get("sources").map(String::as_str) == Some("0")
                    && item.fields.get("usable_claims").map(String::as_str) == Some("0")
                    && item.fields.get("gaps").map(String::as_str) == Some("1")
                    && item.fields.get("confidence").map(String::as_str) == Some("0.00")
                    && item.fields.get("stale_filtered_count").map(String::as_str) == Some("2")
                    && item
                        .fields
                        .get("untraceable_domain_filtered_count")
                        .map(String::as_str)
                        == Some("1")
                    && item.fields.get("total_filtered_count").map(String::as_str) == Some("3")
            }),
            "search evidence diagnostic should expose typed degraded adapter state"
        );
    }

    #[test]
    fn parse_worker_role_output_promotes_search_trace_to_typed_diagnostic() {
        let parsed = parse_worker_role_output(
            r#"{"role_output":{"summary":"搜索岗已整理证据。","evidence":["search_evidence_packet:{\"intent\":\"competitor_price_or_benchmark\"}","search_evidence_trace: skill=search_competitor status=success adapter_status=live intent=competitor_price_or_benchmark strength=standard sources=1 usable_claims=1 gaps=0 confidence=0.72 filtered=4 stale=1 missing_url=1 untraceable=1 duplicate=1 query=\"校园冷萃咖啡 竞品价格\""]}}"#,
        );

        assert!(
            parsed.diagnostics.iter().any(|item| {
                item.kind == "search_evidence"
                    && item.source == "search_competitor"
                    && item.fields.get("adapter_status").map(String::as_str) == Some("live")
                    && item.fields.get("sources").map(String::as_str) == Some("1")
                    && item.fields.get("usable_claims").map(String::as_str) == Some("1")
                    && item.fields.get("gaps").map(String::as_str) == Some("0")
                    && item.fields.get("confidence").map(String::as_str) == Some("0.72")
                    && item.fields.get("total_filtered_count").map(String::as_str) == Some("4")
                    && item.fields.get("stale_filtered_count").map(String::as_str) == Some("1")
                    && item
                        .fields
                        .get("missing_url_filtered_count")
                        .map(String::as_str)
                        == Some("1")
                    && item
                        .fields
                        .get("untraceable_domain_filtered_count")
                        .map(String::as_str)
                        == Some("1")
                    && item
                        .fields
                        .get("duplicate_filtered_count")
                        .map(String::as_str)
                        == Some("1")
            }),
            "legacy search_evidence_trace should be promoted to typed diagnostics"
        );
    }

    #[test]
    fn merge_skill_results_adds_typed_failure_diagnostics_for_skill_errors() {
        let parsed = parse_worker_role_output(
            r#"{"role_output":{"summary":"本轮先尝试调用指标 Skill。","evidence":[]}}"#,
        );
        let results = vec![
            SkillExecutionResult {
                request_id: "req-validation".to_string(),
                skill_id: SkillId::new("query_store_metrics"),
                status: SkillExecutionStatus::ValidationError,
                output: None,
                summary: None,
                metadata: Map::new(),
                error: Some("missing required metric input".to_string()),
            },
            SkillExecutionResult {
                request_id: "req-adapter".to_string(),
                skill_id: SkillId::new("platform_update_price"),
                status: SkillExecutionStatus::Deferred,
                output: Some(json!({
                    "adapter_status": "adapter_not_implemented",
                    "data_readiness": "adapter_required",
                    "message": "Platform execution adapter is not implemented yet."
                })),
                summary: Some("平台执行 adapter 尚未接入".to_string()),
                metadata: Map::new(),
                error: None,
            },
        ];

        let merged = merge_skill_results(parsed, &results);

        assert!(
            merged.diagnostics.iter().any(|item| {
                item.kind == "failure"
                    && item.source == "skill_validation_error"
                    && item.fields.get("failure_kind").map(String::as_str)
                        == Some("skill_validation_error")
                    && item.fields.get("status").map(String::as_str) == Some("validation_error")
                    && item.fields.get("skill").map(String::as_str) == Some("query_store_metrics")
                    && item
                        .fields
                        .get("reason")
                        .is_some_and(|value| value.contains("missing required metric input"))
            }),
            "validation errors should become typed failure diagnostics"
        );
        assert!(
            merged.diagnostics.iter().any(|item| {
                item.kind == "failure"
                    && item.source == "skill_adapter_unavailable"
                    && item.fields.get("failure_kind").map(String::as_str)
                        == Some("skill_adapter_unavailable")
                    && item.fields.get("status").map(String::as_str) == Some("deferred")
                    && item.fields.get("skill").map(String::as_str) == Some("platform_update_price")
                    && item.fields.get("adapter_status").map(String::as_str)
                        == Some("adapter_not_implemented")
                    && item.fields.get("data_readiness").map(String::as_str)
                        == Some("adapter_required")
            }),
            "adapter unavailable states should become typed failure diagnostics"
        );
    }
}
