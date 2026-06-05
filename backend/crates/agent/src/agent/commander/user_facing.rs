use std::{collections::BTreeMap, fs, path::Path, sync::OnceLock};

use serde::Deserialize;

use crate::{
    core::protocol::WorkerReport,
    llm_routing::{
        is_llm_access_denied_message, is_llm_provider_error_message,
        llm_access_denied_user_message, llm_provider_error_user_message,
    },
};

const USER_FACING_NORMALIZATION_POLICY: &str =
    include_str!("../../../../../config/json/agent/user-facing-normalization-policy.json");

#[derive(Debug, Clone, Deserialize)]
struct UserFacingNormalizationPolicy {
    no_user_facing_content_message: Vec<String>,
    #[serde(default)]
    material_gap_markers: Vec<String>,
    #[serde(default)]
    material_gap_business_markers: Vec<String>,
    live_search_default_success_message: String,
    #[serde(default)]
    live_search_degraded_lower_markers: Vec<String>,
    #[serde(default)]
    live_search_degraded_markers: Vec<String>,
    #[serde(default)]
    live_search_success_markers: Vec<String>,
    #[serde(default)]
    live_search_required_evidence_markers: Vec<String>,
    #[serde(default)]
    unverified_live_search_success_line_markers: Vec<String>,
    #[serde(default)]
    unverified_live_search_success_phrases: Vec<String>,
    #[serde(default)]
    no_source_claim_markers: Vec<String>,
    #[serde(default)]
    internal_context_markers: Vec<String>,
    #[serde(default)]
    internal_skill_name_replacements: BTreeMap<String, String>,
    #[serde(default)]
    business_phrase_replacements: Vec<[String; 2]>,
    #[serde(default)]
    unverified_live_search_availability_phrases: Vec<String>,
    #[serde(default)]
    memory_context_leak_markers: Vec<String>,
    #[serde(default)]
    internal_noise_lower_markers: Vec<String>,
    #[serde(default)]
    internal_noise_markers: Vec<String>,
    #[serde(default)]
    prompt_instruction_leak_markers: Vec<String>,
}

static USER_FACING_POLICY: OnceLock<UserFacingNormalizationPolicy> = OnceLock::new();

fn user_facing_policy() -> &'static UserFacingNormalizationPolicy {
    USER_FACING_POLICY.get_or_init(load_user_facing_policy)
}

fn load_user_facing_policy() -> UserFacingNormalizationPolicy {
    load_user_facing_policy_from_path(Path::new(
        "config/json/agent/user-facing-normalization-policy.json",
    ))
    .unwrap_or_else(|| {
        serde_json::from_str(USER_FACING_NORMALIZATION_POLICY).unwrap_or_else(|err| {
            tracing::warn!(
                error = %err,
                "failed to parse embedded user-facing normalization policy"
            );
            UserFacingNormalizationPolicy::fallback()
        })
    })
}

fn load_user_facing_policy_from_path(path: &Path) -> Option<UserFacingNormalizationPolicy> {
    let content = fs::read_to_string(path).ok()?;
    match serde_json::from_str(&content) {
        Ok(policy) => Some(policy),
        Err(err) => {
            tracing::warn!(
                path = %path.display(),
                error = %err,
                "failed to load user-facing normalization policy; using embedded policy"
            );
            None
        }
    }
}

impl UserFacingNormalizationPolicy {
    fn fallback() -> Self {
        Self {
            no_user_facing_content_message: vec![
                "## 当前状态".to_string(),
                String::new(),
                "这次没有拿到可直接展示的完整内容。".to_string(),
                String::new(),
                "## 下一步".to_string(),
                String::new(),
                "- 可以补充更明确的问题、商品、活动或关键材料。".to_string(),
            ],
            material_gap_markers: Vec::new(),
            material_gap_business_markers: Vec::new(),
            live_search_default_success_message: "本轮已通过外部搜索取得实时来源。".to_string(),
            live_search_degraded_lower_markers: Vec::new(),
            live_search_degraded_markers: Vec::new(),
            live_search_success_markers: Vec::new(),
            live_search_required_evidence_markers: Vec::new(),
            unverified_live_search_success_line_markers: Vec::new(),
            unverified_live_search_success_phrases: Vec::new(),
            no_source_claim_markers: Vec::new(),
            internal_context_markers: Vec::new(),
            internal_skill_name_replacements: BTreeMap::new(),
            business_phrase_replacements: Vec::new(),
            unverified_live_search_availability_phrases: Vec::new(),
            memory_context_leak_markers: Vec::new(),
            internal_noise_lower_markers: Vec::new(),
            internal_noise_markers: Vec::new(),
            prompt_instruction_leak_markers: Vec::new(),
        }
    }
}

pub(crate) fn user_facing_report_text(report: &WorkerReport) -> Option<String> {
    if report_has_llm_access_denied(report) {
        return Some(llm_access_denied_user_message().to_string());
    }
    if report_has_llm_provider_error(report) {
        return Some(llm_provider_error_user_message().to_string());
    }

    report
        .role_output
        .as_ref()
        .and_then(|output| role_output_user_facing_text(&report.role, output))
        .or_else(|| sanitize_user_facing_text(&report.content))
}

pub(crate) fn report_has_llm_access_denied(report: &WorkerReport) -> bool {
    report_has_failure_diagnostic(report, "llm_access_denied")
        || is_llm_access_denied_message(&report.content)
        || report
            .open_questions
            .iter()
            .any(|item| is_llm_access_denied_message(item))
        || report
            .risks
            .iter()
            .any(|item| is_llm_access_denied_message(item))
        || report
            .evidence
            .iter()
            .any(|item| is_llm_access_denied_message(item))
        || report.role_output.as_ref().is_some_and(|output| {
            is_llm_access_denied_message(&output.summary)
                || output
                    .findings
                    .iter()
                    .any(|item| is_llm_access_denied_message(item))
                || output
                    .recommendations
                    .iter()
                    .any(|item| is_llm_access_denied_message(item))
                || output
                    .open_questions
                    .iter()
                    .any(|item| is_llm_access_denied_message(item))
                || output
                    .risks
                    .iter()
                    .any(|item| is_llm_access_denied_message(item))
                || output
                    .evidence
                    .iter()
                    .any(|item| is_llm_access_denied_message(item))
        })
}

pub(crate) fn report_has_llm_provider_error(report: &WorkerReport) -> bool {
    report_has_failure_diagnostic(report, "llm_provider_error")
        || is_llm_provider_error_message(&report.content)
        || report
            .open_questions
            .iter()
            .any(|item| is_llm_provider_error_message(item))
        || report
            .risks
            .iter()
            .any(|item| is_llm_provider_error_message(item))
        || report
            .evidence
            .iter()
            .any(|item| is_llm_provider_error_message(item))
        || report.role_output.as_ref().is_some_and(|output| {
            is_llm_provider_error_message(&output.summary)
                || output
                    .findings
                    .iter()
                    .any(|item| is_llm_provider_error_message(item))
                || output
                    .recommendations
                    .iter()
                    .any(|item| is_llm_provider_error_message(item))
                || output
                    .open_questions
                    .iter()
                    .any(|item| is_llm_provider_error_message(item))
                || output
                    .risks
                    .iter()
                    .any(|item| is_llm_provider_error_message(item))
                || output
                    .evidence
                    .iter()
                    .any(|item| is_llm_provider_error_message(item))
        })
}

pub(crate) fn report_has_failure_diagnostic(report: &WorkerReport, failure_kind: &str) -> bool {
    report.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == "failure"
            && (diagnostic.source == failure_kind
                || diagnostic
                    .fields
                    .get("failure_kind")
                    .is_some_and(|value| value == failure_kind))
    })
}

fn role_output_user_facing_text(
    role: &str,
    output: &crate::core::protocol::RoleWorkOutput,
) -> Option<String> {
    let has_live_search = has_reliable_live_search_evidence(output);
    let live_search = live_search_user_facing_text(output);
    if role == "search" {
        return live_search.or_else(|| {
            role_output_business_text(output)
                .map(|text| remove_unverified_live_search_success_claims(&text, has_live_search))
                .filter(|text| !text.trim().is_empty())
        });
    }

    let business_text = role_output_business_text(output)
        .map(|text| remove_unverified_live_search_success_claims(&text, has_live_search))
        .filter(|text| !text.trim().is_empty());
    if let Some(text) = business_text
        .as_ref()
        .filter(|text| !looks_like_no_source_claim(text))
    {
        return Some(append_live_search_evidence_once(text, output));
    }

    live_search.or(business_text)
}

fn role_output_business_text(output: &crate::core::protocol::RoleWorkOutput) -> Option<String> {
    let has_material_evidence = output
        .evidence
        .iter()
        .chain(output.findings.iter())
        .any(|item| looks_like_material_evidence(item));
    let sections = std::iter::once(output.summary.as_str())
        .chain(output.findings.iter().map(String::as_str))
        .chain(output.recommendations.iter().map(String::as_str))
        .chain(output.risks.iter().map(String::as_str))
        .chain(output.open_questions.iter().map(String::as_str))
        .filter(|item| !looks_like_live_search_evidence(item))
        .filter_map(|item| {
            let text = sanitize_user_facing_text(item)?;
            if (has_material_evidence && looks_like_no_source_claim(&text))
                || looks_like_blocking_material_gap(&text)
            {
                None
            } else {
                Some(text)
            }
        })
        .fold(Vec::<String>::new(), |mut sections, text| {
            push_unique_section(&mut sections, text);
            sections
        });
    let text = if sections.is_empty() {
        material_evidence_sections(output).into_iter().next()?
    } else {
        sections.join("\n\n")
    };
    Some(append_material_evidence_once(&text, output))
}

fn push_unique_section(sections: &mut Vec<String>, text: String) {
    let text = text.trim();
    if text.is_empty() || sections.iter().any(|existing| existing == text) {
        return;
    }
    sections.push(text.to_string());
}

fn append_material_evidence_once(
    business_text: &str,
    output: &crate::core::protocol::RoleWorkOutput,
) -> String {
    if business_text.contains("资料依据：") || business_text.contains("资料关键事实：")
    {
        return business_text.to_string();
    }
    let evidence = material_evidence_sections(output);
    if evidence.is_empty() || material_evidence_already_covered(business_text, &evidence) {
        return business_text.to_string();
    }

    let mut sections = vec![business_text.to_string()];
    sections.extend(evidence);
    sections.join("\n\n")
}

fn material_evidence_sections(output: &crate::core::protocol::RoleWorkOutput) -> Vec<String> {
    output
        .evidence
        .iter()
        .chain(output.findings.iter())
        .filter(|item| looks_like_material_evidence(item))
        .filter_map(|item| sanitize_user_facing_text(item))
        .fold(Vec::<String>::new(), |mut items, item| {
            if !items.iter().any(|existing| existing == &item) {
                items.push(item);
            }
            items
        })
}

fn looks_like_material_evidence(item: &str) -> bool {
    let item = item.trim();
    (item.starts_with("资料依据：")
        || item.starts_with("资料关键事实：")
        || item.starts_with("资料摘要："))
        && !looks_like_live_search_evidence(item)
}

fn looks_like_material_boundary(item: &str) -> bool {
    item.trim().starts_with("资料边界：")
}

fn looks_like_blocking_material_gap(item: &str) -> bool {
    let text = item.trim();
    if looks_like_material_boundary(text) {
        return false;
    }
    let policy = user_facing_policy();
    let has_gap_marker = policy
        .material_gap_markers
        .iter()
        .any(|marker| text.contains(marker));
    if !has_gap_marker {
        return false;
    }
    let has_business_content = policy
        .material_gap_business_markers
        .iter()
        .any(|marker| text.contains(marker));
    !has_business_content || text.chars().count() < 160
}

fn material_evidence_already_covered(business_text: &str, evidence: &[String]) -> bool {
    let numeric_tokens = evidence
        .iter()
        .flat_map(|item| numeric_tokens(item))
        .collect::<Vec<_>>();
    !numeric_tokens.is_empty()
        && numeric_tokens
            .iter()
            .any(|token| business_text.contains(token.as_str()))
}

fn numeric_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit()
            || matches!(
                ch,
                '.' | '%' | ',' | '元' | '万' | '亿' | 'k' | 'K' | 'w' | 'W'
            )
        {
            current.push(ch);
            continue;
        }
        push_numeric_token(&mut tokens, &mut current);
    }
    push_numeric_token(&mut tokens, &mut current);
    tokens
}

fn push_numeric_token(tokens: &mut Vec<String>, current: &mut String) {
    let token = current.trim_matches(',').trim();
    if token.chars().any(|ch| ch.is_ascii_digit()) && token.chars().count() >= 2 {
        let token = token.to_string();
        if !tokens.iter().any(|existing| existing == &token) {
            tokens.push(token);
        }
    }
    current.clear();
}

fn append_live_search_evidence_once(
    business_text: &str,
    output: &crate::core::protocol::RoleWorkOutput,
) -> String {
    if looks_like_live_search_evidence(business_text) {
        return business_text.to_string();
    }
    let evidence = live_search_evidence_sections(output);
    if evidence.is_empty() {
        return business_text.to_string();
    }

    let mut sections = vec![business_text.to_string()];
    sections.extend(evidence);
    sections.join("\n\n")
}

fn live_search_user_facing_text(output: &crate::core::protocol::RoleWorkOutput) -> Option<String> {
    let live_evidence = live_search_evidence_sections(output);
    if live_evidence.is_empty() {
        return None;
    }

    let mut sections = Vec::new();
    if let Some(summary) = sanitize_user_facing_text(&output.summary)
        .filter(|summary| !looks_like_no_source_claim(summary))
    {
        sections.push(summary);
    } else {
        sections.push(
            user_facing_policy()
                .live_search_default_success_message
                .clone(),
        );
    }
    sections.extend(live_evidence);

    (!sections.is_empty()).then_some(sections.join("\n\n"))
}

fn live_search_evidence_sections(output: &crate::core::protocol::RoleWorkOutput) -> Vec<String> {
    output
        .evidence
        .iter()
        .filter(|item| looks_like_live_search_evidence(item))
        .filter_map(|item| sanitize_user_facing_text(item))
        .fold(Vec::<String>::new(), |mut items, item| {
            if !items.iter().any(|existing| existing == &item) {
                items.push(item);
            }
            items
        })
}

fn has_reliable_live_search_evidence(output: &crate::core::protocol::RoleWorkOutput) -> bool {
    output
        .evidence
        .iter()
        .chain(output.findings.iter())
        .any(|item| looks_like_live_search_evidence(item))
}

fn looks_like_live_search_evidence(item: &str) -> bool {
    let item = item.trim();
    let lower = item.to_ascii_lowercase();
    let policy = user_facing_policy();
    if policy
        .live_search_degraded_lower_markers
        .iter()
        .any(|marker| lower.contains(marker))
        || policy
            .live_search_degraded_markers
            .iter()
            .any(|marker| item.contains(marker))
    {
        return false;
    }

    policy
        .live_search_success_markers
        .iter()
        .any(|marker| item.contains(marker))
        && policy
            .live_search_required_evidence_markers
            .iter()
            .any(|marker| item.contains(marker))
}

fn remove_unverified_live_search_success_claims(text: &str, has_live_search: bool) -> String {
    if has_live_search {
        return text.to_string();
    }

    let lines = text
        .lines()
        .filter_map(strip_unverified_live_search_success_claim_line)
        .collect::<Vec<_>>();
    normalize_blank_lines(lines)
}

fn strip_unverified_live_search_success_claim_line(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let policy = user_facing_policy();
    if policy
        .unverified_live_search_success_line_markers
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return None;
    }

    let mut text = line.to_string();
    for phrase in &policy.unverified_live_search_success_phrases {
        text = text.replace(phrase, "");
    }

    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn looks_like_no_source_claim(item: &str) -> bool {
    user_facing_policy()
        .no_source_claim_markers
        .iter()
        .any(|marker| item.contains(marker))
}

pub(crate) fn sanitize_user_facing_text(raw: &str) -> Option<String> {
    let raw = normalize_escaped_newlines(raw);
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let text = remove_internal_context_fragments(&remove_placeholder_phrases(
        &remove_placeholder_brackets(raw),
    ));
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            lines.push(String::new());
            continue;
        }
        let line = clean_user_facing_line(line);
        if line.is_empty() || is_internal_noise_line(&line) {
            continue;
        }
        lines.push(line);
    }
    let lines = drop_orphan_section_headings(lines);

    let output = normalize_blank_lines(lines);
    (!output.is_empty() && !is_internal_noise_line(&output)).then_some(output)
}

fn normalize_escaped_newlines(raw: &str) -> String {
    let mut output = String::new();
    let chars = raw.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\\' {
            let mut next = index + 1;
            while next < chars.len() && chars[next].is_whitespace() {
                next += 1;
            }
            if next < chars.len() && chars[next] == 'n' {
                output.push('\n');
                index = next + 1;
                continue;
            }
        }
        output.push(chars[index]);
        index += 1;
    }
    output
}

pub(crate) fn sanitize_final_user_facing_text(raw: &str) -> Option<String> {
    let output = sanitize_user_facing_text(raw)?;
    let output = restore_readable_answer_layout(&output);
    let lines = output.lines().map(str::to_string).collect::<Vec<_>>();
    let lines = drop_historical_evidence_quote_blocks(lines);
    let lines = truncate_tail_historical_answer_quotes(lines);
    let output = normalize_blank_lines(lines);
    (!output.is_empty() && !is_internal_noise_line(&output)).then_some(output)
}

pub(crate) fn no_user_facing_content_message() -> String {
    user_facing_policy()
        .no_user_facing_content_message
        .join("\n")
}

fn clean_user_facing_line(line: &str) -> String {
    let line = strip_role_prefix(line.trim());
    let line = rewrite_note_prefix(&line);
    let line = rewrite_internal_evidence_terms(&line);
    let line = rewrite_misplaced_evidence_heading(&line);
    let line = rewrite_selected_product_source_line(&line);
    let line = rewrite_internal_skill_names(&line);
    let line = rewrite_business_phrases(&line);
    let line = remove_unverified_live_search_availability_claims_from_line(&line);
    let line = hide_internal_ids(&line);
    let line = compact_line_spacing(&line);
    if is_empty_list_artifact(&line) {
        String::new()
    } else {
        line
    }
}

fn remove_internal_context_fragments(input: &str) -> String {
    input
        .lines()
        .filter_map(trim_internal_context_fragment)
        .collect::<Vec<_>>()
        .join("\n")
}

fn trim_internal_context_fragment(line: &str) -> Option<String> {
    if line.trim().is_empty() {
        return Some(String::new());
    }

    let mut keep_end = line.len();
    let lower = line.to_ascii_lowercase();
    if looks_like_memory_context_leak(&lower) {
        return None;
    }

    for marker in &user_facing_policy().internal_context_markers {
        if let Some(index) = lower.find(marker) {
            keep_end = keep_end.min(index);
        }
    }
    if lower.contains("product_knowledge:")
        && let Some(index) = lower.find(" title=")
    {
        keep_end = keep_end.min(index);
    }

    let candidate = line[.. keep_end].trim_end();
    (!candidate.trim().is_empty()).then(|| candidate.to_string())
}

fn rewrite_internal_skill_names(line: &str) -> String {
    let mut text = line.to_string();
    for (from, to) in &user_facing_policy().internal_skill_name_replacements {
        text = text.replace(from, to);
    }
    text
}

fn rewrite_business_phrases(line: &str) -> String {
    let mut text = line.to_string();
    for [from, to] in &user_facing_policy().business_phrase_replacements {
        text = text.replace(from, to);
    }
    text
}

fn remove_unverified_live_search_availability_claims_from_line(line: &str) -> String {
    let mut text = line.to_string();
    for phrase in &user_facing_policy().unverified_live_search_availability_phrases {
        text = text.replace(phrase, "");
    }
    text.trim_start_matches(['，', ',', '。', '.', '；', ';', ' '])
        .to_string()
}

fn hide_internal_ids(line: &str) -> String {
    let line = replace_uuid_like_tokens(line);
    let mut text = line
        .replace("商品ID 已选商品、", "")
        .replace("商品ID已选商品、", "")
        .replace("商品ID 已选商品，", "")
        .replace("商品ID已选商品，", "")
        .replace("商品ID 已选商品,", "")
        .replace("商品ID已选商品,", "")
        .replace("商品ID 已选商品", "已选商品")
        .replace("商品ID已选商品", "已选商品")
        .replace("product:已选商品", "已选商品")
        .replace("product=已选商品", "已选商品");
    while text.contains("  ") {
        text = text.replace("  ", " ");
    }
    text
}

fn replace_uuid_like_tokens(input: &str) -> String {
    let mut output = String::new();
    let mut index = 0;

    while index < input.len() {
        if index + 36 <= input.len()
            && input.is_char_boundary(index)
            && input.is_char_boundary(index + 36)
            && is_uuid_boundary(input, index, true)
            && is_uuid_boundary(input, index + 36, false)
            && is_uuid_like(&input[index .. index + 36])
        {
            output.push_str("已选商品");
            index += 36;
            continue;
        }

        let Some(ch) = input[index ..].chars().next() else {
            break;
        };
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

fn is_uuid_boundary(input: &str, index: usize, before: bool) -> bool {
    let ch = if before {
        input[.. index].chars().next_back()
    } else {
        input[index ..].chars().next()
    };
    ch.is_none_or(|ch| !ch.is_ascii_hexdigit() && ch != '-')
}

fn is_uuid_like(value: &str) -> bool {
    value.len() == 36
        && value.char_indices().all(|(index, ch)| match index {
            8 | 13 | 18 | 23 => ch == '-',
            _ => ch.is_ascii_hexdigit(),
        })
}

fn remove_placeholder_brackets(input: &str) -> String {
    let text = replace_placeholder_pairs(input, '【', '】');
    replace_placeholder_pairs(&text, '[', ']')
}

fn replace_placeholder_pairs(input: &str, open: char, close: char) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find(open) {
        output.push_str(&rest[.. start]);
        let segment_start = start + open.len_utf8();
        let Some(end) = rest[segment_start ..].find(close) else {
            output.push_str(&rest[start ..]);
            return output;
        };
        let segment_end = segment_start + end;
        let segment = &rest[segment_start .. segment_end];
        if placeholder_segment(segment) {
            if let Some(replacement) = placeholder_replacement(segment) {
                output.push_str(&replacement);
            }
        } else {
            output.push(open);
            output.push_str(segment);
            output.push(close);
        }
        rest = &rest[segment_end + close.len_utf8() ..];
    }

    output.push_str(rest);
    output
}

fn placeholder_segment(segment: &str) -> bool {
    let lower = segment.to_ascii_lowercase();
    segment.contains("占位符")
        || segment.contains("需替换")
        || segment.contains("替换为真实")
        || segment.contains("真实品类")
        || segment.contains("真实规格")
        || segment.contains("真实价格")
        || segment.contains("真实售价")
        || segment.contains("到手价")
        || segment.contains("待填写")
        || segment.contains("待补充")
        || lower.contains("placeholder")
        || lower.contains("replace")
}

fn placeholder_replacement(segment: &str) -> Option<String> {
    let mut text = remove_placeholder_phrases(segment);
    for delimiter in ["（", "(", "，", ",", "如"] {
        if let Some(index) = text.find(delimiter) {
            text.truncate(index);
        }
    }
    text = text.replace('/', "、");
    let text = text
        .trim()
        .trim_matches(['：', ':', '。', '、', '.', ' '])
        .to_string();
    if text.is_empty()
        || text.contains("...")
        || text.contains('…')
        || text.contains("占位符")
        || !placeholder_field_label(&text)
        || text.chars().count() > 24
    {
        return None;
    }
    Some(text)
}

fn placeholder_field_label(text: &str) -> bool {
    text.contains("真实")
        || text.contains("规格")
        || text.contains("到手价")
        || text.contains("售价")
        || text.contains("价格")
        || text.contains("字段")
}

fn remove_placeholder_phrases(input: &str) -> String {
    let mut text = input.to_string();
    for phrase in [
        "（占位符需替换为真实信息）",
        "(占位符需替换为真实信息)",
        "【占位符需替换为真实信息】",
        "[占位符需替换为真实信息]",
        "占位符需替换为真实信息",
        "占位符需替换",
        "需替换为真实信息",
        "请替换为真实信息",
        "给用户看的完整中文回复：",
        "给用户看的完整中文回复:",
        "给用户看的完整中文回复",
        "placeholder",
        "PLACEHOLDER",
    ] {
        text = text.replace(phrase, "");
    }
    text
}

fn rewrite_note_prefix(line: &str) -> String {
    line.replace("备注：", "说明：").replace("备注:", "说明：")
}

fn restore_readable_answer_layout(text: &str) -> String {
    let text = insert_breaks_before_bracket_headings(text);
    let text = insert_breaks_after_bracket_headings(&text);
    let text = split_inline_markdown_heading_content(&text);
    let text = insert_breaks_around_inline_colon_headings(&text);
    let text = insert_breaks_before_inline_numbered_items(&text);
    let text = split_inline_markdown_tables(&text);
    let text = repair_broken_markdown_table_rows(&text);
    let text = promote_plain_section_headings(&text);
    let text = format_readable_answer_lists(&text);
    normalize_blank_lines(text.lines().map(str::to_string).collect())
}

fn insert_breaks_before_bracket_headings(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();
    let mut at_line_start = true;
    while let Some(ch) = chars.next() {
        if ch == '【' && !at_line_start && looks_like_bracket_heading_ahead(chars.clone()) {
            trim_trailing_spaces(&mut output);
            ensure_paragraph_break(&mut output);
        }
        output.push(ch);
        at_line_start = ch == '\n';
    }
    output
}

fn looks_like_bracket_heading_ahead<I>(chars: I) -> bool
where
    I: Iterator<Item = char>,
{
    let mut content = String::new();
    for ch in chars.take(36) {
        if ch == '】' {
            return !content.is_empty()
                && content.chars().count() <= 24
                && content
                    .chars()
                    .any(|ch| ch.is_ascii_alphabetic() || is_cjk(ch))
                && !content.contains("如")
                && !content.contains('/')
                && !content.contains(',');
        }
        if ch == '\n' {
            return false;
        }
        content.push(ch);
    }
    false
}

fn insert_breaks_after_bracket_headings(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        output.push(ch);
        if ch == '】' {
            let heading_start = output.rfind('【').unwrap_or(output.len());
            let heading = &output[heading_start ..];
            if looks_like_complete_bracket_heading(heading)
                && chars.peek().is_some_and(|next| *next != '\n')
            {
                output.push_str("\n\n");
            }
        }
    }
    output
}

fn looks_like_complete_bracket_heading(value: &str) -> bool {
    let value = value.trim();
    value.starts_with('【')
        && value.ends_with('】')
        && value.chars().count() <= 26
        && value.chars().any(is_cjk)
        && !value.contains("如")
        && !value.contains('/')
}

fn insert_breaks_around_inline_colon_headings(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return String::new();
    }

    let mut break_before = vec![false; chars.len()];
    let mut break_after = vec![false; chars.len()];
    for (index, ch) in chars.iter().enumerate() {
        if !is_heading_colon(*ch) {
            continue;
        }
        let Some(start) = inline_colon_heading_start(&chars, index) else {
            continue;
        };
        let heading = chars[start .. index].iter().collect::<String>();
        if !looks_like_inline_colon_heading(&heading) {
            continue;
        }
        if start > 0 && chars[start - 1] != '\n' {
            break_before[start] = true;
        }
        if chars.get(index + 1).is_some_and(|next| *next != '\n') {
            break_after[index] = true;
        }
    }

    let mut output = String::new();
    for (index, ch) in chars.into_iter().enumerate() {
        if break_before[index] {
            trim_trailing_spaces(&mut output);
            ensure_paragraph_break(&mut output);
        }
        output.push(ch);
        if break_after[index] {
            ensure_paragraph_break(&mut output);
        }
    }
    output
}

fn is_heading_colon(ch: char) -> bool {
    matches!(ch, ':' | '：')
}

fn inline_colon_heading_start(chars: &[char], colon_index: usize) -> Option<usize> {
    if colon_index == 0 {
        return None;
    }

    let mut start = colon_index;
    while start > 0 {
        let prev = chars[start - 1];
        if prev == '\n' || is_sentence_boundary(prev) {
            break;
        }
        start -= 1;
    }
    while start < colon_index && chars[start].is_whitespace() {
        start += 1;
    }
    (start < colon_index).then_some(start)
}

fn is_sentence_boundary(ch: char) -> bool {
    matches!(ch, '。' | '！' | '？' | '；' | ';' | '!' | '?')
}

fn looks_like_inline_colon_heading(heading: &str) -> bool {
    let heading = heading.trim();
    let char_count = heading.chars().count();
    (2 ..= 28).contains(&char_count)
        && heading.chars().any(is_cjk)
        && heading
            .chars()
            .all(|ch| is_heading_text_char(ch) || ch.is_whitespace())
        && !heading.contains("如")
        && !matches!(heading, "说明" | "资料边界" | "资料依据" | "依据说明")
        && !heading.starts_with("本轮已使用")
        && heading_has_section_suffix(heading)
}

fn is_heading_text_char(ch: char) -> bool {
    is_cjk(ch)
        || ch.is_ascii_alphanumeric()
        || matches!(ch, '%' | '％' | '、' | '·' | '-' | '_' | ' ')
}

fn heading_has_section_suffix(heading: &str) -> bool {
    let heading = heading_core_without_trailing_note(heading);
    [
        "结论",
        "判断",
        "逻辑",
        "分析",
        "说明",
        "依据",
        "动作",
        "建议",
        "边界",
        "风险",
        "问题",
        "清单",
        "计划",
        "练习",
        "训练",
        "实训",
        "复盘表",
        "执行表",
        "字段表",
        "方案",
        "策略",
        "步骤",
        "表格",
        "字段",
        "标题",
        "文案",
        "卖点",
        "事实",
        "重点",
        "环节",
        "部分",
        "方向",
        "角度",
        "资料",
        "数据",
        "原因",
        "限制",
        "核验",
        "下一步",
        "适合",
        "不适合",
    ]
    .iter()
    .any(|suffix| heading.ends_with(suffix))
}

fn heading_core_without_trailing_note(heading: &str) -> &str {
    let heading = heading.trim();
    for (open, close) in [('（', '）'), ('(', ')')] {
        if !heading.ends_with(close) {
            continue;
        }
        let Some(index) = heading.rfind(open) else {
            continue;
        };
        let core = heading[.. index].trim();
        if !core.is_empty() {
            return core;
        }
    }
    heading
}

fn split_inline_markdown_heading_content(text: &str) -> String {
    text.lines()
        .flat_map(split_inline_markdown_heading_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn split_inline_markdown_heading_line(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let Some(rest) = trimmed.strip_prefix('#') else {
        return vec![line.to_string()];
    };
    let hash_count = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1 ..= 6).contains(&hash_count) {
        return vec![line.to_string()];
    }
    let content = rest.trim_start_matches('#').trim_start();
    if content.is_empty() {
        return vec![line.to_string()];
    }
    let Some((heading, body)) = split_heading_content(content) else {
        return vec![line.to_string()];
    };
    vec![
        format!("{} {heading}", "#".repeat(hash_count)),
        String::new(),
        body.to_string(),
    ]
}

fn split_heading_content(content: &str) -> Option<(&str, &str)> {
    for (index, ch) in content.char_indices() {
        if !ch.is_whitespace() {
            continue;
        }
        let heading = content[.. index].trim();
        let body = content[index ..].trim();
        if body.is_empty() {
            return None;
        }
        if looks_like_markdown_heading_fragment(heading) {
            return Some((heading, body));
        }
    }
    None
}

fn looks_like_markdown_heading_fragment(heading: &str) -> bool {
    let char_count = heading.chars().count();
    (2 ..= 18).contains(&char_count)
        && heading.chars().any(is_cjk)
        && heading
            .chars()
            .all(|ch| is_heading_text_char(ch) || ch.is_whitespace())
        && heading_has_section_suffix(heading)
}

fn format_readable_answer_lists(text: &str) -> String {
    let text = merge_key_value_heading_lines(text);
    let mut output = Vec::new();
    let mut block = Vec::new();
    let mut section_context = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush_readable_list_block(&mut output, &mut block, &section_context);
            output.push(String::new());
            continue;
        }
        if is_markdown_table_line(trimmed) {
            flush_readable_list_block(&mut output, &mut block, &section_context);
            output.push(line.to_string());
            if !section_context.contains("表") {
                section_context = "表格补充".to_string();
            }
            continue;
        }
        if let Some(heading) = display_heading_text(trimmed) {
            flush_readable_list_block(&mut output, &mut block, &section_context);
            section_context = heading.to_string();
            output.push(line.to_string());
            continue;
        }
        if looks_like_list_intro_line(trimmed) {
            flush_readable_list_block(&mut output, &mut block, &section_context);
            section_context = trimmed.trim_end_matches(['：', ':']).to_string();
            output.push(line.to_string());
            continue;
        }
        block.push(line.to_string());
    }
    flush_readable_list_block(&mut output, &mut block, &section_context);
    output.join("\n")
}

fn merge_key_value_heading_lines(text: &str) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if let Some(heading) = single_line_colon_heading(line)
            && heading_should_remain_key_value(heading, &lines, index)
        {
            let mut next_index = index + 1;
            while next_index < lines.len() && lines[next_index].trim().is_empty() {
                next_index += 1;
            }
            if let Some(body) = lines.get(next_index).map(|value| value.trim())
                && !body.is_empty()
            {
                output.push(format!("{heading}：{body}"));
                index = next_index + 1;
                continue;
            }
        }
        output.push(lines[index].to_string());
        index += 1;
    }
    output.join("\n")
}

fn single_line_colon_heading(line: &str) -> Option<&str> {
    let heading = line.strip_suffix('：').or_else(|| line.strip_suffix(':'))?;
    looks_like_inline_colon_heading(heading).then_some(heading)
}

fn heading_should_remain_key_value(heading: &str, lines: &[&str], index: usize) -> bool {
    if !looks_like_business_key_value_label(heading) {
        return false;
    }
    previous_non_blank_line(lines, index)
        .is_some_and(|line| looks_like_business_key_value_line(line.trim()))
        || next_non_blank_after_heading_body(lines, index)
            .is_some_and(|line| looks_like_business_key_value_line(line.trim()))
}

fn previous_non_blank_line<'a>(lines: &'a [&str], index: usize) -> Option<&'a str> {
    lines[.. index]
        .iter()
        .rev()
        .copied()
        .find(|line| !line.trim().is_empty())
}

fn next_non_blank_after_heading_body<'a>(lines: &'a [&str], index: usize) -> Option<&'a str> {
    let mut seen_body = false;
    for line in lines.iter().skip(index + 1).copied() {
        if line.trim().is_empty() {
            continue;
        }
        if !seen_body {
            seen_body = true;
            continue;
        }
        return Some(line);
    }
    None
}

fn flush_readable_list_block(
    output: &mut Vec<String>,
    block: &mut Vec<String>,
    section_context: &str,
) {
    if block.is_empty() {
        return;
    }

    let eligible = block
        .iter()
        .map(|line| readable_list_item_kind(line.trim(), section_context))
        .collect::<Vec<_>>();
    let eligible_count = eligible
        .iter()
        .filter(|kind| !matches!(kind, ReadableListItemKind::None))
        .count();
    let key_value_count = eligible
        .iter()
        .filter(|kind| matches!(kind, ReadableListItemKind::KeyValue))
        .count();
    let short_plain_count = eligible
        .iter()
        .filter(|kind| matches!(kind, ReadableListItemKind::PlainShort))
        .count();
    let should_bulletize = eligible_count >= 2
        && (key_value_count >= 2
            || section_context_prefers_list(section_context)
            || short_plain_count >= 3);

    for (line, kind) in block.drain(..).zip(eligible) {
        if should_bulletize && !matches!(kind, ReadableListItemKind::None) {
            output.push(format!("- {}", line.trim()));
        } else {
            output.push(line);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadableListItemKind {
    None,
    KeyValue,
    PlainShort,
}

fn readable_list_item_kind(line: &str, section_context: &str) -> ReadableListItemKind {
    if line.is_empty()
        || line.starts_with('#')
        || line.starts_with('|')
        || line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("• ")
        || has_ordered_list_prefix(line)
        || looks_like_list_intro_line(line)
    {
        return ReadableListItemKind::None;
    }
    if looks_like_business_key_value_line(line) {
        return ReadableListItemKind::KeyValue;
    }
    if section_context_prefers_list(section_context) && looks_like_short_list_line(line) {
        return ReadableListItemKind::PlainShort;
    }
    ReadableListItemKind::None
}

fn has_ordered_list_prefix(line: &str) -> bool {
    let mut chars = line.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if matches!(first, '①' ..= '⑳') {
        return true;
    }
    first.is_ascii_digit()
        && chars
            .next()
            .is_some_and(|next| matches!(next, '.' | '、' | '．' | ')'))
}

fn looks_like_business_key_value_line(line: &str) -> bool {
    let Some((colon_index, colon_width)) = first_colon(line) else {
        return false;
    };
    let label = line[.. colon_index].trim();
    let body = line[colon_index + colon_width ..].trim();
    if label.is_empty() || body.is_empty() {
        return false;
    }
    looks_like_business_key_value_label(label)
}

fn looks_like_business_key_value_label(label: &str) -> bool {
    let label_chars = label.chars().count();
    (2 ..= 14).contains(&label_chars)
        && label.chars().any(is_cjk)
        && !matches!(
            label,
            "先给结论" | "说明" | "资料边界" | "资料依据" | "依据说明"
        )
}

fn looks_like_short_list_line(line: &str) -> bool {
    let char_count = line.chars().count();
    (6 ..= 56).contains(&char_count)
        && line.chars().any(is_cjk)
        && !line.contains("。")
        && !line.contains("；")
        && !line.contains(';')
}

fn section_context_prefers_list(section_context: &str) -> bool {
    [
        "动作",
        "下一步",
        "清单",
        "文案",
        "标题",
        "话术",
        "卖点",
        "表格",
        "字段",
        "指标",
        "事实",
        "依据",
        "建议",
        "风险",
        "复盘",
        "复盘表",
        "执行表",
        "字段表",
        "学生分工",
        "执行",
    ]
    .iter()
    .any(|marker| section_context.contains(marker))
}

fn looks_like_list_intro_line(line: &str) -> bool {
    if !(line.ends_with('：') || line.ends_with(':')) {
        return false;
    }
    section_context_prefers_list(line.trim_end_matches(['：', ':']))
}

fn display_heading_text(line: &str) -> Option<&str> {
    if line.starts_with('#') {
        let text = line.trim_start_matches('#').trim();
        return (!text.is_empty()).then_some(text);
    }
    if line.starts_with('【') && line.ends_with('】') {
        let text = line.trim_matches(['【', '】']);
        return (!text.is_empty()).then_some(text);
    }
    None
}

fn split_inline_markdown_tables(text: &str) -> String {
    text.lines()
        .flat_map(split_inline_markdown_table_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn split_inline_markdown_table_line(line: &str) -> Vec<String> {
    let Some(table_start) = find_inline_markdown_table_start(line) else {
        return vec![line.to_string()];
    };
    let prefix = line[.. table_start].trim();
    let table_text = line[table_start ..].trim();
    let rows = split_inline_table_rows(table_text);
    if !rows_have_markdown_table_shape(&rows) {
        return vec![line.to_string()];
    }

    let mut output = Vec::new();
    if !prefix.is_empty() {
        output.push(prefix.to_string());
    }
    output.extend(rows);
    output
}

fn find_inline_markdown_table_start(line: &str) -> Option<usize> {
    if !(line.contains("|---") || line.contains("| ---")) {
        return None;
    }
    line.char_indices().find_map(|(index, ch)| {
        (ch == '|' && line[index ..].matches('|').count() >= 6).then_some(index)
    })
}

fn split_inline_table_rows(table_text: &str) -> Vec<String> {
    let chars = table_text.chars().collect::<Vec<_>>();
    let mut rows = Vec::new();
    let mut current = String::new();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '|' && !current.is_empty() {
            let mut next = index + 1;
            while next < chars.len() && chars[next].is_whitespace() {
                next += 1;
            }
            if next < chars.len() && chars[next] == '|' && current.matches('|').count() >= 2 {
                current.push('|');
                rows.push(current.trim().to_string());
                current.clear();
                index = next;
                continue;
            }
        }
        current.push(ch);
        index += 1;
    }

    if !current.trim().is_empty() {
        rows.push(current.trim().to_string());
    }
    rows
}

fn rows_have_markdown_table_shape(rows: &[String]) -> bool {
    if rows.len() < 2 {
        return false;
    }
    rows.iter()
        .take(3)
        .any(|row| is_markdown_table_separator_line(row))
}

fn is_markdown_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim().trim_matches('|');
    let cells = trimmed.split('|').map(str::trim).collect::<Vec<_>>();
    cells.len() >= 2
        && cells
            .iter()
            .all(|cell| cell.len() >= 3 && cell.chars().all(|ch| matches!(ch, '-' | ':')))
}

fn is_markdown_table_line(line: &str) -> bool {
    line.starts_with('|') && line.ends_with('|')
}

fn repair_broken_markdown_table_rows(text: &str) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = Vec::with_capacity(lines.len());
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index].trim();
        if looks_like_incomplete_markdown_table_row(line) {
            let mut merged = line.to_string();
            let mut next_index = index + 1;
            let mut completed = false;
            while next_index < lines.len() {
                let next = lines[next_index].trim();
                if next.is_empty() {
                    next_index += 1;
                    continue;
                }
                if starts_new_markdown_table_row(next) {
                    break;
                }
                append_table_row_continuation(&mut merged, next);
                next_index += 1;
                if merged.ends_with('|') {
                    completed = true;
                    break;
                }
            }
            if completed {
                output.push(merged);
                index = next_index;
                continue;
            }
        }
        output.push(lines[index].to_string());
        index += 1;
    }

    output.join("\n")
}

fn looks_like_incomplete_markdown_table_row(line: &str) -> bool {
    line.starts_with('|') && !line.ends_with('|') && line.matches('|').count() >= 2
}

fn starts_new_markdown_table_row(line: &str) -> bool {
    line.starts_with('|') && line.matches('|').count() >= 2
}

fn append_table_row_continuation(row: &mut String, continuation: &str) {
    if !row.ends_with(char::is_whitespace) {
        row.push(' ');
    }
    row.push_str(continuation);
}

fn insert_breaks_before_inline_numbered_items(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    for (index, ch) in chars.iter().enumerate() {
        if should_break_before_inline_numbered_item(&chars, index) {
            trim_trailing_spaces(&mut output);
            ensure_paragraph_break(&mut output);
        }
        output.push(*ch);
    }
    output
}

fn promote_plain_section_headings(text: &str) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = Vec::with_capacity(lines.len());
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if looks_like_plain_section_heading(trimmed, lines.get(index + 1).copied()) {
            output.push(format!("## {trimmed}"));
        } else {
            output.push((*line).to_string());
        }
    }
    output.join("\n")
}

fn looks_like_plain_section_heading(line: &str, next_line: Option<&str>) -> bool {
    if line.is_empty()
        || line.starts_with('#')
        || line.starts_with('|')
        || line.starts_with('-')
        || line.starts_with('*')
        || line.ends_with('：')
        || line.ends_with(':')
        || line.contains('。')
        || line.contains('，')
        || line.contains(',')
        || line.contains(';')
        || line.contains('；')
        || line.contains('！')
        || line.contains('？')
        || line.contains('!')
        || line.contains('?')
        || looks_like_risk_statement_not_heading(line)
        || matches!(line, "资料边界" | "资料依据" | "依据说明")
    {
        return false;
    }
    let char_count = line.chars().count();
    (2 ..= 18).contains(&char_count)
        && line.chars().any(is_cjk)
        && heading_has_section_suffix(line)
        && next_line.is_some_and(|next| {
            let next = next.trim();
            !next.is_empty() && !next.starts_with('#')
        })
}

fn looks_like_risk_statement_not_heading(line: &str) -> bool {
    line.ends_with("风险") && (line.contains("可能") || line.contains("导致"))
}

fn should_break_before_inline_numbered_item(chars: &[char], index: usize) -> bool {
    if index == 0 || chars[index - 1] == '\n' {
        return false;
    }
    if index_is_inside_markdown_table_line(chars, index) {
        return false;
    }
    if !has_inline_numbered_item_marker(chars, index) {
        return false;
    }
    if !previous_context_allows_inline_numbered_break(chars, index) {
        return false;
    }
    following_text_looks_like_list_item(chars, index)
}

fn index_is_inside_markdown_table_line(chars: &[char], index: usize) -> bool {
    let line_start = chars[.. index]
        .iter()
        .rposition(|ch| *ch == '\n')
        .map_or(0, |position| position + 1);
    let mut first_non_space = line_start;
    while first_non_space < index && chars[first_non_space].is_whitespace() {
        first_non_space += 1;
    }
    first_non_space < index && chars[first_non_space] == '|'
}

fn has_inline_numbered_item_marker(chars: &[char], index: usize) -> bool {
    if matches!(chars[index], '①' ..= '⑳') {
        return true;
    }
    chars[index].is_ascii_digit()
        && chars
            .get(index + 1)
            .is_some_and(|next| matches!(*next, '.' | '、' | '．'))
        && chars
            .get(index + 2)
            .is_some_and(|next| next.is_whitespace() || is_cjk(*next))
}

fn previous_context_allows_inline_numbered_break(chars: &[char], index: usize) -> bool {
    let previous = chars[.. index]
        .iter()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .copied();
    matches!(
        previous,
        Some('：' | ':' | '；' | ';' | '。' | '！' | '？' | ',' | '，' | '、')
    )
}

fn following_text_looks_like_list_item(chars: &[char], index: usize) -> bool {
    let mut cursor = if matches!(chars[index], '①' ..= '⑳') {
        index + 1
    } else {
        index + 2
    };
    while cursor < chars.len() && chars[cursor].is_whitespace() {
        cursor += 1;
    }
    chars.get(cursor).is_some_and(|ch| is_cjk(*ch))
}

fn trim_trailing_spaces(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

fn ensure_paragraph_break(output: &mut String) {
    if output.is_empty() || output.ends_with("\n\n") {
        return;
    }
    if output.ends_with('\n') {
        output.push('\n');
    } else {
        output.push_str("\n\n");
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF)
}

fn rewrite_internal_evidence_terms(line: &str) -> String {
    let mut text = line.to_string();
    for phrase in [
        "商品锚点：",
        "商品锚点:",
        "资料锚点：",
        "资料锚点:",
        "QA文件锚点：",
        "QA文件锚点:",
        "文件锚点：",
        "文件锚点:",
    ] {
        text = text.replace(phrase, "");
    }
    text = text.replace("低置信参考", "仍需核验的参考");
    text = text.replace("低置信度", "仍需核验");
    text = text.replace("低置信", "仍需核验");
    text = text.replace("商品锚点", "商品资料");
    text = text.replace("资料锚点", "资料依据");
    text = text.replace("QA文件锚点", "资料依据");
    text = text.replace("文件锚点", "资料依据");
    text = rewrite_confidence_phrases(&text);
    text = text.replace("资料可信度较高", "资料来源较明确");
    text = text.replace("资料可信度", "资料来源");
    text = text.replace("资料来源较明确，资料来源较明确", "资料来源较明确");
    text = text.replace("资料来源较明确、资料来源较明确", "资料来源较明确");
    text = rewrite_internal_context_field_terms(&text);
    text
}

fn rewrite_misplaced_evidence_heading(line: &str) -> String {
    for marker in [
        "资料依据：## ",
        "资料依据:## ",
        "资料依据：# ",
        "资料依据:# ",
        "资料关键事实：## ",
        "资料关键事实:## ",
        "资料关键事实：# ",
        "资料关键事实:# ",
    ] {
        if let Some(rest) = line.strip_prefix(marker) {
            let heading_prefix = if marker.contains("##") { "## " } else { "# " };
            return format!("{heading_prefix}{}", rest.trim_start());
        }
    }
    line.to_string()
}

fn rewrite_internal_context_field_terms(line: &str) -> String {
    let mut text = line.to_string();
    for phrase in [
        "本轮未检测到 context.selected_summary；",
        "本轮未检测到 context.selected_summary;",
        "未检测到 context.selected_summary；",
        "未检测到 context.selected_summary;",
        "没有检测到 context.selected_summary；",
        "没有检测到 context.selected_summary;",
    ] {
        text = text.replace(phrase, "本轮没有手动选择具体商品；");
    }
    text = text.replace("来源 context.selected_summary", "来源为本轮选择的商品资料");
    text = text.replace(
        "source context.selected_summary",
        "source selected product context",
    );
    text = text.replace("context.selected_summary", "本轮选择的商品资料");
    text = text.replace("context.combined_summary", "本轮可用上下文");
    text = text.replace("workspaceContext", "工作区资料");
    text = text.replace("workspace_context", "工作区资料");
    text
}

fn rewrite_confidence_phrases(line: &str) -> String {
    let text = rewrite_chinese_confidence_phrases(line);
    rewrite_ascii_confidence_phrases(&text)
}

fn rewrite_chinese_confidence_phrases(line: &str) -> String {
    let mut output = String::new();
    let mut rest = line;
    while let Some(index) = rest.find("置信度") {
        output.push_str(&rest[.. index]);
        let after_marker = &rest[index + "置信度".len() ..];
        let consumed = trim_confidence_value(after_marker);
        if consumed == 0 {
            output.push_str("资料来源");
        } else {
            output.push_str("资料来源较明确");
        }
        rest = &after_marker[consumed ..];
    }
    output.push_str(rest);
    output
}

fn rewrite_ascii_confidence_phrases(line: &str) -> String {
    let mut output = String::new();
    let mut rest = line;
    loop {
        let lower = rest.to_ascii_lowercase();
        let Some(index) = lower.find("confidence") else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[.. index]);
        let after_marker = &rest[index + "confidence".len() ..];
        let consumed = trim_confidence_value(after_marker);
        if consumed == 0 {
            output.push_str("资料来源");
        } else {
            output.push_str("资料来源较明确");
        }
        rest = &after_marker[consumed ..];
    }
    output
}

fn trim_confidence_value(input: &str) -> usize {
    let mut consumed = 0;
    for ch in input.chars() {
        if ch.is_whitespace() || matches!(ch, '=' | '：' | ':' | '约' | '为') {
            consumed += ch.len_utf8();
            continue;
        }
        break;
    }

    let value_start = consumed;
    let mut has_digit = false;
    for ch in input[value_start ..].chars() {
        if ch.is_ascii_digit() || matches!(ch, '.' | '%' | '％') {
            has_digit = true;
            consumed += ch.len_utf8();
            continue;
        }
        break;
    }

    if has_digit {
        for ch in input[consumed ..].chars() {
            if ch.is_whitespace() {
                consumed += ch.len_utf8();
                continue;
            }
            break;
        }
        consumed
    } else {
        0
    }
}

fn rewrite_selected_product_source_line(line: &str) -> String {
    let mut marker_match = None;
    for marker in [
        "用户本轮手动选择的预设商品资料：",
        "用户本轮手动选择的商品资料：",
    ] {
        if let Some(index) = line.find(marker) {
            marker_match = Some((index, marker));
            break;
        }
    }
    let Some((index, marker)) = marker_match else {
        return line.to_string();
    };

    let mut product = line[index + marker.len() ..].trim().to_string();
    for delimiter in ["（", "(", "，来源", ", source", "来源"] {
        if let Some(delimiter_index) = product.find(delimiter) {
            product.truncate(delimiter_index);
        }
    }
    let product = product.trim().trim_matches(['。', '.', '：', ':']).trim();
    if product.is_empty() {
        "本轮已使用你选中的商品资料。".to_string()
    } else {
        format!("本轮已使用你选中的商品资料：{product}。")
    }
}

fn strip_role_prefix(line: &str) -> String {
    let (prefix, body) = split_list_prefix(line);
    let Some((colon_index, colon_width)) = first_colon(body) else {
        return line.trim().to_string();
    };
    let label = body[.. colon_index].trim();
    if !looks_like_internal_role_label(label) {
        return line.trim().to_string();
    }

    format!(
        "{prefix}{}",
        body[colon_index + colon_width ..].trim_start()
    )
    .trim()
    .to_string()
}

fn split_list_prefix(line: &str) -> (&str, &str) {
    for prefix in ["- ", "* ", "• "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return (prefix, rest);
        }
    }
    ("", line)
}

fn first_colon(text: &str) -> Option<(usize, usize)> {
    text.char_indices()
        .find(|(_, ch)| matches!(ch, ':' | '：'))
        .map(|(index, ch)| (index, ch.len_utf8()))
}

fn looks_like_internal_role_label(label: &str) -> bool {
    let lower = label.to_ascii_lowercase();
    if is_internal_runtime_role_label(&lower) {
        return true;
    }
    if lower.starts_with("primary ")
        || lower.starts_with("support ")
        || lower.starts_with("worker.")
        || lower.starts_with("role ")
    {
        return true;
    }

    let Some(start) = label.find('(') else {
        return false;
    };
    let Some(end) = label[start + 1 ..].find(')') else {
        return false;
    };
    let role = &label[start + 1 .. start + 1 + end];
    !role.is_empty()
        && role.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '.')
        })
}

fn is_internal_runtime_role_label(label: &str) -> bool {
    let label = label.trim().trim_end_matches("_final");
    matches!(
        label,
        "chat"
            | "ops"
            | "data"
            | "creative"
            | "service"
            | "engineering"
            | "accounting"
            | "design"
            | "web"
            | "search"
            | "douyin_ads"
    )
}

fn compact_line_spacing(line: &str) -> String {
    let mut line = line
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    for phrase in [" ，", " 。", " ：", " ；", " 、", " ！", " ？"] {
        let replacement = phrase.trim_start();
        line = line.replace(phrase, replacement);
    }
    line
}

fn normalize_blank_lines(lines: Vec<String>) -> String {
    let mut normalized = Vec::new();
    let mut last_blank = true;

    for line in lines {
        if line.trim().is_empty() {
            if !last_blank {
                normalized.push(String::new());
                last_blank = true;
            }
            continue;
        }
        normalized.push(line.trim().to_string());
        last_blank = false;
    }

    while normalized.last().is_some_and(|line| line.is_empty()) {
        normalized.pop();
    }

    let normalized = collapse_repeated_blocks(normalized);
    normalized.join("\n")
}

fn drop_orphan_section_headings(lines: Vec<String>) -> Vec<String> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            if is_generated_section_heading(line)
                && lines
                    .get(index + 1)
                    .is_none_or(|next| is_generated_section_heading(next))
            {
                return None;
            }
            Some(line.clone())
        })
        .collect()
}

fn drop_historical_evidence_quote_blocks(lines: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if is_generated_material_heading(line)
            && next_non_blank_line_from(&lines, index + 1)
                .is_some_and(looks_like_material_evidence_replay_line)
        {
            index += 1;
            while index < lines.len() && lines[index].trim().is_empty() {
                index += 1;
            }
            while index < lines.len() && !lines[index].trim().is_empty() {
                index += 1;
            }
            continue;
        }
        output.push(lines[index].clone());
        index += 1;
    }
    output
}

fn truncate_tail_historical_answer_quotes(lines: Vec<String>) -> Vec<String> {
    for index in 0 .. lines.len() {
        if !has_substantial_answer_before(&lines[.. index]) {
            continue;
        }
        if looks_like_tail_historical_answer_quote_start(&lines, index) {
            return lines[.. index].to_vec();
        }
    }
    lines
}

fn has_substantial_answer_before(lines: &[String]) -> bool {
    let chars = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().count())
        .sum::<usize>();
    let non_blank = lines.iter().filter(|line| !line.trim().is_empty()).count();
    chars >= 240 && non_blank >= 5
}

fn looks_like_tail_historical_answer_quote_start(lines: &[String], index: usize) -> bool {
    let line = lines[index].trim();
    if is_generated_material_heading(line)
        && next_non_blank_line_from(lines, index + 1)
            .is_some_and(looks_like_material_evidence_replay_line)
    {
        return true;
    }
    if looks_like_historical_answer_quote_line(line) {
        return true;
    }
    if line == "## 判断依据" || line == "判断依据" {
        return upcoming_lines_look_like_previous_judgment(lines, index + 1);
    }
    false
}

fn upcoming_lines_look_like_previous_judgment(lines: &[String], start: usize) -> bool {
    let joined = lines
        .iter()
        .skip(start)
        .take(8)
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ");
    let judgment_markers = [
        "精准踩中刚性需求",
        "卖点真实无需编造",
        "合规风险低",
        "大学生开学最急需",
        "新手不容易踩规则红线",
    ];
    judgment_markers
        .iter()
        .filter(|marker| joined.contains(*marker))
        .count()
        >= 2
}

fn next_non_blank_line_from(lines: &[String], start: usize) -> Option<&str> {
    lines
        .iter()
        .skip(start)
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
}

fn is_generated_material_heading(line: &str) -> bool {
    matches!(
        line.trim(),
        "资料依据:"
            | "资料依据："
            | "资料关键事实:"
            | "资料关键事实："
            | "资料摘要:"
            | "资料摘要："
    )
}

fn looks_like_historical_answer_quote_line(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() {
        return false;
    }
    let lower = line.to_ascii_lowercase();
    line.starts_with("文案：")
        || line.starts_with("1. 文案：")
        || line.starts_with("2. 文案：")
        || line.starts_with("3. 文案：")
        || line.starts_with("1、文案：")
        || line.starts_with("2、文案：")
        || line.starts_with("3、文案：")
        || (line.contains("文案：") && line.contains("画面：") && line.contains("转化意图"))
        || lower.starts_with("user:")
        || lower.starts_with("assistant:")
}

fn looks_like_material_evidence_replay_line(line: &str) -> bool {
    looks_like_historical_answer_quote_line(line)
        || looks_like_material_evidence_history_replay_line(line)
}

fn looks_like_material_evidence_history_replay_line(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() {
        return false;
    }
    line.starts_with("运营对象&平台")
        || line.starts_with("素材&页面落地规则")
        || line.starts_with("主图要求")
        || (line.contains("复盘字段")
            && (line.contains("活动期间统计")
                || line.contains("咨询top问题")
                || line.contains("库存剩余量")
                || line.contains("成交转化率")))
        || (line.contains("主打卖点组合")
            && (line.contains("版本可核对")
                || line.contains("48小时发出")
                || line.contains("开学公共课教材")))
}

fn is_generated_section_heading(line: &str) -> bool {
    matches!(
        line.trim(),
        "发现:"
            | "发现："
            | "建议:"
            | "建议："
            | "依据:"
            | "依据："
            | "风险:"
            | "风险："
            | "待确认:"
            | "待确认："
            | "资料依据:"
            | "资料依据："
            | "资料关键事实:"
            | "资料关键事实："
            | "资料摘要:"
            | "资料摘要："
    )
}

fn collapse_repeated_blocks(lines: Vec<String>) -> Vec<String> {
    let lines = collapse_repeated_line_sequence(lines);
    let lines = collapse_blank_line_runs(collapse_non_adjacent_repeated_line_runs(lines));
    let mut blocks = lines_to_blocks(lines);
    if blocks.is_empty() {
        return Vec::new();
    }

    blocks = collapse_adjacent_duplicate_blocks(blocks);
    blocks = collapse_duplicate_blocks(blocks);
    if blocks.len().is_multiple_of(2) {
        let half = blocks.len() / 2;
        if blocks[.. half] == blocks[half ..] {
            blocks.truncate(half);
        }
    }

    blocks_to_lines(blocks)
}

fn collapse_blank_line_runs(lines: Vec<String>) -> Vec<String> {
    lines.into_iter().fold(Vec::new(), |mut output, line| {
        if line.trim().is_empty() && output.last().is_none_or(|last| last.trim().is_empty()) {
            return output;
        }
        output.push(line);
        output
    })
}

fn collapse_non_adjacent_repeated_line_runs(lines: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    let mut cursor = 0;
    while cursor < lines.len() {
        if let Some(run_len) = longest_seen_line_run(&output, &lines[cursor ..]) {
            cursor += run_len;
            continue;
        }
        output.push(lines[cursor].clone());
        cursor += 1;
    }
    output
}

fn longest_seen_line_run(seen: &[String], remaining: &[String]) -> Option<usize> {
    let max_len = remaining.len().min(seen.len());
    (2 ..= max_len).rev().find(|run_len| {
        seen.windows(*run_len)
            .any(|window| window == &remaining[.. *run_len])
    })
}

fn collapse_repeated_line_sequence(mut lines: Vec<String>) -> Vec<String> {
    loop {
        let len = lines.len();
        if len <= 1 || !len.is_multiple_of(2) || lines[.. len / 2] != lines[len / 2 ..] {
            break;
        }
        lines.truncate(len / 2);
    }

    lines.into_iter().fold(Vec::new(), |mut unique, line| {
        if unique.last().is_none_or(|existing| existing != &line) {
            unique.push(line);
        }
        unique
    })
}

fn lines_to_blocks(lines: Vec<String>) -> Vec<Vec<String>> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(current);
                current = Vec::new();
            }
            continue;
        }
        current.push(line);
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

fn collapse_adjacent_duplicate_blocks(blocks: Vec<Vec<String>>) -> Vec<Vec<String>> {
    blocks.into_iter().fold(Vec::new(), |mut unique, block| {
        if unique.last().is_none_or(|existing| existing != &block) {
            unique.push(block);
        }
        unique
    })
}

fn collapse_duplicate_blocks(blocks: Vec<Vec<String>>) -> Vec<Vec<String>> {
    blocks.into_iter().fold(Vec::new(), |mut unique, block| {
        if !unique.iter().any(|existing| existing == &block) {
            unique.push(block);
        }
        unique
    })
}

fn blocks_to_lines(blocks: Vec<Vec<String>>) -> Vec<String> {
    let mut lines = Vec::new();
    for block in blocks {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(block);
    }
    lines
}

fn is_internal_noise_line(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() {
        return true;
    }
    let lower = line.to_ascii_lowercase();
    let policy = user_facing_policy();

    policy
        .internal_noise_lower_markers
        .iter()
        .any(|marker| lower.contains(marker))
        || policy
            .internal_noise_markers
            .iter()
            .any(|marker| line.contains(marker))
        || looks_like_task_objective_shell(line)
        || line.starts_with("任务已完成")
        || line.starts_with("支持角色已完成")
        || line.contains("正在思考中")
        || line.contains("用户未提供具体问题")
        || line.contains("占位符")
        || lower.contains(" worker.")
        || lower.contains(" worker:")
        || lower.contains(" worker_id")
        || lower.contains(" worker report")
        || lower.contains("search_evidence_packet")
        || lower.contains("search_evidence_trace")
        || lower.starts_with("skill:")
        || (lower.contains(" status=") && lower.contains("skill"))
        || lower.starts_with("context.selected_summary")
        || lower.contains(" context.selected_summary=")
        || (lower.contains("product:") && contains_uuid_like(line))
        || (lower.contains(" | name=") && lower.contains(" | platform="))
        || (lower.contains(" title=") && lower.contains('|'))
        || (lower.contains(" product=") && lower.contains('|'))
        || looks_like_history_as_material_evidence(line)
        || looks_like_prompt_instruction_leak(line)
        || lower.starts_with("worker")
        || line == "- none"
        || lower == "none"
        || is_empty_list_artifact(line)
}

fn looks_like_memory_context_leak(lower: &str) -> bool {
    user_facing_policy()
        .memory_context_leak_markers
        .iter()
        .any(|marker| lower.contains(marker))
}

fn looks_like_task_objective_shell(line: &str) -> bool {
    let line = line.trim();
    line.chars().count() <= 80
        && line.contains('整')
        && line.contains('合')
        && line.contains("结果")
        && (line.contains("形成可执行") || line.contains("形成可提交"))
        && (line.contains("方案") || line.contains("运营"))
}

fn looks_like_history_as_material_evidence(line: &str) -> bool {
    let line = line.trim();
    if !(line.starts_with("资料依据：")
        || line.starts_with("资料关键事实：")
        || line.starts_with("资料摘要："))
    {
        return false;
    }
    let body = line
        .trim_start_matches("资料依据：")
        .trim_start_matches("资料关键事实：")
        .trim_start_matches("资料摘要：")
        .trim();
    body.starts_with('-')
        || body.starts_with("请")
        || body.starts_with("继续")
        || body.starts_with("新话题")
        || body.starts_with("运营对象&平台")
        || body.starts_with("判断依据：")
        || body.starts_with("【判断依据】")
        || looks_like_memory_context_leak(&body.to_ascii_lowercase())
        || body.contains("上一轮")
        || body.contains("这个商品")
        || body.contains("同一个商品")
        || body.contains("618返场")
        || body.contains("回答给新手学生看")
        || body.contains("表格要包含动作")
        || body.contains("素材&页面落地规则")
        || looks_like_historical_product_material(body)
        || body.to_ascii_lowercase().contains("user:")
        || body.to_ascii_lowercase().contains("assistant:")
}

fn looks_like_prompt_instruction_leak(line: &str) -> bool {
    user_facing_policy()
        .prompt_instruction_leak_markers
        .iter()
        .any(|marker| line.contains(marker))
}

fn looks_like_historical_product_material(body: &str) -> bool {
    let product_markers = ["商品资料", "活动", "客群", "低价", "价格", "卖点"];
    let metric_markers = ["CTR", "CVR", "ROAS", "退款率", "点击率", "转化率", "投产比"];
    product_markers
        .iter()
        .filter(|marker| body.contains(*marker))
        .count()
        >= 2
        && !metric_markers.iter().any(|marker| body.contains(*marker))
}

fn contains_uuid_like(line: &str) -> bool {
    let mut index = 0;
    while index + 36 <= line.len() {
        if line.is_char_boundary(index)
            && line.is_char_boundary(index + 36)
            && is_uuid_like(&line[index .. index + 36])
        {
            return true;
        }
        let Some(ch) = line[index ..].chars().next() else {
            break;
        };
        index += ch.len_utf8();
    }
    false
}

fn is_empty_list_artifact(line: &str) -> bool {
    matches!(
        line.trim(),
        "-" | "*" | "•" | "-：" | "*：" | "•：" | "-:" | "*:" | "•:"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        sanitize_final_user_facing_text, sanitize_user_facing_text, user_facing_report_text,
    };
    use crate::core::protocol::{
        RoleWorkOutput, TaskGraphId, TaskId, TaskNodeId, WorkerId, WorkerReport, WorkerReportStatus,
    };

    #[test]
    fn sanitizes_placeholder_and_note_language() {
        let raw = "内容创意(creative): \
                   【可直接测试的方案（占位符需替换为真实信息）】建议突出【真实品类，如挂耳/速溶/\
                   胶囊】和【真实规格/到手价】。\n备注：这是教学模式说明。";

        let output = sanitize_user_facing_text(raw).expect("output should remain readable");

        assert!(output.contains("建议突出真实品类和真实规格、到手价。"));
        assert!(output.contains("说明：这是教学模式说明。"));
        assert!(!output.contains("内容创意(creative)"));
        assert!(!output.contains("占位符"));
        assert!(!output.contains('【'));
        assert!(!output.contains('】'));
        assert!(!output.contains("备注："));
    }

    #[test]
    fn removes_user_answer_placeholder_shell() {
        let raw = "给用户看的完整中文回复：\n\n## 结论\n\n这个商品先按学生开学场景做小范围验证。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("## 结论"));
        assert!(output.contains("学生开学场景"));
        assert!(!output.contains("给用户看的完整中文回复"));
    }

    #[test]
    fn filters_internal_task_graph_lines() {
        let raw = "Task graph completed.\nPassed worker reports:\n- creative(worker.creative): \
                   raw={\"role_output\":{}}\n真正可用的建议：先补商品信息。";

        let output = sanitize_user_facing_text(raw).expect("valid line should remain");
        let lower = output.to_ascii_lowercase();

        assert_eq!(output, "真正可用的建议：先补商品信息。");
        assert!(!lower.contains("task graph"));
        assert!(!lower.contains("worker"));
        assert!(!lower.contains("role_output"));
        assert!(!lower.contains("raw="));
    }

    #[test]
    fn failed_report_surfaces_llm_access_denied_user_message() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-llm".to_string()),
            node_id: TaskNodeId("data".to_string()),
            task_id: TaskId("task-data".to_string()),
            worker_id: WorkerId("worker.data".to_string()),
            role: "data".to_string(),
            content: "llm_access_denied: 当前 AI 模型服务配置不可用".to_string(),
            role_output: None,
            evidence: vec!["failure_kind:llm_access_denied".to_string()],
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Failed,
        };

        let output = user_facing_report_text(&report).expect("diagnostic should be visible");

        assert!(output.contains("模型服务配置不可用"));
        assert!(!output.contains("services.toml"));
        assert!(!output.contains("[agent_llm_usage]"));
        assert!(!output.contains("API Key"));
        assert!(!output.contains("api_key"));
        assert!(!output.contains("Request id"));
    }

    #[test]
    fn removes_empty_role_shell_and_worker_artifacts() {
        let raw = "Task graph completed. Task: 这个咖啡如何做营销\nPassed worker reports\n- \
                   内容创意(creative): \
                   【可直接测试的千乐咖啡营销创意（占位符需替换为真实信息）】\n1. \
                   主推标题：千乐咖啡【真实品类（如挂耳/速溶/胶囊/咖啡豆）】日常口粮款\n备注：\
                   替换真实品类后即可上线，测试核心指标参考点击率";

        let output = sanitize_user_facing_text(raw).expect("business content should remain");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("千乐咖啡真实品类日常口粮款"));
        assert!(output.contains("说明：替换真实品类后即可上线"));
        assert!(!lower.contains("task graph"));
        assert!(!lower.contains("passed worker"));
        assert!(!lower.contains("worker"));
        assert!(!output.contains("内容创意(creative)"));
        assert!(!output.contains("占位符"));
        assert!(!output.contains("备注："));
        assert!(!output.contains('【'));
        assert!(!output.contains('】'));
    }

    #[test]
    fn includes_live_search_evidence_in_user_facing_text() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-search".to_string()),
            node_id: TaskNodeId("search".to_string()),
            task_id: TaskId("task-search".to_string()),
            worker_id: WorkerId("worker.search".to_string()),
            role: "search".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "本轮未拿到实时来源，不能直接给出对应规则明细。".to_string(),
                evidence: vec![
                    "实时搜索来源（search_platform_policy）：1 个可采信来源\n\
                     1. 官方公告 | 2026-05-20 | https://example.com/policy | 风险点：价格合规"
                        .to_string(),
                ],
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let output = user_facing_report_text(&report).expect("live evidence should be visible");

        assert!(output.contains("本轮已通过外部搜索取得实时来源"));
        assert!(output.contains("实时搜索来源"));
        assert!(output.contains("https://example.com/policy"));
        assert!(!output.contains("本轮未拿到"));
    }

    #[test]
    fn filters_internal_search_evidence_trace_and_packet_from_user_answer() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-search".to_string()),
            node_id: TaskNodeId("search".to_string()),
            task_id: TaskId("task-search".to_string()),
            worker_id: WorkerId("worker.search".to_string()),
            role: "search".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "本轮已核对竞品价格，建议先用 12-19 元价格带做小样本测试。".to_string(),
                evidence: vec![
                    "实时搜索来源（search_competitor）：1 个可采信来源\n\
                     1. 冷萃咖啡竞品价格参考 | 2026-05-20 | \
                     https://example.com/cold-brew-price | 竞品价格集中在 12-19 元。"
                        .to_string(),
                    "search_evidence_packet:{\"sources\":[{\"url\":\"https://example.com/raw\"}]}"
                        .to_string(),
                    "search_evidence_trace: skill=search_competitor status=success \
                     adapter_status=live intent=competitor_price_or_benchmark sources=1 \
                     usable_claims=1 gaps=0 confidence=0.72 query=\"冷萃咖啡 竞品价格\""
                        .to_string(),
                ],
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let output = user_facing_report_text(&report).expect("answer should remain readable");

        assert!(output.contains("本轮已核对竞品价格"));
        assert!(output.contains("实时搜索来源"));
        assert!(output.contains("https://example.com/cold-brew-price"));
        assert!(!output.contains("search_evidence_packet"));
        assert!(!output.contains("search_evidence_trace"));
        assert!(!output.contains("https://example.com/raw"));
    }

    #[test]
    fn degraded_search_packet_does_not_claim_live_sources() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-search".to_string()),
            node_id: TaskNodeId("search".to_string()),
            task_id: TaskId("task-search".to_string()),
            worker_id: WorkerId("worker.search".to_string()),
            role: "search".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "本轮已通过外部搜索取得实时来源。\n继续按星河杯方案，标题强调温显、防漏和礼盒感。"
                    .to_string(),
                evidence: vec![
                    "search_evidence_packet:{\"adapter_status\":\"degraded\",\"source_state\":\"no_reliable_sources\",\"sources\":[],\"gaps\":[\"external search provider request failed: http://1.85.61.130:29003/api/external/search?q=test\"]}"
                        .to_string(),
                    "search_evidence_trace: skill=search_market_info status=deferred adapter_status=degraded sources=0 usable_claims=0 gaps=2 confidence=0.00"
                        .to_string(),
                ],
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let output = user_facing_report_text(&report).expect("business answer should remain");

        assert!(output.contains("继续按星河杯方案"));
        assert!(!output.contains("本轮已通过外部搜索取得实时来源"));
        assert!(!output.contains("实时搜索来源"));
        assert!(!output.contains("search_evidence_packet"));
        assert!(!output.contains("search_evidence_trace"));
        assert!(!output.contains("adapter_status"));
        assert!(!output.contains("1.85.61.130"));
    }

    #[test]
    fn drops_empty_evidence_heading_after_internal_trace_filtering() {
        let raw = "建议先检查竞品价格。\n依据:\n- search_evidence_trace: skill=search_competitor \
                   status=success adapter_status=live";

        let output = sanitize_user_facing_text(raw).expect("business line should remain");

        assert_eq!(output, "建议先检查竞品价格。");
    }

    #[test]
    fn non_search_role_keeps_business_answer_before_live_search_evidence() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-final".to_string()),
            node_id: TaskNodeId("ops_final".to_string()),
            task_id: TaskId("task-final".to_string()),
            worker_id: WorkerId("worker.ops".to_string()),
            role: "ops".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "建议先用校园内小范围试卖验证冷萃咖啡价格带，再按毛利和复购反馈调整。"
                    .to_string(),
                evidence: vec![
                    "实时搜索来源（search_competitor）：1 个可采信来源\n\
                     1. 竞品价格参考 | 2026-05-20 | https://example.com/price | 价格区间"
                        .to_string(),
                ],
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let output = user_facing_report_text(&report).expect("business answer should be visible");

        assert!(output.starts_with("建议先用校园内小范围试卖"));
        assert!(output.contains("实时搜索来源"));
        assert!(output.contains("https://example.com/price"));
    }

    #[test]
    fn non_search_role_keeps_material_evidence_when_summary_is_too_generic() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-material".to_string()),
            node_id: TaskNodeId("data_final".to_string()),
            task_id: TaskId("task-material".to_string()),
            worker_id: WorkerId("worker.data".to_string()),
            role: "data".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "根据选中文件的广告数据，当前最大问题是退款率偏高，并非点击率不足。"
                    .to_string(),
                evidence: vec![
                    "资料依据：广告数据里点击率 CTR=1.2%、转化率 \
                     CVR=2.1%、退款率=18%，最大问题是退款率偏高，不是点击率不足。"
                        .to_string(),
                    "skill:material_file_understanding status=success summary=已形成资料理解。"
                        .to_string(),
                ],
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let output = user_facing_report_text(&report).expect("answer should include evidence");

        assert!(output.contains("当前最大问题是退款率偏高"));
        assert!(output.contains("资料依据"));
        assert!(output.contains("退款率=18%"));
        assert!(output.contains("点击率 CTR=1.2%"));
        assert!(output.contains("点击率不是主要矛盾"));
        assert!(!output.contains("点击率不足"));
        assert!(!output.contains("skill:"));
        assert!(!output.contains("material_file_understanding"));
    }

    #[test]
    fn role_output_keeps_findings_recommendations_and_risks_in_user_answer() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-rich-role-output".to_string()),
            node_id: TaskNodeId("ops_final".to_string()),
            task_id: TaskId("task-rich-role-output".to_string()),
            worker_id: WorkerId("worker.ops".to_string()),
            role: "ops".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "已经稳定：销量、转化率、用户评价这三个观察指标已经明确。".to_string(),
                findings: vec![
                    "仍有风险：真实曝光、点击、成交和退款数据还不足，不能直接验证优化效果。"
                        .to_string(),
                ],
                recommendations: vec![
                    "学生下一步：先补齐一周样本数据，再用两个版本主图做小测试。".to_string(),
                ],
                risks: vec!["注意不要把没有核验过的网络资料当成确定结论。".to_string()],
                open_questions: vec!["还需要确认活动期间的目标 ROAS 和可接受退款率。".to_string()],
                evidence: Vec::new(),
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let output = user_facing_report_text(&report).expect("rich role output should be visible");

        assert!(output.contains("已经稳定：销量、转化率、用户评价这三个观察指标已经明确。"));
        assert!(output.contains("仍有风险：真实曝光、点击、成交和退款数据还不足"));
        assert!(output.contains("学生下一步：先补齐一周样本数据"));
        assert!(output.contains("注意不要把没有核验过的网络资料当成确定结论。"));
        assert!(output.contains("还需要确认活动期间的目标 ROAS 和可接受退款率。"));
        assert!(!output.contains("role_output"));
        assert!(!output.contains("worker."));
    }

    #[test]
    fn material_boundary_filters_blocking_wait_and_unable_lines() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-material-boundary".to_string()),
            node_id: TaskNodeId("ops_final".to_string()),
            task_id: TaskId("task-material-boundary".to_string()),
            worker_id: WorkerId("worker.ops".to_string()),
            role: "ops".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "先给学生书包开学季满减的可执行分析：先列字段，再看点击、加购、\
                          付款和退款。"
                    .to_string(),
                findings: vec![
                    "未提供真实曝光、点击、加购、付款等指标时，不能编造转化率，\
                     但可以先给建表字段和判断方法。"
                        .to_string(),
                ],
                recommendations: vec![
                    "等待用户提供附件或表格数据。".to_string(),
                    "先建一张基础表：日期、商品/SKU、渠道、曝光、点击、加购、下单、付款、GMV、\
                     退款、库存、客服咨询。"
                        .to_string(),
                ],
                risks: vec![
                    "无法进行字段、粒度、主键、指标口径、样本边界和缺失数据的定义。".to_string(),
                    "资料边界：表格资料分析没有产出可引用资料；\
                     本轮没有收到可解析的表格行或表格文本。\
                     以下业务判断只能先按用户文字和已进入上下文的信息给出，等补齐资料后再复核。"
                        .to_string(),
                ],
                open_questions: vec!["是否提供了附件或表格数据？".to_string()],
                evidence: Vec::new(),
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let output = user_facing_report_text(&report).expect("answer should remain");

        assert!(output.contains("学生书包开学季满减"));
        assert!(output.contains("先建一张基础表"));
        assert!(output.contains("资料边界"));
        assert!(!output.contains("等待用户提供"));
        assert!(!output.contains("无法进行字段"));
        assert!(!output.contains("是否提供了附件"));
    }

    #[test]
    fn material_evidence_suppresses_irrelevant_no_live_source_fallback() {
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-material".to_string()),
            node_id: TaskNodeId("data_final".to_string()),
            task_id: TaskId("task-material".to_string()),
            worker_id: WorkerId("worker.data".to_string()),
            role: "data".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "本轮没有可引用的实时外部来源，所以先不给平均数、排名或确定性对标。"
                    .to_string(),
                evidence: vec![
                    "资料依据：广告数据里点击率 CTR=1.2%、转化率 \
                     CVR=2.1%、退款率=18%，最大问题是退款率偏高。"
                        .to_string(),
                ],
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let output = user_facing_report_text(&report).expect("material evidence should answer");

        assert!(output.contains("资料依据"));
        assert!(output.contains("退款率=18%"));
        assert!(!output.contains("实时外部来源"));
        assert!(!output.contains("平均数、排名"));
    }

    #[test]
    fn sanitizing_collapses_repeated_answer_blocks() {
        let raw =
            "先看毛利，再看销量。\n- 补齐成本字段。\n\n先看毛利，再看销量。\n- 补齐成本字段。";

        let output = sanitize_user_facing_text(raw).expect("deduped output should remain");

        assert_eq!(output, "先看毛利，再看销量。\n- 补齐成本字段。");
    }

    #[test]
    fn final_sanitizing_collapses_non_adjacent_duplicate_blocks() {
        let raw = "先看毛利，再看销量。\n- \
                   补齐成本字段。\n\n资料边界：附件成本字段仍需核验。\n\n先看毛利，再看销量。\n- \
                   补齐成本字段。";

        let output = sanitize_final_user_facing_text(raw).expect("deduped output should remain");
        assert_eq!(
            output,
            "先看毛利，再看销量。\n- 补齐成本字段。\n\n资料边界：附件成本字段仍需核验。"
        );
    }

    #[test]
    fn filters_internal_context_prompt_leakage_from_final_answer() {
        let raw = "针对千乐咖啡，当前只确认了名称、售价和在售状态，缺少品类、平台和成本数据。\\
                   nCommerce product context resolved from current user's backend data. Use these \
                   facts when relevant, cite product or knowledge titles, and do not invent \
                   missing product details. Product source \
                   product:ce3f729c-a8bd-45ca-9859-7f2591d5973f | name=千乐咖啡 | \
                   platform=unknown | category=unknown | brand=unknown | price=100 CNY | \
                   status=active | description=适合于白领 Requested context refs: \
                   product:ce3f729c-a8bd-45ca-9859-7f2591d5973f\n建议先补真实成本、\
                   目标平台和用户反馈，再判断是否继续做活动。";

        let output = sanitize_final_user_facing_text(raw).expect("business content should remain");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("针对千乐咖啡"));
        assert!(output.contains("建议先补真实成本"));
        assert!(!lower.contains("commerce product context"));
        assert!(!lower.contains("product source product"));
        assert!(!lower.contains("requested context refs"));
        assert!(!lower.contains("platform=unknown"));
        assert!(!lower.contains("product:"));
        assert!(!output.contains("ce3f729c-a8bd-45ca-9859-7f2591d5973f"));
    }

    #[test]
    fn filters_inline_context_fields_without_destroying_answer_layout() {
        let raw = "本轮已通过外部搜索取得实时来源。\n当前岗位任务：整合已选商品 已选商品 \
                   title=QA矩阵智能保温杯 | product_knowledge:已选商品 title=QA矩阵商品知识 | \
                   confidence=0.96\n先给结论：这款商品应该主打温显、防漏和礼盒感，\
                   不要做学生低价定位。\n\n建议动作：\n- 主图第一屏放温显和防漏场景。\n- \
                   活动页强调三件套礼盒。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("先给结论："));
        assert!(output.contains("这款商品应该主打温显、防漏和礼盒感"));
        assert!(output.contains("建议动作：\n- 主图第一屏"));
        assert!(!output.contains("当前岗位任务"));
        assert!(!output.contains("title="));
        assert!(!lower.contains("product_knowledge"));
        assert!(!lower.contains("confidence"));
    }

    #[test]
    fn filters_conversation_history_wrappers_from_final_answer() {
        let raw = "给新手学生的三条618返场短视频开头文案：\n\n1. \
                   办公室姐妹看过来，温显防漏保温杯返场了。\n\nUser: 请分析这个商品。\nAssistant: \
                   这个商品在618返场活动里最应该主打温显、防漏、礼盒。\n当前用户请求：\\\\
                   \
                   nn继续同一个商品和活动，给我三条短视频开头文案。\n\n基于上游数据、设计、\
                   预算或创意结果，形成可执行运营调整方案。\n已选商品 \
                   已选商品\n落地注意：每天看退款率，超过8%就调整素材。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("办公室姐妹看过来"));
        assert!(output.contains("落地注意"));
        assert!(!lower.contains("user:"));
        assert!(!lower.contains("assistant:"));
        assert!(!output.contains("当前用户请求"));
        assert!(!output.contains("基于上游数据、设计、预算或创意结果"));
        assert!(!output.contains("已选商品 已选商品"));
    }

    #[test]
    fn filters_generic_task_objective_shell_from_final_answer() {
        let raw = "给新手学生的三条618返场短视频开头文案：\n\n1. \
                   办公室姐妹看过来，温显防漏保温杯返场了。\n\n整合财务、千川投放和创意结果，\
                   形成可执行运营方案。\n\n落地注意：每天看退款率，超过8%就调整素材。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("办公室姐妹看过来"));
        assert!(output.contains("落地注意"));
        assert!(!output.contains("整合财务、千川投放和创意结果"));
    }

    #[test]
    fn filters_history_questions_misclassified_as_material_evidence() {
        let raw = "学生开网店第一周先看访客数、点击率和咨询加购率。\n\n资料依据：- \
                   请分析这个商品在618返场活动里最应该主打什么卖点？回答给新手学生看\n\n资料依据：\
                   原因是商品资料明确要求主打这三个点，活动针对的是办公室女性客群。\n\n资料依据：\
                   广告数据里点击率 CTR=1.2%、转化率 CVR=2.1%、退款率=18%，最大问题是退款率偏高。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("学生开网店第一周先看访客数"));
        assert!(output.contains("退款率=18%"));
        assert!(!output.contains("请分析这个商品"));
        assert!(!output.contains("618返场活动"));
        assert!(!output.contains("办公室女性客群"));
    }

    #[test]
    fn filters_internal_memory_labels_from_final_answer() {
        let raw = "继续沿用上一轮的校园二手教材包和618返场活动，先把首屏标题压短。\n\\
                   n资料关键事实：Same conversation structured memory: product=校园二手教材包; \
                   campaign=618返场教材包\n资料关键事实：Rolling summary: No older compressed \
                   history. Recent user facts: asked for table and image \
                   diagnosis.\n\n首屏建议：突出版本齐全、48小时发出和开学前可复核书况。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("校园二手教材包"));
        assert!(output.contains("首屏建议"));
        assert!(!lower.contains("same conversation structured memory"));
        assert!(!lower.contains("rolling summary"));
        assert!(!lower.contains("recent user facts"));
        assert!(!lower.contains("no older compressed history"));
    }

    #[test]
    fn rewrites_internal_evidence_terms_from_final_answer() {
        let raw = "QA矩阵智能保温杯适合做学生电商实训，因为商品锚点：温显、防漏、礼盒都清楚。\\
                   n资料边界：低置信参考只能作为补充，置信度0.96 \
                   的内容仍要用真实素材核对，confidence=0.72 \
                   不能直接展示给学生。\n建议先围绕宿舍、社团礼品和冬季热饮三个场景做短视频测试。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("QA矩阵智能保温杯"));
        assert!(output.contains("温显、防漏、礼盒"));
        assert!(output.contains("仍需核验的参考"));
        assert!(output.contains("资料来源较明确"));
        assert!(output.contains("不能直接展示给学生"));
        assert!(output.contains("短视频测试"));
        assert!(!output.contains("商品锚点"));
        assert!(!output.contains("低置信"));
        assert!(!output.contains("置信度"));
        assert!(!lower.contains("confidence"));
    }

    #[test]
    fn final_sanitizing_restores_readable_sections_without_internal_terms() {
        let raw =
            "同学们好，基于当前你手动引用的QA矩阵智能保温杯全部资料，\
             现将这款商品适合和不适合做学生电商实训的地方明确梳理如下：\
             【核心业务结论与定价判断】这款商品当前129元的售价完全符合不低于119元的定价安全线，\
             定价合规性明确，整体规则边界清晰。【100%确定、\
             可直接落地的适合实训环节】选品定位练习：商品有明确的客群边界，\
             严格禁止学生党低价定位。内容合规练习：首屏文案必须包含温显、防漏、礼盒三个核心卖点。\
             【当前资料缺失、暂不适合开展的实训环节】当前没有实际曝光、点击、成交、成本、\
             退款等真实运营数据。【依据说明】确定部分的支撑来源：QA矩阵智能保温杯商品锚点、\
             QA矩阵商品知识、QA矩阵卖点素材，置信度0.96。\
             【下一步核验动作】你可以优先核验抖音平台智能保温杯类目的公开运营规则。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("【核心业务结论与定价判断】\n\n这款商品"));
        assert!(output.contains("\n\n【100%确定、可直接落地的适合实训环节】\n\n选品定位练习"));
        assert!(
            output.contains("\n\n【当前资料缺失、暂不适合开展的实训环节】\n\n当前没有实际曝光")
        );
        assert!(output.contains("\n\n【依据说明】\n\n确定部分的支撑来源"));
        assert!(output.contains("商品资料、QA矩阵商品知识"));
        assert!(output.contains("资料来源较明确"));
        assert!(!output.contains("商品锚点"));
        assert!(!output.contains("置信度"));
        assert!(!lower.contains("confidence"));
    }

    #[test]
    fn final_sanitizing_keeps_existing_paragraph_breaks() {
        let raw = "先给结论：这款商品可以作为学生实训样例，但不能直接当成低价引流品。\n\\
                   \
                   n适合部分：商品卖点、价格边界和场景都比较清楚，可以练选品定位。\n\n依据说明：\
                   支撑来自QA矩阵智能保温杯商品锚点，置信度0.96。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("样例，但不能直接当成低价引流品。\n\n适合部分："));
        assert!(output.contains("选品定位。\n\n依据说明："));
        assert!(output.contains("QA矩阵智能保温杯商品资料"));
        assert!(output.contains("资料来源较明确"));
        assert!(!output.contains("商品锚点"));
        assert!(!output.contains("置信度"));
        assert!(!lower.contains("confidence"));
    }

    #[test]
    fn final_sanitizing_restores_generic_inline_colon_headings() {
        let raw = "这款商品可以做实训，但要先限定任务范围。适合部分：围绕卖点、\
                   价格和场景做选品定位。不适合部分：没有真实投放数据时，不要做确定性ROI判断。\
                   核验动作：补成本、退款和平台规则资料。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("限定任务范围。\n\n适合部分：\n\n围绕卖点"));
        assert!(output.contains("选品定位。\n\n不适合部分：\n\n没有真实投放数据"));
        assert!(output.contains("ROI判断。\n\n核验动作：\n\n补成本"));
    }

    #[test]
    fn final_sanitizing_restores_inline_practice_headings() {
        let raw = "当前暂不适合做进阶判断。进阶数据类实训：没有曝光、点击、成交数据，\
                   暂不做转化测算。行业对比类实训：没有可采信公开来源，暂不做行业对标。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("进阶判断。\n\n进阶数据类实训：\n\n没有曝光"));
        assert!(output.contains("转化测算。\n\n行业对比类实训：\n\n没有可采信"));
    }

    #[test]
    fn final_sanitizing_restores_inline_numbered_lists() {
        let raw = "以下是给电商学生的全品类通用选品复盘清单，包含核心复盘模块、字段及风险提示：1. \
                   前置说明：本清单为通用运营规则；2. 选品启动前复盘：核验选品依据；3. \
                   小范围试卖后复盘：统计曝光、点击、咨询。风险边界：\
                   无对应资质不得进入高风险品类。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("风险提示：\n\n1. 前置说明："));
        assert!(output.contains("通用运营规则；\n\n2. 选品启动前复盘："));
        assert!(output.contains("核验选品依据；\n\n3. 小范围试卖后复盘："));
        assert!(output.contains("咨询。\n\n风险边界："));
    }

    #[test]
    fn final_sanitizing_promotes_plain_section_headings() {
        let raw = "当前判断\n校园二手教材包可以先做开学前低风险试跑。\n\n下一步动作\n先核对版本、\
                   ISBN、成色和缺页情况。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("## 当前判断\n校园二手教材包"));
        assert!(output.contains("## 下一步动作\n先核对版本"));
    }

    #[test]
    fn final_sanitizing_does_not_promote_risk_sentence_as_heading() {
        let raw = "未核验平台规则可能导致素材违规、投放受限、定价被误判\\
                   n提前囤积库存可能导致滞销现金流风险\n是否有指定的淘宝平台规则查询入口？";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("提前囤积库存可能导致滞销现金流风险"));
        assert!(!output.contains("## 提前囤积库存可能导致滞销现金流风险"));
    }

    #[test]
    fn final_sanitizing_formats_business_key_value_blocks_as_bullets() {
        let raw = "教学模式分析：校园二手教材包是否适合开学季电商实训\n\n判断逻辑（通俗版）\\
                   \
                   n需求匹配度：开学前学生确实需要按课程快速配齐教材。\n履约难度：版本、ISBN、\
                   成色和缺页必须能逐本核对。\n风险边界：不能承诺全新、官方指定或固定升值。\n\\
                   n下一步实训动作\n核对每本书的版本、ISBN、成色和缺页\\
                   n建一张教材包字段表记录成本、售价和售后问题";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("## 判断逻辑"));
        assert!(output.contains("- 需求匹配度：开学前学生确实需要按课程快速配齐教材。"));
        assert!(output.contains("- 履约难度：版本、ISBN、成色和缺页必须能逐本核对。"));
        assert!(output.contains("- 风险边界：不能承诺全新、官方指定或固定升值。"));
        assert!(output.contains("## 下一步实训动作"));
        assert!(output.contains("- 核对每本书的版本、ISBN、成色和缺页"));
        assert!(output.contains("- 建一张教材包字段表记录成本、售价和售后问题"));
    }

    #[test]
    fn final_sanitizing_formats_short_copy_lines_and_drops_empty_evidence_heading() {
        let raw = "给新手学生的三条短视频开头文案：\n开学季高数英语经济学教材一次配齐，\
                   版本可核对48小时发\n成套二手教材包，版本对得上才发货，48小时就能收到\\
                   n开学教材不用挨个找，成套版本可查48小时发出\n\n资料依据：## 下一步实训动作 \
                   准备3-5套真实教材包，补齐版本和书况照片。\n\n资料依据：";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("给新手学生的三条短视频开头文案："));
        assert!(output.contains("- 开学季高数英语经济学教材一次配齐，版本可核对48小时发"));
        assert!(output.contains("- 成套二手教材包，版本对得上才发货，48小时就能收到"));
        assert!(output.contains("- 开学教材不用挨个找，成套版本可查48小时发出"));
        assert!(output.contains("## 下一步实训动作"));
        assert!(output.contains("准备3-5套真实教材包"));
        assert!(!output.ends_with("资料依据："));
        assert!(!output.contains("资料依据：##"));
    }

    #[test]
    fn final_sanitizing_truncates_tail_history_answer_quotes() {
        let raw = "校园二手教材包+618返场活动 优先级动作排布：\n- \
                   今天：核对所有在售教材包的版本清单、书况标签和618返场活动公示内容。\n- \
                   优先理由：这是交易链路最前端的避错动作，减少版本咨询混乱、退单和售后争议。\n- \
                   明天：把之前产出的3条短视频开头文案对应素材全部上线，每条绑定验证指标投放。\n- \
                   优先理由：信息对齐后立刻落地内容引流，承接618返场剩余流量。\n- \
                   本周：整理真实交易数据，统计版本核对咨询量、下单转化率、\
                   售后反馈量和发货达标率。\n- \
                   优先理由：前面的信息校验和内容投放跑完后，再汇总全链路数据做复盘。\n- \
                   所有动作过程中只记录真实产生的数据，不要编造任何指标数值。\n\n## 判断依据\n\n- \
                   精准踩中刚性需求：大学生开学最急需配齐新学期教材，打包售卖免去逐本找货。\n- \
                   卖点真实无需编造：59元定价亲民，成套配齐、版本可核对、48小时发出。\n- \
                   合规风险低：所属淘宝图书教材/校园二手类目，新手不容易踩规则红线。\n\n资料依据：\
                   \n\n2. 文案：怕买错教材版本？这款二手教材包支持版本核对，高数、\
                   英语等开学必备教材成套打包，59元配齐，48小时发货";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("校园二手教材包+618返场活动"));
        assert!(output.contains("本周：整理真实交易数据"));
        assert!(!output.contains("## 判断依据"));
        assert!(!output.contains("精准踩中刚性需求"));
        assert!(!output.contains("资料依据"));
        assert!(!output.contains("2. 文案：怕买错教材版本"));
    }

    #[test]
    fn final_sanitizing_filters_worker_output_policy_lines() {
        let raw = "动作优先级排序\n- 今天：完成商品首屏ISBN核对流程上线\n- \
                   明天：完成校园社群、短视频两个核心渠道的ROAS分渠道复盘\n- \
                   本周：全量上线下单前书况、退换规则明确告知的链路优化\n\n排序理由\\
                   n版本咨询是用户当前最集中的转化阻碍，今天上线就能直接减少客服压力。\\
                   \
                   n先写清目标、周期、预算和成功指标；如果用户只要局部交付，先完成局部交付，\
                   不把短任务展开成完整方案。\n把数据、创意、投放和客服输入合并成一张行动清单：\
                   谁做、今天做什么、明天看什么数据。\n保留清晰的复盘口径，重点看点击、加购、\
                   付款、退款和客服咨询；表现明显低于本轮其他方向或本店上一轮基准时再调整。\\
                   \
                   n缺资料时先给当前问题能用的版本，等用户补齐商品、平台、价格、\
                   预算或表格后再精修。\n先把这类商品当成一个小项目来做：\
                   明确用户真正要解决的问题，再选择人群、场景、渠道、素材或复盘指标中的关键动作，\
                   避免一开始就做大预算。\n同时改动过多变量会导致复盘无法归因；\
                   每轮只改最影响判断的少量变量。\n资料依据：判断依据：\
                   大学生开学需要快速配齐教材，59元、版本可核对、48小时发出、书况清晰标注";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("动作优先级排序"));
        assert!(output.contains("版本咨询是用户当前最集中的转化阻碍"));
        assert!(!output.contains("先写清目标"));
        assert!(!output.contains("把数据、创意、投放和客服输入"));
        assert!(!output.contains("保留清晰的复盘口径"));
        assert!(!output.contains("缺资料时先给当前问题能用的版本"));
        assert!(!output.contains("先把这类商品当成一个小项目"));
        assert!(!output.contains("同时改动过多变量"));
        assert!(!output.contains("资料依据：判断依据"));
    }

    #[test]
    fn final_sanitizing_filters_bracketed_history_evidence_prefix() {
        let raw =
            "优先级动作\n- 今天：补全618返场投放素材的版本核验、书况标签、成套配齐核心卖点元素。\\
             n- 明天：把版本核对流程更新到商品详情页首屏。\n\n资料依据：【判断依据】1. \
             天然场景：开学季大学生必买教材，不用教育用户需求 2. 低门槛：客单价59元，试错成本低。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("优先级动作"));
        assert!(output.contains("版本核验"));
        assert!(!output.contains("资料依据"));
        assert!(!output.contains("【判断依据】"));
    }

    #[test]
    fn final_sanitizing_keeps_current_turn_copywriting_items() {
        let raw = "文案：开学季教材不用逐本找！成套高数、英语、经济学教材配齐，版本可核对，\
                   48小时发出。画面：展示打包好的成套教材。转化意图：吸引学生点击。\n\n2. \
                   文案：怕买错教材版本？这款二手教材包支持版本核对，59元配齐，48小时发货。画面：\
                   手持教材展示ISBN码。转化意图：提升下单转化率。\n\n3. \
                   文案：开学季环保又省钱！二手教材包59元配齐教材，书况清晰标注。画面：\
                   书况标签特写。转化意图：激发购买意愿。";

        let output = sanitize_final_user_facing_text(raw).expect("copy answer should remain");

        assert!(output.contains("文案：开学季教材不用逐本找"));
        assert!(output.contains("2. 文案：怕买错教材版本"));
        assert!(output.contains("3. 文案：开学季环保又省钱"));
    }

    #[test]
    fn final_sanitizing_filters_tail_material_evidence_history_replay() {
        let raw = "建议今天先做库存盘点，确认每个品类的可售套数是否能支撑初期活动订单。\n\n原因：\
                   如果库存不足，「开学公共课教材一次配齐」这个核心卖点不成立，\
                   还会导致缺货和退款纠纷。\n\n资料依据：运营对象&平台 - \
                   服务对象：开学季需要购买高数、英语、经济学等公共课教材的大学生 - \
                   运营平台：淘宝 - 商品属性：公共课二手教材包，售价59元\n\n资料依据：\
                   表格要包含动作、负责人、判断指标、达标线、风险处理，重点看59元定价、库存、\
                   退款和ROAS";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain");

        assert!(output.contains("建议今天先做库存盘点"));
        assert!(output.contains("退款纠纷"));
        assert!(!output.contains("资料依据"));
        assert!(!output.contains("运营对象&平台"));
        assert!(!output.contains("表格要包含动作"));
    }

    #[test]
    fn final_sanitizing_filters_material_evidence_heading_with_history_replay_body() {
        let raw = "三条文案均严格控制在18字以内，完全匹配主图素材的预设布局位置。\n\n资料依据：\n\\
                   \
                   n3. 复盘字段：活动期间统计咨询top问题、成交转化率、退款原因、库存剩余量";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain");

        assert!(output.contains("三条文案均严格控制在18字以内"));
        assert!(!output.contains("资料依据"));
        assert!(!output.contains("复盘字段"));
        assert!(!output.contains("咨询top问题"));
    }

    #[test]
    fn final_sanitizing_preserves_markdown_tables_for_frontend_table_renderer() {
        let raw = "## 执行表\n\n| 指标 | 怎么看 | 可能问题 | 下一步动作 |\n| --- | --- | --- | \
                   --- |\n| 触达/点击 | 用户是否愿意进入详情 | 入口表达不清 | 测试标题和主图 |\n| \
                   转化/付款 | 用户是否完成购买 | 价格或信任不足 | 补清楚版本、书况和发货 \
                   |\n\n风险边界：没有真实销量和退款数据时，只能做方向判断。";

        let output = sanitize_final_user_facing_text(raw).expect("table answer should remain");

        assert!(output.contains("| 指标 | 怎么看 | 可能问题 | 下一步动作 |"));
        assert!(output.contains("| --- | --- | --- | --- |"));
        assert!(
            output.contains("| 触达/点击 | 用户是否愿意进入详情 | 入口表达不清 | 测试标题和主图 |")
        );
        assert!(output.contains("\n\n风险边界："));
    }

    #[test]
    fn final_sanitizing_repairs_broken_multiline_table_rows() {
        let raw = "618返场执行表\n| 时间 | 动作 | 负责人 | 风险处理 |\n| ---- | ---- | ---- | \
                   ---- |\n| 今天 | 1. 完成主图上线；\n\n2. 同步客服规范；\n\n3. 盘点教材包库存 | \
                   学生运营 | 1. 若素材排版不符合要求，1小时内调整；\n\n2. \
                   若库存不足，先临时下架对应链接 |\n| 明天 | 每2小时看咨询和付款 | 学生运营+客服 \
                   | 若ROAS低于2，暂停付费投放 |";

        let output = sanitize_final_user_facing_text(raw).expect("table answer should remain");

        assert!(output.contains("| 时间 | 动作 | 负责人 | 风险处理 |"));
        assert!(output.contains("| ---- | ---- | ---- | ---- |"));
        assert!(output.contains(
            "| 今天 | 1. 完成主图上线； 2. 同步客服规范； 3. 盘点教材包库存 | 学生运营 | 1. \
             若素材排版不符合要求，1小时内调整； 2. 若库存不足，先临时下架对应链接 |"
        ));
        assert!(output.contains(
            "| 明天 | 每2小时看咨询和付款 | 学生运营+客服 | 若ROAS低于2，暂停付费投放 |"
        ));
        assert!(!output.contains("| 今天 | 1. 完成主图上线；\n\n2."));
    }

    #[test]
    fn final_sanitizing_splits_inline_markdown_tables() {
        let raw = "今日到后天618返场活动执行表如下，聚焦59元定价、库存、退款、ROAS核心指标：\
                   |执行日期|动作|负责人|判断指标|达标线|风险处理| |---|---|---|---|---|---| \
                   |今日|核对教材库存|库存岗|库存盘点准确率|账实一致|库存不足时关闭超库存下单通道| \
                   |明日|监控ROAS和退款|数据岗|ROAS/退款率|ROAS不低于目标|低转化渠道暂停投放|\\
                   n活动预算8888元，目标ROAS3.2。";

        let output = sanitize_final_user_facing_text(raw).expect("table answer should remain");

        assert!(output.contains(
            "今日到后天618返场活动执行表如下，聚焦59元定价、库存、退款、ROAS核心指标：\\
             n|执行日期|动作|负责人|判断指标|达标线|风险处理|"
        ));
        assert!(output.contains("\n|---|---|---|---|---|---|"));
        assert!(output.contains(
            "\n|今日|核对教材库存|库存岗|库存盘点准确率|账实一致|库存不足时关闭超库存下单通道|"
        ));
        assert!(output.contains(
            "\n|明日|监控ROAS和退款|数据岗|ROAS/退款率|ROAS不低于目标|低转化渠道暂停投放|"
        ));
    }

    #[test]
    fn final_sanitizing_bulletizes_table_followup_lines() {
        let raw = "校园二手教材包618返场活动复盘表\n| 指标 | 怎么看 | 可能问题 | 下一步动作 |\n| \
                   ---- | ---- | ---- | ---- |\n| 48小时发货率 | 核对承诺是否达标 | \
                   库存或打包效率不足 | 按套系统计库存 \
                   |\n618返场活动预算8888元，目标ROAS3.2，核心投放渠道为校园社群、短视频\\
                   n校园二手教材包核心风险点集中在版本不匹配、书况争议、库存滞销、退换纠纷\\
                   n优先优化首屏版本核对提示，降低用户咨询量，减少后续版本不符的退款风险";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("## 校园二手教材包618返场活动复盘表"));
        assert!(output.contains("| 指标 | 怎么看 | 可能问题 | 下一步动作 |"));
        assert!(
            output.contains("- 618返场活动预算8888元，目标ROAS3.2，核心投放渠道为校园社群、短视频")
        );
        assert!(
            output.contains(
                "- 校园二手教材包核心风险点集中在版本不匹配、书况争议、库存滞销、退换纠纷"
            )
        );
        assert!(
            output
                .contains("- 优先优化首屏版本核对提示，降低用户咨询量，减少后续版本不符的退款风险")
        );
    }

    #[test]
    fn removes_unverified_live_search_availability_claims() {
        let raw = "外部搜索虽可用但暂未完成，先按已选商品资料判断。\n本轮未拿到实时来源，\
                   不能当作最新竞品价格。\n可先把二手教材包价格和书况字段整理出来。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("本轮未拿到实时来源"));
        assert!(output.contains("二手教材包价格"));
        assert!(!output.contains("外部搜索虽可用"));
        assert!(!output.contains("外部搜索可用"));
    }

    #[test]
    fn rewrites_internal_skill_ids_for_student_answer() {
        let raw = "本地无匹配商品知识库，当前无其他工作区数据源可用，\
                   material_table_profiler技能未收到可剖析的表格行数据，暂无法产出结果。";

        let output = sanitize_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("表格资料分析"));
        assert!(!output.contains("material_table_profiler"));
        assert!(!output.contains("skill"));
    }

    #[test]
    fn filters_runtime_skill_event_lines_from_answer() {
        let raw = "当前只掌握商品名称和售价，先补成本、平台和用户反馈。\nskill:\
                   learning_teaching_explain status=success \
                   summary=教学解释已生成。\n用户本轮手动选择的预设商品资料：千乐咖啡（已选商品，\
                   来源 context.selected_summary）。\n本轮未检测到 \
                   context.selected_summary；不注入任何预设商品资料，只按用户文本、\
                   会话历史和工作区上下文理解问题。\n后续再做毛利核算和转化分析。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
        let lower = output.to_ascii_lowercase();
        assert!(output.contains("当前只掌握商品名称和售价"));
        assert!(output.contains("本轮已使用你选中的商品资料：千乐咖啡。"));
        assert!(output.contains("本轮没有手动选择具体商品"));
        assert!(output.contains("后续再做毛利核算"));
        assert!(!lower.contains("skill:"));
        assert!(!lower.contains("status="));
        assert!(!lower.contains("context.selected_summary"));
    }
}
