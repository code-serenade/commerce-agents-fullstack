use serde_json::Value;

use crate::skills::SkillContext;

pub(super) fn normalized_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn metadata_text(context: &SkillContext, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| context.metadata.get(*key))
        .find_map(text_from_value)
        .and_then(|value| normalized_optional_text(Some(value)))
}

pub(super) fn metadata_refs(context: &SkillContext, keys: &[&str]) -> Vec<String> {
    let mut refs = Vec::new();
    for key in keys {
        let Some(value) = context.metadata.get(*key) else {
            continue;
        };
        collect_string_items(value, &mut refs);
    }
    refs
}

pub(super) fn metadata_usize(context: &SkillContext, keys: &[&str]) -> Option<usize> {
    keys.iter()
        .filter_map(|key| context.metadata.get(*key))
        .find_map(|value| match value {
            Value::Number(number) => number.as_u64().map(|value| value as usize),
            Value::String(text) => text.trim().parse::<usize>().ok(),
            _ => None,
        })
}

pub(super) fn infer_material_type(name: Option<&str>, provided: Option<&str>) -> String {
    if let Some(value) = provided.map(str::trim).filter(|value| !value.is_empty()) {
        let lower = value.to_ascii_lowercase();
        return match lower.as_str() {
            "pdf" | "word" | "doc" | "docx" | "excel" | "xls" | "xlsx" | "csv" | "image"
            | "img" | "png" | "jpg" | "jpeg" | "text" | "txt" | "markdown" | "md" => {
                normalize_material_type(&lower).to_owned()
            }
            _ => value.to_owned(),
        };
    }

    let Some(name) = name else {
        return "unknown".to_owned();
    };
    let lower = name.to_ascii_lowercase();
    let extension = lower
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .unwrap_or("");
    normalize_material_type(extension).to_owned()
}

pub(super) fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_owned();
    }
    let mut output = trimmed.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}

pub(super) fn meaningful_segments(text: &str, max_items: usize) -> Vec<String> {
    let mut segments = Vec::new();
    for raw in text
        .replace('\r', "\n")
        .split(['\n', '。', '；', ';'])
        .map(str::trim)
        .filter(|value| value.chars().count() >= 6)
    {
        push_unique(&mut segments, compact_text(raw, 180));
        if segments.len() >= max_items {
            break;
        }
    }
    segments
}

pub(super) fn question_terms(question: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let normalized = question.replace(
        ['，', '。', '？', '?', '：', ':', '、', ',', '.', ';', '；'],
        " ",
    );
    for token in normalized.split_whitespace() {
        let token = token.trim();
        let char_count = token.chars().count();
        if (2..=20).contains(&char_count) && !is_question_stop_word(token) {
            push_unique(&mut terms, token.to_owned());
        }
    }
    if terms.is_empty() {
        for marker in [
            "成本", "利润", "毛利", "价格", "销量", "库存", "转化", "人群", "卖点", "风险", "规则",
            "竞品", "复盘", "活动", "投放", "图片", "表格",
        ] {
            if question.contains(marker) {
                push_unique(&mut terms, marker.to_owned());
            }
        }
    }
    terms
}

pub(super) fn evidence_segments(text: &str, question: &str, max_items: usize) -> Vec<String> {
    let mut candidates = Vec::new();
    let terms = question_terms(question);
    let segments = meaningful_segments(text, 80);

    for segment in &segments {
        let relevant = !terms.is_empty() && terms.iter().any(|term| segment.contains(term));
        let has_number = segment.chars().any(|char| char.is_ascii_digit());
        if relevant || has_number {
            push_unique(&mut candidates, segment.clone());
        }
        if candidates.len() >= max_items {
            return candidates;
        }
    }

    for segment in segments {
        push_unique(&mut candidates, segment);
        if candidates.len() >= max_items {
            break;
        }
    }
    candidates
}

pub(super) fn material_scenarios(text: &str, material_type: &str) -> Vec<String> {
    let mut scenarios = Vec::new();
    let lower_type = material_type.to_ascii_lowercase();
    if matches!(lower_type.as_str(), "excel" | "csv" | "xlsx" | "xls") || looks_tabular(text) {
        scenarios
            .push("可用于经营指标、成本利润、活动复盘或库存表的字段识别与后续测算。".to_owned());
    }
    if text.contains("评价")
        || text.contains("差评")
        || text.contains("客服")
        || text.contains("售后")
    {
        scenarios.push("可用于客服问题归因、评价主题提炼和服务改进建议。".to_owned());
    }
    if text.contains("竞品")
        || text.contains("价格")
        || text.contains("卖点")
        || text.contains("标题")
    {
        scenarios.push("可用于竞品对比、商品卖点提炼、标题/详情页优化。".to_owned());
    }
    if text.contains("规则") || text.contains("合规") || text.contains("平台") {
        scenarios.push("可用于平台规则摘要、合规风险提示和操作边界整理。".to_owned());
    }
    if matches!(lower_type.as_str(), "image" | "png" | "jpg" | "jpeg") {
        scenarios.push(
            "可用于图片 OCR 文本理解；视觉构图、主体识别仍应交给图片诊断类 Skill 或视觉模型。"
                .to_owned(),
        );
    }
    if scenarios.is_empty() {
        scenarios.push(
            "可作为本轮 Agent 回答的资料依据，用于摘要、证据引用和后续岗位 Skill 输入。".to_owned(),
        );
    }
    scenarios
}

pub(super) fn missing_info(
    has_content: bool,
    has_question: bool,
    refs: &[String],
    material_type: &str,
) -> Vec<String> {
    let mut missing = Vec::new();
    if !has_content {
        missing.push(
            "缺少可读取的正文、OCR 文本或上游摘要；当前不能判断资料里的真实内容。".to_owned(),
        );
    }
    if !has_question {
        missing.push("缺少用户围绕资料提出的具体问题，只能做通用资料摘要。".to_owned());
    }
    if refs.is_empty() {
        missing.push("缺少显式资料引用，后续跨文件追踪和引用定位会变弱。".to_owned());
    }
    if material_type == "unknown" {
        missing.push("资料类型未知，无法判断是否应进入表格、图片或长文档专项处理。".to_owned());
    }
    missing
}

pub(super) fn risks(has_content: bool, material_type: &str, truncated: bool) -> Vec<String> {
    let mut risks = Vec::new();
    if !has_content {
        risks.push(
            "不能把文件名、资料标题或附件数量当成文件正文；回答必须标注“未读取到正文”。".to_owned(),
        );
    }
    if truncated {
        risks.push(
            "输入正文较长，当前只做截断后的初步理解，长文档应继续分段抽取或进入证据提取 Skill。"
                .to_owned(),
        );
    }
    if matches!(material_type, "image" | "png" | "jpg" | "jpeg") {
        risks.push(
            "图片资料如果只有 OCR 文本，不能据此断言完整视觉效果、构图质量或商品主体细节。"
                .to_owned(),
        );
    }
    if matches!(material_type, "excel" | "csv" | "xlsx" | "xls") {
        risks.push(
            "表格资料的字段类型、异常值和汇总方式需要表格剖析 Skill 或数据 Skill 继续核验。"
                .to_owned(),
        );
    }
    if risks.is_empty() {
        risks.push(
            "当前结论只基于本轮传入资料，不代表实时外部市场、平台规则或店铺后台数据。".to_owned(),
        );
    }
    risks
}

fn text_from_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => {
            let values = items.iter().filter_map(text_from_value).collect::<Vec<_>>();
            (!values.is_empty()).then(|| values.join("\n"))
        }
        Value::Object(object) => {
            for key in [
                "text",
                "content",
                "summary",
                "combinedSummary",
                "combined_summary",
                "selectedSummary",
                "selected_summary",
                "agentContext",
                "agent_context",
            ] {
                if let Some(text) = object.get(key).and_then(text_from_value) {
                    return Some(text);
                }
            }
            None
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => None,
    }
}

fn collect_string_items(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                collect_string_items(&parsed, out);
            } else {
                for item in text.split([',', '|']) {
                    push_unique(out, item.trim().to_owned());
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_string_items(item, out);
            }
        }
        Value::Object(object) => {
            for key in ["ref", "sourceRef", "id", "fileId", "title", "name"] {
                if let Some(value) = object.get(key) {
                    collect_string_items(value, out);
                }
            }
        }
        Value::Number(number) => push_unique(out, number.to_string()),
        Value::Bool(value) => push_unique(out, value.to_string()),
        Value::Null => {}
    }
}

fn normalize_material_type(value: &str) -> &str {
    match value {
        "pdf" => "pdf",
        "doc" | "docx" | "word" => "word",
        "xls" | "xlsx" | "xlsm" | "xlsb" | "csv" | "excel" => "excel",
        "png" | "jpg" | "jpeg" | "webp" | "image" | "img" => "image",
        "txt" | "text" | "md" | "markdown" | "json" => "text",
        _ => "unknown",
    }
}

fn looks_tabular(text: &str) -> bool {
    text.lines()
        .take(20)
        .filter(|line| line.matches(',').count() >= 2 || line.matches('\t').count() >= 2)
        .count()
        >= 2
}

fn is_question_stop_word(value: &str) -> bool {
    matches!(
        value,
        "请" | "帮我"
            | "分析"
            | "一下"
            | "这个"
            | "这些"
            | "资料"
            | "文件"
            | "上传"
            | "看看"
            | "什么"
            | "怎么"
            | "为什么"
            | "如何"
    )
}

fn push_unique(out: &mut Vec<String>, value: String) {
    let value = value.trim();
    if value.is_empty() || out.iter().any(|item| item == value) {
        return;
    }
    out.push(value.to_owned());
}

#[cfg(test)]
mod tests {
    use super::{evidence_segments, infer_material_type, metadata_refs};
    use crate::skills::SkillContext;

    #[test]
    fn infers_common_material_types_from_names() {
        assert_eq!(infer_material_type(Some("review.xlsx"), None), "excel");
        assert_eq!(infer_material_type(Some("brief.PDF"), None), "pdf");
        assert_eq!(infer_material_type(Some("photo.png"), None), "image");
    }

    #[test]
    fn evidence_prefers_question_terms_and_numbers() {
        let text = "商品A成本 12 元，售价 29 元。\n客服评价集中在发货慢。\n活动页点击率偏低。";
        let evidence = evidence_segments(text, "请分析成本利润", 2);

        assert!(evidence[0].contains("成本"));
    }

    #[test]
    fn parses_metadata_refs_from_json_string() {
        let context = SkillContext::new().with_metadata(
            "request.context.refs",
            serde_json::json!(r#"["conversation_file:file-1","workspace_source:file-2"]"#),
        );

        let refs = metadata_refs(&context, &["request.context.refs"]);

        assert_eq!(
            refs,
            vec!["conversation_file:file-1", "workspace_source:file-2"]
        );
    }
}
