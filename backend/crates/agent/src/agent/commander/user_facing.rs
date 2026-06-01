use crate::{
    core::protocol::WorkerReport,
    llm_routing::{is_llm_access_denied_message, llm_access_denied_user_message},
};

pub(crate) fn user_facing_report_text(report: &WorkerReport) -> Option<String> {
    if report_has_llm_access_denied(report) {
        return Some(llm_access_denied_user_message().to_string());
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
    let live_search = live_search_user_facing_text(output);
    if role == "search" {
        return live_search.or_else(|| role_output_business_text(output));
    }

    let business_text = role_output_business_text(output);
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
    let text = std::iter::once(output.summary.as_str())
        .chain(output.recommendations.iter().map(String::as_str))
        .chain(output.findings.iter().map(String::as_str))
        .filter(|item| !looks_like_live_search_evidence(item))
        .find_map(|item| {
            let text = sanitize_user_facing_text(item)?;
            if has_material_evidence && looks_like_no_source_claim(&text) {
                None
            } else {
                Some(text)
            }
        })
        .or_else(|| material_evidence_sections(output).into_iter().next())?;
    Some(append_material_evidence_once(&text, output))
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
        sections.push("本轮已通过外部搜索取得实时来源。".to_string());
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

fn looks_like_live_search_evidence(item: &str) -> bool {
    let item = item.trim();
    item.contains("实时搜索来源") || (item.contains("http://") || item.contains("https://"))
}

fn looks_like_no_source_claim(item: &str) -> bool {
    [
        "本轮未拿到",
        "未拿到实时来源",
        "没有可引用的实时外部来源",
        "没有可引用的实时来源",
        "无实时外部来源",
        "没有可采信实时来源",
        "缺少官方发布",
        "缺少抖音电商官方发布",
        "无法给出具体的合规要求细则和对应来源信息",
        "不能直接给出对应规则明细",
    ]
    .iter()
    .any(|marker| item.contains(marker))
}

pub(crate) fn sanitize_user_facing_text(raw: &str) -> Option<String> {
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

pub(crate) fn sanitize_final_user_facing_text(raw: &str) -> Option<String> {
    let output = sanitize_user_facing_text(raw)?;
    let output = restore_readable_answer_layout(&output);
    let lines = output.lines().map(str::to_string).collect::<Vec<_>>();
    let output = collapse_repeated_blocks(lines).join("\n");
    (!output.is_empty() && !is_internal_noise_line(&output)).then_some(output)
}

pub(crate) fn no_user_facing_content_message() -> String {
    "这次没有拿到可直接展示的完整内容。请补充更明确的问题或关键材料，我再继续整理。".to_string()
}

fn clean_user_facing_line(line: &str) -> String {
    let line = strip_role_prefix(line.trim());
    let line = rewrite_note_prefix(&line);
    let line = rewrite_internal_evidence_terms(&line);
    let line = rewrite_selected_product_source_line(&line);
    let line = rewrite_internal_skill_names(&line);
    let line = rewrite_business_phrases(&line);
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

    for marker in [
        "commerce product context resolved",
        "use these facts when relevant",
        "relevant, cite product",
        "do not invent missing product details",
        "product source product:",
        "knowledge source product_knowledge:",
        "requested context refs:",
        "local commerce rag context",
        "local commerce rag retrieval",
        "retrieval query terms:",
        "retrieved knowledge source",
        "no matching local product knowledge",
        "context.selected_summary",
        "context.combined_summary",
        "commerce.selected_product_context",
        "productcontextsummary",
        "workspacecontext.summary",
        "workspacecontextpack.summary",
    ] {
        if let Some(index) = lower.find(marker) {
            keep_end = keep_end.min(index);
        }
    }

    let candidate = line[..keep_end].trim_end();
    (!candidate.trim().is_empty()).then(|| candidate.to_string())
}

fn rewrite_internal_skill_names(line: &str) -> String {
    let mut text = line.to_string();
    for (from, to) in [
        ("material_table_profiler", "表格资料分析"),
        ("material_file_understanding", "文件资料理解"),
        ("material_image_diagnosis", "图片资料分析"),
        ("material_multi_file_compare", "多资料对比分析"),
        ("search_platform_policy", "平台规则搜索"),
        ("search_competitor", "竞品搜索"),
        ("search_trends", "趋势搜索"),
        ("search_market_info", "市场信息搜索"),
        ("search_industry_benchmarks", "行业基准搜索"),
        ("search_product_reviews", "用户评价搜索"),
        ("service_ticket_handler", "客服工单处理"),
        ("service_escalation_flow", "客服升级处理"),
        ("service_return_handler", "退换货处理"),
        ("service_query_product", "商品资料查询"),
        ("service_sentiment_analyzer", "客服情绪判断"),
        ("accounting_profit_analysis", "利润分析"),
        ("accounting_budget_plan", "预算规划"),
        ("accounting_cost_calc", "成本核算"),
        ("engineering_arch_review", "技术链路检查"),
        ("engineering_system_check", "系统检查"),
        ("douyin_ads_plan_structure", "投放结构规划"),
        ("douyin_ads_budget_pacing", "预算节奏分析"),
        ("creative_ip_branding", "内容卖点梳理"),
        ("creative_live_script", "直播脚本梳理"),
        ("web_seo_optimize", "搜索优化"),
        ("web_keyword_research", "关键词研究"),
        ("web_title_generator", "标题优化"),
        ("web_product_description", "商品描述优化"),
        ("web_page_conversion", "页面转化优化"),
    ] {
        text = text.replace(from, to);
    }
    text
}

fn rewrite_business_phrases(line: &str) -> String {
    line.replace("并非点击率不足", "点击率不是主要矛盾")
        .replace("并不是点击率不足", "点击率不是主要矛盾")
        .replace("不是点击率不足", "点击率不是主要矛盾")
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
    let text = insert_breaks_around_inline_colon_headings(&text);
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
            let heading = &output[heading_start..];
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
        let heading = chars[start..index].iter().collect::<String>();
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
    (2..=28).contains(&char_count)
        && heading.chars().any(is_cjk)
        && heading
            .chars()
            .all(|ch| is_heading_text_char(ch) || ch.is_whitespace())
        && !heading.contains("如")
        && heading_has_section_suffix(heading)
}

fn is_heading_text_char(ch: char) -> bool {
    is_cjk(ch)
        || ch.is_ascii_alphanumeric()
        || matches!(ch, '%' | '％' | '、' | '·' | '-' | '_' | ' ')
}

fn heading_has_section_suffix(heading: &str) -> bool {
    [
        "结论",
        "判断",
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
        "方案",
        "策略",
        "步骤",
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
    let marker = "用户本轮手动选择的预设商品资料：";
    let Some(index) = line.find(marker) else {
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
    )
}

fn collapse_repeated_blocks(lines: Vec<String>) -> Vec<String> {
    let lines = collapse_repeated_line_sequence(lines);
    let lines = collapse_non_adjacent_repeated_line_runs(lines);
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

    [
        "task graph completed",
        "passed worker reports",
        "incomplete nodes",
        "task graph",
        "role_output",
        "final_report",
        "cognition failure",
        "json template validation failed",
        "missing key",
        "worker reported failed status",
        "search_evidence_packet:",
        "search_evidence_trace:",
        "support role",
        "support worker",
        "support_workers",
        "task_id",
        "data role should provide",
        "service role output lacks",
        "engineering role output lacks",
        "skill:",
        "skill=",
        "adapter_status",
        "requested context refs",
        "product source product:",
        "knowledge source product_knowledge:",
        "commerce product context resolved",
        "local commerce rag context",
        "local commerce rag retrieval",
        "retrieved knowledge source",
        "context.selected_summary",
        "context.combined_summary",
        "commerce.selected_product_context",
        "productcontextsummary",
        "confidence=",
        "raw=",
        "fast-route",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
        || line.contains("任务图")
        || line.contains("岗位报告")
        || line.contains("Product source product")
        || line.contains("Knowledge source product_knowledge")
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
        || (lower.contains("product:") && contains_uuid_like(line))
        || (lower.contains(" | name=") && lower.contains(" | platform="))
        || lower.starts_with("worker")
        || line == "- none"
        || lower == "none"
        || is_empty_list_artifact(line)
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
        assert!(output.contains("services.toml"));
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
                    "资料依据：广告数据里点击率 CTR=1.2%、转化率 CVR=2.1%、退款率=18%，最大问题是退款率偏高，不是点击率不足。"
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
                    "资料依据：广告数据里点击率 CTR=1.2%、转化率 CVR=2.1%、退款率=18%，最大问题是退款率偏高。"
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
            "先看毛利，再看销量。\n- 补齐成本字段。\n资料边界：附件成本字段仍需核验。"
        );
    }

    #[test]
    fn filters_internal_context_prompt_leakage_from_final_answer() {
        let raw = "针对千乐咖啡，当前只确认了名称、售价和在售状态，缺少品类、平台和成本数据。\n\
                   Commerce product context resolved from current user's backend data. Use these facts when relevant, cite product or knowledge titles, and do not invent missing product details. Product source product:ce3f729c-a8bd-45ca-9859-7f2591d5973f | name=千乐咖啡 | platform=unknown | category=unknown | brand=unknown | price=100 CNY | status=active | description=适合于白领 Requested context refs: product:ce3f729c-a8bd-45ca-9859-7f2591d5973f\n\
                   建议先补真实成本、目标平台和用户反馈，再判断是否继续做活动。";

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
    fn rewrites_internal_evidence_terms_from_final_answer() {
        let raw = "QA矩阵智能保温杯适合做学生电商实训，因为商品锚点：温显、防漏、礼盒都清楚。\n\
                   资料边界：低置信参考只能作为补充，置信度0.96 的内容仍要用真实素材核对，confidence=0.72 不能直接展示给学生。\n\
                   建议先围绕宿舍、社团礼品和冬季热饮三个场景做短视频测试。";

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
        let raw = "同学们好，基于当前你手动引用的QA矩阵智能保温杯全部资料，现将这款商品适合和不适合做学生电商实训的地方明确梳理如下：【核心业务结论与定价判断】这款商品当前129元的售价完全符合不低于119元的定价安全线，定价合规性明确，整体规则边界清晰。【100%确定、可直接落地的适合实训环节】选品定位练习：商品有明确的客群边界，严格禁止学生党低价定位。内容合规练习：首屏文案必须包含温显、防漏、礼盒三个核心卖点。【当前资料缺失、暂不适合开展的实训环节】当前没有实际曝光、点击、成交、成本、退款等真实运营数据。【依据说明】确定部分的支撑来源：QA矩阵智能保温杯商品锚点、QA矩阵商品知识、QA矩阵卖点素材，置信度0.96。【下一步核验动作】你可以优先核验抖音平台智能保温杯类目的公开运营规则。";

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
        let raw = "先给结论：这款商品可以作为学生实训样例，但不能直接当成低价引流品。\n\n适合部分：商品卖点、价格边界和场景都比较清楚，可以练选品定位。\n\n依据说明：支撑来自QA矩阵智能保温杯商品锚点，置信度0.96。";

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
        let raw = "这款商品可以做实训，但要先限定任务范围。适合部分：围绕卖点、价格和场景做选品定位。不适合部分：没有真实投放数据时，不要做确定性ROI判断。核验动作：补成本、退款和平台规则资料。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("限定任务范围。\n\n适合部分：\n\n围绕卖点"));
        assert!(output.contains("选品定位。\n\n不适合部分：\n\n没有真实投放数据"));
        assert!(output.contains("ROI判断。\n\n核验动作：\n\n补成本"));
    }

    #[test]
    fn final_sanitizing_restores_inline_practice_headings() {
        let raw = "当前暂不适合做进阶判断。进阶数据类实训：没有曝光、点击、成交数据，暂不做转化测算。行业对比类实训：没有可采信公开来源，暂不做行业对标。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

        assert!(output.contains("进阶判断。\n\n进阶数据类实训：\n\n没有曝光"));
        assert!(output.contains("转化测算。\n\n行业对比类实训：\n\n没有可采信"));
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
        let raw = "当前只掌握商品名称和售价，先补成本、平台和用户反馈。\n\
                   skill:learning_teaching_explain status=success summary=教学解释已生成。\n\
                   用户本轮手动选择的预设商品资料：千乐咖啡（已选商品，来源 context.selected_summary）。\n\
                   后续再做毛利核算和转化分析。";

        let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("当前只掌握商品名称和售价"));
        assert!(output.contains("本轮已使用你选中的商品资料：千乐咖啡。"));
        assert!(output.contains("后续再做毛利核算"));
        assert!(!lower.contains("skill:"));
        assert!(!lower.contains("status="));
        assert!(!lower.contains("context.selected_summary"));
    }
}
