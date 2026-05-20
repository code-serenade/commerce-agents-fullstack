use crate::core::protocol::RoleWorkOutput;

pub(super) fn normalize_user_facing_text(output: &mut RoleWorkOutput) {
    output.summary = normalize_text(&output.summary);
    normalize_items(&mut output.findings);
    normalize_items(&mut output.recommendations);
    normalize_items(&mut output.evidence);
    normalize_items(&mut output.risks);
    normalize_items(&mut output.open_questions);
}

fn normalize_items(items: &mut [String]) {
    for item in items {
        *item = normalize_text(item);
    }
}

fn normalize_text(value: &str) -> String {
    let mut text = normalize_markdown_surface(value.replace('\r', " "));
    text = remove_parenthetical_examples(&text);
    text = normalize_template_phrases(text);

    for (from, to) in [
        ("$\\rightarrow$", "→"),
        ("$ ightarrow$", "→"),
        ("$ightarrow$", "→"),
        ("\\rightarrow", "→"),
        ("$\\to$", "→"),
        ("\\to", "→"),
        ("$\\leftarrow$", "←"),
        ("$ leftarrow$", "←"),
        ("$leftarrow$", "←"),
        ("\\leftarrow", "←"),
        ("$\\Rightarrow$", "⇒"),
        ("$ Rightarrow$", "⇒"),
        ("$Rightarrow$", "⇒"),
        ("\\Rightarrow", "⇒"),
        ("$\\Leftarrow$", "⇐"),
        ("$ Leftarrow$", "⇐"),
        ("$Leftarrow$", "⇐"),
        ("\\Leftarrow", "⇐"),
    ] {
        text = text.replace(from, to);
    }

    compact_arrow_spacing(text)
}

fn normalize_markdown_surface(text: String) -> String {
    let text = text.replace("**", "").replace("__", "");
    let mut normalized = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            continue;
        }

        let line = normalize_markdown_line(line);
        if should_drop_half_finished_line(&line) {
            continue;
        }
        normalized.push(line);
    }

    compact_blank_lines(normalized)
}

fn normalize_markdown_line(line: &str) -> String {
    let line = strip_heading_marker(line.trim());
    let line = strip_blockquote_marker(line);
    let line = normalize_teaching_label(line);
    if let Some(content) = strip_ordered_marker(&line) {
        return format!("- {content}");
    }
    line
}

fn strip_heading_marker(line: &str) -> &str {
    let trimmed = line.trim_start_matches('#').trim_start();
    if trimmed.len() != line.len() && !trimmed.is_empty() {
        trimmed
    } else {
        line
    }
}

fn strip_blockquote_marker(line: &str) -> &str {
    line.strip_prefix('>').map(str::trim_start).unwrap_or(line)
}

fn strip_ordered_marker(line: &str) -> Option<&str> {
    let chars = line.char_indices();
    let mut marker_end = None;
    let mut saw_digit = false;

    for (index, ch) in chars {
        if ch.is_ascii_digit() {
            saw_digit = true;
            continue;
        }
        if saw_digit && matches!(ch, '.' | '、' | ')') {
            let end = index + ch.len_utf8();
            if ch != '.' || line[end..].chars().next().is_some_and(char::is_whitespace) {
                marker_end = Some(end);
            }
        }
        break;
    }

    if marker_end.is_none() {
        let mut iter = line.char_indices();
        if let Some((index, ch)) = iter.next()
            && matches!(
                ch,
                '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十'
            )
            && let Some((_, next)) = iter.next()
            && matches!(next, '、' | ')')
        {
            marker_end = Some(index + ch.len_utf8() + next.len_utf8());
        }
    }

    let end = marker_end?;
    let content = line[end..].trim_start();
    (!content.is_empty()).then_some(content)
}

fn normalize_teaching_label(line: &str) -> String {
    for prefix in ["备注：", "备注:"] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return format!("说明：{}", rest.trim_start());
        }
    }
    line.to_string()
}

fn should_drop_half_finished_line(line: &str) -> bool {
    let text = line.trim();
    if text.is_empty() {
        return false;
    }

    let lower = text.to_ascii_lowercase();
    lower.contains("placeholder")
        || lower.contains("todo")
        || text.contains("占位符")
        || text.contains("请替换")
        || text.contains("替换为真实")
        || text.contains("{{")
        || text.contains("}}")
}

fn compact_blank_lines(lines: Vec<String>) -> String {
    let mut output = Vec::new();
    let mut previous_blank = true;

    for line in lines {
        let is_blank = line.trim().is_empty();
        if is_blank && previous_blank {
            continue;
        }
        previous_blank = is_blank;
        output.push(line);
    }

    while output.last().is_some_and(|line| line.trim().is_empty()) {
        output.pop();
    }

    output.join("\n")
}

fn remove_parenthetical_examples(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0;

    while index < chars.len() {
        let Some(close) = closing_bracket(chars[index]) else {
            output.push(chars[index]);
            index += 1;
            continue;
        };

        let Some(end) = chars[index + 1..]
            .iter()
            .position(|ch| *ch == close)
            .map(|offset| index + 1 + offset)
        else {
            output.push(chars[index]);
            index += 1;
            continue;
        };

        let content = chars[index + 1..end]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        if is_example_or_placeholder_bracket(&content) {
            index = end + 1;
            continue;
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}

fn closing_bracket(ch: char) -> Option<char> {
    match ch {
        '(' => Some(')'),
        '（' => Some('）'),
        '[' => Some(']'),
        '【' => Some('】'),
        _ => None,
    }
}

fn is_example_or_placeholder_bracket(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }

    content.starts_with("如")
        || content.starts_with("例如")
        || content.starts_with("比如")
        || content.contains("占位")
        || content.contains("替换")
        || content.contains("请填写")
        || content.eq_ignore_ascii_case("xxx")
        || content.eq_ignore_ascii_case("xx")
        || is_common_placeholder(content)
}

fn is_common_placeholder(content: &str) -> bool {
    matches!(
        content,
        "商品名称"
            | "商品名"
            | "产品名称"
            | "品牌名称"
            | "价格"
            | "售价"
            | "链接"
            | "店铺名"
            | "平台"
            | "姓名"
            | "联系方式"
    )
}

fn normalize_template_phrases(mut text: String) -> String {
    for (from, to) in [
        ("比如从", "从"),
        ("例如从", "从"),
        ("譬如从", "从"),
        ("，比如", "，可以按真实资料补充"),
        ("，例如", "，可以按真实资料补充"),
        ("，譬如", "，可以按真实资料补充"),
        ("比如", "可以按真实资料补充"),
        ("例如", "可以按真实资料补充"),
        ("譬如", "可以按真实资料补充"),
        ("占位符需替换为真实信息", ""),
    ] {
        text = text.replace(from, to);
    }
    text
}

fn compact_arrow_spacing(mut text: String) -> String {
    for arrow in ["→", "←", "⇒", "⇐"] {
        for from in [
            format!("  {arrow}  "),
            format!(" {arrow}  "),
            format!("  {arrow} "),
        ] {
            text = text.replace(&from, &format!(" {arrow} "));
        }
    }
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::{normalize_text, normalize_user_facing_text};
    use crate::core::protocol::RoleWorkOutput;

    #[test]
    fn repairs_json_decoded_latex_arrow() {
        let text = "流量导入 $\rightarrow$ 高效转化";

        assert_eq!(normalize_text(text), "流量导入 → 高效转化");
    }

    #[test]
    fn repairs_visible_broken_latex_arrow() {
        let text = "数据指标 $ ightarrow$ 内容创意 $ ightarrow$ 视觉承接";

        assert_eq!(normalize_text(text), "数据指标 → 内容创意 → 视觉承接");
    }

    #[test]
    fn keeps_normal_text_unchanged() {
        let text = "当前方案进入产品特有级执行。";

        assert_eq!(normalize_text(text), text);
    }

    #[test]
    fn normalizes_all_user_facing_role_output_fields() {
        let mut output = RoleWorkOutput {
            summary: "流量导入 $\rightarrow$ 高效转化".to_string(),
            findings: vec!["复购 $ ightarrow$ 私域".to_string()],
            recommendations: vec!["曝光 \\rightarrow 转化".to_string()],
            evidence: vec!["任务输入".to_string()],
            risks: vec!["预算 $\\to$ 消耗需要复核".to_string()],
            open_questions: vec!["是否需要从预热 \\Rightarrow 爆发？".to_string()],
        };

        normalize_user_facing_text(&mut output);

        assert_eq!(output.summary, "流量导入 → 高效转化");
        assert_eq!(output.findings, vec!["复购 → 私域"]);
        assert_eq!(output.recommendations, vec!["曝光 → 转化"]);
        assert_eq!(output.evidence, vec!["任务输入"]);
        assert_eq!(output.risks, vec!["预算 → 消耗需要复核"]);
        assert_eq!(output.open_questions, vec!["是否需要从预热 ⇒ 爆发？"]);
    }

    #[test]
    fn removes_markdown_surface_markers() {
        let text = "### 相机租赁服务起步方案\n1. **服务清单**：按真实设备清单写";

        assert_eq!(
            normalize_text(text),
            "相机租赁服务起步方案\n- 服务清单：按真实设备清单写"
        );
    }

    #[test]
    fn removes_chinese_ordered_markers() {
        let text = "一、先看字段\n二、再补口径";

        assert_eq!(normalize_text(text), "- 先看字段\n- 再补口径");
    }

    #[test]
    fn removes_half_finished_surface_text() {
        let text = "备注：这是一段教学说明\n占位符需替换为真实信息\n```json\n正文\n```";

        assert_eq!(normalize_text(text), "说明：这是一段教学说明\n正文");
    }

    #[test]
    fn removes_parenthetical_examples_without_dropping_decimals() {
        let text = "避开无证据功效（如提神、减肥），保留1.5折这种真实表达。";

        assert_eq!(
            normalize_text(text),
            "避开无证据功效，保留1.5折这种真实表达。"
        );
    }
}
