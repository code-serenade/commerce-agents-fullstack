mod layout;
mod policy;

use layout::restore_readable_answer_layout;
use policy::{UserFacingAnswerStructureRule, user_facing_policy};

use crate::{
    core::protocol::WorkerReport,
    llm_routing::{
        is_llm_access_denied_message, is_llm_provider_error_message,
        llm_access_denied_user_message, llm_provider_error_user_message,
    },
};

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
    let output = apply_answer_structure_rules(&output);
    let lines = output.lines().map(str::to_string).collect::<Vec<_>>();
    let lines = drop_historical_evidence_quote_blocks(lines);
    let lines = truncate_tail_historical_answer_quotes(lines);
    let output = normalize_blank_lines(lines);
    (!output.is_empty() && !is_internal_noise_line(&output)).then_some(output)
}

fn apply_answer_structure_rules(text: &str) -> String {
    for rule in &user_facing_policy().answer_structure_rules {
        if !answer_structure_rule_matches(rule, text) {
            continue;
        }
        if let Some(output) = structure_answer_by_rule(text, rule) {
            return output;
        }
    }
    text.to_string()
}

fn answer_structure_rule_matches(rule: &UserFacingAnswerStructureRule, text: &str) -> bool {
    !rule.sections.is_empty()
        && (rule.text_any.is_empty() || rule.text_any.iter().any(|marker| text.contains(marker)))
}

struct AnswerStructureBucket {
    heading: String,
    lines: Vec<String>,
    seen: bool,
}

fn structure_answer_by_rule(text: &str, rule: &UserFacingAnswerStructureRule) -> Option<String> {
    let mut buckets = rule
        .sections
        .iter()
        .map(|section| AnswerStructureBucket {
            heading: section.heading.clone(),
            lines: Vec::new(),
            seen: false,
        })
        .collect::<Vec<_>>();
    let mut preamble = Vec::new();
    let mut current_section = None;
    let mut matched_sections = 0;

    for raw_line in text.lines() {
        if let Some((section_index, body)) = answer_structure_heading_match(raw_line, rule) {
            current_section = Some(section_index);
            if !buckets[section_index].seen {
                buckets[section_index].seen = true;
                matched_sections += 1;
            }
            if let Some(body) = body.filter(|body| !body.trim().is_empty()) {
                buckets[section_index].lines.push(body);
            }
            continue;
        }

        let target = current_section
            .map(|index| &mut buckets[index].lines)
            .unwrap_or(&mut preamble);
        target.push(raw_line.to_string());
    }

    if matched_sections < 2 {
        return None;
    }

    let mut output = trim_answer_structure_lines(&preamble);
    let mut emitted_sections = 0;
    if !output.is_empty() {
        output.push(String::new());
    }

    for bucket in buckets {
        let lines = trim_answer_structure_lines(&bucket.lines);
        if !bucket.seen || lines.is_empty() {
            continue;
        }
        output.push(format!("## {}", bucket.heading));
        output.push(String::new());
        output.extend(lines);
        output.push(String::new());
        emitted_sections += 1;
    }

    (emitted_sections >= 2).then(|| normalize_blank_lines(output))
}

fn answer_structure_heading_match(
    line: &str,
    rule: &UserFacingAnswerStructureRule,
) -> Option<(usize, Option<String>)> {
    let candidate = strip_answer_heading_markup(line);
    if candidate.is_empty() {
        return None;
    }

    if let Some((heading, body)) = split_answer_heading_body(candidate)
        && let Some(index) = answer_structure_section_index(rule, heading)
    {
        return Some((index, Some(body.to_string())));
    }

    let heading = normalize_answer_heading(candidate);
    answer_structure_section_index(rule, &heading).map(|index| (index, None))
}

fn strip_answer_heading_markup(line: &str) -> &str {
    let mut text = line.trim();
    if text.starts_with('#') {
        text = text.trim_start_matches('#').trim_start();
    }
    text = strip_unordered_list_prefix(text);
    strip_ordered_list_prefix(text)
}

fn strip_unordered_list_prefix(text: &str) -> &str {
    ["- ", "* ", "• "]
        .iter()
        .find_map(|prefix| text.strip_prefix(prefix))
        .unwrap_or(text)
}

fn strip_ordered_list_prefix(text: &str) -> &str {
    let mut digit_end = 0;
    for (index, ch) in text.char_indices() {
        if !ch.is_ascii_digit() {
            break;
        }
        digit_end = index + ch.len_utf8();
    }
    if digit_end == 0 || digit_end > 3 {
        return text;
    }

    let rest = &text[digit_end..];
    let Some(delimiter) = rest.chars().next() else {
        return text;
    };
    if !matches!(delimiter, '.' | '、' | ')' | '）') {
        return text;
    }

    let rest = &rest[delimiter.len_utf8()..];
    if !rest.chars().next().is_some_and(char::is_whitespace) {
        return text;
    }
    rest.trim_start()
}

fn split_answer_heading_body(candidate: &str) -> Option<(&str, &str)> {
    let (colon_index, colon_width) = first_colon(candidate)?;
    let heading = candidate[..colon_index].trim();
    let body = candidate[colon_index + colon_width..].trim();
    if heading.is_empty() || body.is_empty() {
        return None;
    }
    Some((heading, body))
}

fn normalize_answer_heading(candidate: &str) -> String {
    candidate
        .trim()
        .trim_matches(['#', ':', '：', '。', '.', ' '])
        .trim()
        .to_string()
}

fn answer_structure_section_index(
    rule: &UserFacingAnswerStructureRule,
    heading: &str,
) -> Option<usize> {
    let heading = answer_heading_key(heading);
    rule.sections.iter().position(|section| {
        answer_heading_key(&section.heading) == heading
            || section
                .aliases
                .iter()
                .any(|alias| answer_heading_key(alias) == heading)
    })
}

fn answer_heading_key(value: &str) -> String {
    normalize_answer_heading(value)
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn trim_answer_structure_lines(lines: &[String]) -> Vec<String> {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    lines[start..end].to_vec()
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

    let candidate = line[..keep_end].trim_end();
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
            && is_uuid_like(&input[index..index + 36])
        {
            output.push_str("已选商品");
            index += 36;
            continue;
        }

        let Some(ch) = input[index..].chars().next() else {
            break;
        };
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

fn is_uuid_boundary(input: &str, index: usize, before: bool) -> bool {
    let ch = if before {
        input[..index].chars().next_back()
    } else {
        input[index..].chars().next()
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
        output.push_str(&rest[..start]);
        let segment_start = start + open.len_utf8();
        let Some(end) = rest[segment_start..].find(close) else {
            output.push_str(&rest[start..]);
            return output;
        };
        let segment_end = segment_start + end;
        let segment = &rest[segment_start..segment_end];
        if placeholder_segment(segment) {
            if let Some(replacement) = placeholder_replacement(segment) {
                output.push_str(&replacement);
            }
        } else {
            output.push(open);
            output.push_str(segment);
            output.push(close);
        }
        rest = &rest[segment_end + close.len_utf8()..];
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
        output.push_str(&rest[..index]);
        let after_marker = &rest[index + "置信度".len()..];
        let consumed = trim_confidence_value(after_marker);
        if consumed == 0 {
            output.push_str("资料来源");
        } else {
            output.push_str("资料来源较明确");
        }
        rest = &after_marker[consumed..];
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
        output.push_str(&rest[..index]);
        let after_marker = &rest[index + "confidence".len()..];
        let consumed = trim_confidence_value(after_marker);
        if consumed == 0 {
            output.push_str("资料来源");
        } else {
            output.push_str("资料来源较明确");
        }
        rest = &after_marker[consumed..];
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
    for ch in input[value_start..].chars() {
        if ch.is_ascii_digit() || matches!(ch, '.' | '%' | '％') {
            has_digit = true;
            consumed += ch.len_utf8();
            continue;
        }
        break;
    }

    if has_digit {
        for ch in input[consumed..].chars() {
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

    let mut product = line[index + marker.len()..].trim().to_string();
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
    let label = body[..colon_index].trim();
    if !looks_like_internal_role_label(label) {
        return line.trim().to_string();
    }

    format!("{prefix}{}", body[colon_index + colon_width..].trim_start())
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
    let Some(end) = label[start + 1..].find(')') else {
        return false;
    };
    let role = &label[start + 1..start + 1 + end];
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
    for index in 0..lines.len() {
        if !has_substantial_answer_before(&lines[..index]) {
            continue;
        }
        if looks_like_tail_historical_answer_quote_start(&lines, index) {
            return lines[..index].to_vec();
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
        if blocks[..half] == blocks[half..] {
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
        if let Some(run_len) = longest_seen_line_run(&output, &lines[cursor..]) {
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
    (2..=max_len).rev().find(|run_len| {
        seen.windows(*run_len)
            .any(|window| window == &remaining[..*run_len])
    })
}

fn collapse_repeated_line_sequence(mut lines: Vec<String>) -> Vec<String> {
    loop {
        let len = lines.len();
        if len <= 1 || !len.is_multiple_of(2) || lines[..len / 2] != lines[len / 2..] {
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
            && is_uuid_like(&line[index..index + 36])
        {
            return true;
        }
        let Some(ch) = line[index..].chars().next() else {
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
mod tests;
