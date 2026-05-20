use crate::core::protocol::WorkerReport;

pub(crate) fn user_facing_report_text(report: &WorkerReport) -> Option<String> {
    report
        .role_output
        .as_ref()
        .and_then(|output| {
            std::iter::once(output.summary.as_str())
                .chain(output.recommendations.iter().map(String::as_str))
                .chain(output.findings.iter().map(String::as_str))
                .find_map(sanitize_user_facing_text)
        })
        .or_else(|| sanitize_user_facing_text(&report.content))
}

pub(crate) fn sanitize_user_facing_text(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let text = remove_placeholder_phrases(&remove_placeholder_brackets(raw));
    let mut lines = Vec::new();
    for line in text.lines() {
        let line = clean_user_facing_line(line);
        if line.is_empty() || is_internal_noise_line(&line) {
            continue;
        }
        lines.push(line);
    }

    let output = normalize_blank_lines(lines);
    (!output.is_empty() && !is_internal_noise_line(&output)).then_some(output)
}

pub(crate) fn no_user_facing_content_message() -> String {
    "这次没有拿到可直接展示的完整内容。请补充更明确的问题或关键材料，我再继续整理。".to_string()
}

fn clean_user_facing_line(line: &str) -> String {
    let line = strip_role_prefix(line.trim());
    let line = rewrite_note_prefix(&line);
    let line = compact_line_spacing(&line);
    if is_empty_list_artifact(&line) {
        String::new()
    } else {
        line
    }
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

    normalized.join("\n")
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
        "support role",
        "support worker",
        "support_workers",
        "task_id",
        "data role should provide",
        "service role output lacks",
        "engineering role output lacks",
        "raw=",
        "fast-route",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
        || line.contains("任务图")
        || line.contains("岗位报告")
        || line.starts_with("任务已完成")
        || line.starts_with("支持角色已完成")
        || line.contains("正在思考中")
        || line.contains("用户未提供具体问题")
        || line.contains("占位符")
        || lower.contains(" worker")
        || lower.starts_with("worker")
        || line == "- none"
        || lower == "none"
        || is_empty_list_artifact(line)
}

fn is_empty_list_artifact(line: &str) -> bool {
    matches!(
        line.trim(),
        "-" | "*" | "•" | "-：" | "*：" | "•：" | "-:" | "*:" | "•:"
    )
}

#[cfg(test)]
mod tests {
    use super::sanitize_user_facing_text;

    #[test]
    fn sanitizes_placeholder_and_note_language() {
        let raw = "内容创意(creative): 【可直接测试的方案（占位符需替换为真实信息）】建议突出【真实品类，如挂耳/速溶/胶囊】和【真实规格/到手价】。\n备注：这是教学模式说明。";

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
        let raw = "Task graph completed.\nPassed worker reports:\n- creative(worker.creative): raw={\"role_output\":{}}\n真正可用的建议：先补商品信息。";

        let output = sanitize_user_facing_text(raw).expect("valid line should remain");
        let lower = output.to_ascii_lowercase();

        assert_eq!(output, "真正可用的建议：先补商品信息。");
        assert!(!lower.contains("task graph"));
        assert!(!lower.contains("worker"));
        assert!(!lower.contains("role_output"));
        assert!(!lower.contains("raw="));
    }

    #[test]
    fn removes_empty_role_shell_and_worker_artifacts() {
        let raw = "Task graph completed. Task: 这个咖啡如何做营销\nPassed worker reports\n- 内容创意(creative): 【可直接测试的千乐咖啡营销创意（占位符需替换为真实信息）】\n1. 主推标题：千乐咖啡【真实品类（如挂耳/速溶/胶囊/咖啡豆）】日常口粮款\n备注：替换真实品类后即可上线，测试核心指标参考点击率";

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
}
