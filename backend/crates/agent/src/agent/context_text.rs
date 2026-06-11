use std::collections::BTreeMap;

pub(crate) fn user_visible_selected_product_context(raw: &str) -> Option<String> {
    let sections = user_visible_context_sections(raw);
    if sections.is_empty() {
        return None;
    }

    Some(format!(
        "当前手动引用的商品资料如下；回答“这个商品”“该商品”或单品分析时，以这些资料为准。\
         不要向用户展示商品ID、资料来源ID或内部字段。\n{}",
        sections.join("\n")
    ))
}

pub(crate) fn user_visible_context_summary(raw: &str) -> Option<String> {
    let sections = user_visible_context_sections(raw);
    if !sections.is_empty() {
        return Some(sections.join("\n"));
    }

    let lines = raw
        .lines()
        .filter_map(clean_context_line)
        .collect::<Vec<_>>();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

pub(crate) fn user_visible_conversation_history(raw: &str) -> Option<String> {
    let lines = raw
        .lines()
        .filter_map(clean_context_line)
        .collect::<Vec<_>>();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

fn user_visible_context_sections(raw: &str) -> Vec<String> {
    raw.lines()
        .filter_map(|line| {
            parse_product_source_line(line)
                .or_else(|| parse_knowledge_source_line(line))
                .or_else(|| clean_context_line(line))
        })
        .fold(Vec::<String>::new(), |mut sections, section| {
            if !sections.iter().any(|existing| existing == &section) {
                sections.push(section);
            }
            sections
        })
}

fn parse_product_source_line(line: &str) -> Option<String> {
    let line = line.trim();
    let rest = line.strip_prefix("Product source product:")?;
    let mut parts = rest.splitn(2, '|');
    let _id = parts.next()?.trim();
    let fields = parse_pipe_fields(parts.next().unwrap_or_default());
    let name = clean_value(fields.get("name")?)?;
    let mut details = vec![format!("商品：{name}")];

    push_detail(&mut details, "平台", fields.get("platform"));
    push_detail(&mut details, "类目", fields.get("category"));
    push_detail(&mut details, "品牌", fields.get("brand"));
    push_detail(&mut details, "售价", fields.get("price"));
    push_detail(&mut details, "说明", fields.get("description"));

    Some(format!("- {}", details.join("；")))
}

fn parse_knowledge_source_line(line: &str) -> Option<String> {
    let line = line.trim();
    let rest = line.strip_prefix("Knowledge source product_knowledge:")?;
    let mut parts = rest.splitn(2, '|');
    let _id = parts.next()?.trim();
    let fields = parse_pipe_fields(parts.next().unwrap_or_default());
    let title = fields
        .get("title")
        .and_then(|value| clean_value(value))
        .or_else(|| fields.get("type").and_then(|value| clean_value(value)))
        .unwrap_or_else(|| "补充资料".to_string());
    let content = fields
        .get("content")
        .and_then(|value| clean_knowledge_content(value))?;

    Some(format!("- 商品资料：{title}：{content}"))
}

fn parse_pipe_fields(value: &str) -> BTreeMap<String, String> {
    value
        .split('|')
        .filter_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn push_detail(details: &mut Vec<String>, label: &str, value: Option<&String>) {
    if let Some(value) = value.and_then(|value| clean_value(value)) {
        details.push(format!("{label}：{value}"));
    }
}

fn clean_value(value: &str) -> Option<String> {
    clean_text_value(value)
}

fn clean_knowledge_content(value: &str) -> Option<String> {
    let mut text = value.trim().to_string();
    for prefix in [
        "QA文件锚点：",
        "QA文件锚点:",
        "文件锚点：",
        "文件锚点:",
        "商品锚点：",
        "商品锚点:",
        "资料锚点：",
        "资料锚点:",
    ] {
        if let Some(rest) = text.strip_prefix(prefix) {
            text = rest.trim().to_string();
        }
    }
    clean_text_value(&text)
}

fn clean_context_line(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() || is_internal_context_line(line) {
        return None;
    }
    clean_text_value(line)
}

fn clean_text_value(value: &str) -> Option<String> {
    let mut text = value.trim().to_string();
    if text.is_empty() || text.eq_ignore_ascii_case("unknown") || text.eq_ignore_ascii_case("null")
    {
        return None;
    }

    for prefix in [
        "QA文件锚点：",
        "QA文件锚点:",
        "文件锚点：",
        "文件锚点:",
        "商品锚点：",
        "商品锚点:",
        "资料锚点：",
        "资料锚点:",
    ] {
        if let Some(rest) = text.strip_prefix(prefix) {
            text = rest.trim().to_string();
        }
    }

    let lower = text.to_ascii_lowercase();
    for marker in [
        "confidence=",
        "source=",
        "sourcemap",
        "source_map",
        "memorybudget",
        "memory_budget",
        "product source product:",
        "knowledge source product_knowledge:",
        "requested context refs:",
        "context.selected_summary",
        "context.combined_summary",
        "conversation.memorysummaryjson",
        "conversation.memory_summary_json",
        "worker.assignment.input",
        "assignment.input",
    ] {
        if let Some(index) = lower.find(marker) {
            text.truncate(index);
        }
    }

    let text = text
        .trim()
        .trim_matches(['|', '，', ',', '。', '.', '；', ';', '：', ':', ' '])
        .to_string();
    (!text.is_empty()).then_some(text)
}

fn is_internal_context_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "commerce product context resolved",
        "use these facts when relevant",
        "local commerce rag context",
        "retrieval query terms:",
        "retrieved knowledge source",
        "requested context refs:",
        "product source product:",
        "knowledge source product_knowledge:",
        "worker.assignment.input",
        "assignment.input",
        "context.selected_summary",
        "context.combined_summary",
        "conversation.memorysummaryjson",
        "conversation.memory_summary_json",
        "memorysummaryjson",
        "memorybudget",
        "memory_budget",
        "sourcemap",
        "source_map",
        "workspacecontext.",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::{
        user_visible_context_summary, user_visible_conversation_history,
        user_visible_selected_product_context,
    };

    #[test]
    fn selected_product_context_becomes_student_readable() {
        let raw = "\
Commerce product context resolved from current user's backend data.
Product source product:222 | name=QA矩阵智能保温杯 | platform=抖音 | category=智能保温杯 | \
                   brand=QA矩阵品牌 | price=129 CNY | status=active | \
                   description=面向办公室女性的中高端礼盒款
Knowledge source product_knowledge:333 | product=222 | title=QA矩阵商品知识 | type=selling_points \
                   | source=manual | confidence=0.96 | \
                   content=商品锚点：首屏必须包含温显、防漏、礼盒三个卖点";

        let output = user_visible_selected_product_context(raw).expect("context");

        assert!(output.contains("商品：QA矩阵智能保温杯"));
        assert!(output.contains("类目：智能保温杯"));
        assert!(output.contains("商品资料：QA矩阵商品知识：首屏必须包含温显、防漏、礼盒三个卖点"));
        assert!(!output.contains("Product source"));
        assert!(!output.contains("Knowledge source"));
        assert!(!output.contains("confidence"));
        assert!(!output.contains("置信度"));
        assert!(!output.contains("商品锚点"));
    }

    #[test]
    fn combined_context_filters_internal_lines() {
        let raw = "\
Same conversation history context:
用户前面问过校园冷萃咖啡。
Product source product:222 | name=QA矩阵智能保温杯 | category=智能保温杯 | brand=QA矩阵品牌
Knowledge source product_knowledge:333 | product=222 | title=卖点素材 | type=selling_points | \
                   source=manual | confidence=0.96 | content=资料锚点：温显、防漏、礼盒";

        let output = user_visible_context_summary(raw).expect("context");

        assert!(output.contains("用户前面问过校园冷萃咖啡"));
        assert!(output.contains("商品：QA矩阵智能保温杯"));
        assert!(output.contains("商品资料：卖点素材：温显、防漏、礼盒"));
        assert!(!output.contains("Product source"));
        assert!(!output.contains("confidence"));
        assert!(!output.contains("资料锚点"));
    }

    #[test]
    fn conversation_history_filters_memory_and_source_debug_lines() {
        let raw = "\
User: 上一轮直播间曝光8000、点击640、付款72。
conversation.memorySummaryJson={\"rolling_summary\":\"internal\"}
sourceMap={\"conversation\":\"debug\"}
worker.assignment.input={\"context\":{\"selected_summary\":\"商品A\"}}
Assistant: 已经判断主要问题是成交承接弱。
下次复盘字段：曝光、点击、加购、付款、退款率。";

        let output = user_visible_conversation_history(raw).expect("history");
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("上一轮直播间曝光8000"));
        assert!(output.contains("成交承接弱"));
        assert!(output.contains("下次复盘字段"));
        assert!(!lower.contains("memorysummaryjson"));
        assert!(!lower.contains("sourcemap"));
        assert!(!lower.contains("worker.assignment.input"));
        assert!(!lower.contains("selected_summary"));
    }
}
