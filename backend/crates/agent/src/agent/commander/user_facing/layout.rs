use super::{first_colon, is_cjk, normalize_blank_lines};

pub(super) fn restore_readable_answer_layout(text: &str) -> String {
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
