use super::{context, domain, metrics, numbers, text::compact_task_text};
use crate::agent::worker::{Task, runner::policy::worker_policies};

pub(in crate::agent::worker::runner) fn query_for_task(text: &str, task: &Task) -> String {
    if refers_to_selected_product(text)
        && let Some(product) = context::first_selected_product(task)
    {
        if let Some(category) = product.category().filter(|value| !value.is_empty()) {
            return format!("{} {}", product.name(), category);
        }
        return product.name().to_string();
    }
    infer_query(text)
}

pub(in crate::agent::worker::runner) fn policy_query_for_task(text: &str, task: &Task) -> String {
    let content = policy_source_text(text, task);
    let platform = platform_for_task(&content, task);
    let keywords = extract_keyword_candidates(&content)
        .into_iter()
        .filter(|keyword| keyword != &platform)
        .take(6)
        .collect::<Vec<_>>();
    if keywords.is_empty() {
        return infer_query(&content);
    }
    keywords.join(" ")
}

pub(in crate::agent::worker::runner) fn industry_for_task(text: &str, task: &Task) -> String {
    if refers_to_selected_product(text)
        && let Some(product) = context::first_selected_product(task)
        && let Some(category) = product.category().filter(|value| !value.is_empty())
    {
        return category.to_string();
    }
    domain::infer_category(text)
}

pub(in crate::agent::worker::runner) fn platform_for_task(text: &str, task: &Task) -> String {
    let explicit_platform = infer_platform(text);
    if explicit_platform != config_string("/platform/generic_label", "电商") {
        return explicit_platform;
    }
    context::first_selected_product(task)
        .and_then(|product| product.platform().map(str::to_owned))
        .filter(|platform| !platform.is_empty())
        .unwrap_or(explicit_platform)
}

pub(in crate::agent::worker::runner) fn bypass_cache(text: &str) -> bool {
    contains_any(
        text,
        &config_string_list("/freshness/bypass_cache_keywords"),
    )
}

pub(in crate::agent::worker::runner) fn max_results(text: &str) -> Option<i64> {
    for (start, ch) in text.char_indices() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if start > 0
            && text[..start]
                .chars()
                .last()
                .is_some_and(|prev| prev.is_ascii_digit() || prev == '.')
        {
            continue;
        }

        let mut end = start + ch.len_utf8();
        let suffix_start = end;
        for (offset, next) in text[suffix_start..].char_indices() {
            if next.is_ascii_digit() {
                end = suffix_start + offset + next.len_utf8();
                continue;
            }
            break;
        }

        let Ok(value) = text[start..end].parse::<i64>() else {
            continue;
        };
        if value <= 0 {
            continue;
        }
        let before = text[..start]
            .chars()
            .rev()
            .take(config_usize("/max_results/scan_before_chars", 16))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        let after = text[end..]
            .chars()
            .take(config_usize("/max_results/scan_after_chars", 16))
            .collect::<String>();

        if is_result_count_context(&before, &after) {
            return Some(value.clamp(
                config_i64("/max_results/min", 1),
                config_i64("/max_results/max", 12),
            ));
        }
    }
    configured_strength_max_results(text)
}

pub(in crate::agent::worker::runner) fn max_results_for_task(
    text: &str,
    task: &Task,
) -> Option<i64> {
    max_results(text).or_else(|| configured_response_mode_max_results(task))
}

pub(in crate::agent::worker::runner) fn policy_type(text: &str) -> &'static str {
    match_configured_label(text, "policy_type", "general")
}

pub(in crate::agent::worker::runner) fn policy_type_for_task(
    text: &str,
    task: &Task,
) -> &'static str {
    let content = policy_source_text(text, task);
    let label = policy_type(&content);
    if label != "general" {
        return label;
    }
    policy_type(text)
}

pub(in crate::agent::worker::runner) fn review_focus(text: &str) -> &'static str {
    match_configured_label(text, "review_focus", "overall")
}

pub(in crate::agent::worker::runner) fn competitor_focus(text: &str) -> &'static str {
    match_configured_label(text, "competitor_focus", "general")
}

pub(in crate::agent::worker::runner) fn benchmark_metric(text: &str) -> &'static str {
    match_configured_label(text, "benchmark_metric", "general")
}

pub(in crate::agent::worker::runner) fn topic(text: &str) -> &'static str {
    match_configured_label(text, "topic", "general")
}

pub(in crate::agent::worker::runner) fn days(text: &str) -> i64 {
    numbers::infer_period_days(text)
}

fn infer_platform(text: &str) -> String {
    crate::agent::worker::runner::policy::worker_policies()
        .inference
        .match_label(
            text,
            &config_string("/platform/rule_set", "platform.search"),
        )
        .unwrap_or(config_string("/platform/generic_label", "电商").as_str())
        .to_string()
}

fn infer_query(text: &str) -> String {
    let keywords = extract_keyword_candidates(text);
    if let Some(keyword) = keywords.first() {
        return keyword.clone();
    }
    let category = domain::infer_category(text);
    if category != config_string("/query/generic_category_label", "电商消费品") {
        return category;
    }
    compact_task_text(text)
}

fn policy_source_text(text: &str, task: &Task) -> String {
    if let Some(root_task) = task
        .assignment
        .as_ref()
        .map(|assignment| assignment.root_task.trim())
        .filter(|value| !value.is_empty())
    {
        return root_task.to_string();
    }
    let description = task.description.trim();
    if !description.is_empty() {
        return description.to_string();
    }
    text.trim().to_string()
}

fn extract_keyword_candidates(text: &str) -> Vec<String> {
    let min_chars = config_usize("/query/candidate_min_chars", 2);
    let max_chars = config_usize("/query/candidate_max_chars", 24);
    let exclude_keywords = config_string_list("/query/candidate_exclude_keywords");
    let delimiters = split_delimiters("/query/candidate_split_delimiters");

    text.split(|ch| delimiters.contains(&ch))
        .filter_map(|value| clean_keyword_candidate(value, &exclude_keywords))
        .filter(|value| value.chars().count() >= min_chars && value.chars().count() <= max_chars)
        .collect()
}

fn clean_keyword_candidate(value: &str, exclude_keywords: &[String]) -> Option<String> {
    let mut candidate = value.trim().to_string();
    for keyword in exclude_keywords {
        let keyword = keyword.trim();
        if !keyword.is_empty() {
            candidate = replace_keyword_case_insensitive(&candidate, keyword, " ");
        }
    }
    let candidate = candidate
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '，' | ','
                        | '：'
                        | ':'
                        | '、'
                        | '；'
                        | ';'
                        | '。'
                        | '.'
                        | '！'
                        | '!'
                        | '？'
                        | '?'
                )
        })
        .trim()
        .to_string();
    (!candidate.is_empty()).then_some(candidate)
}

fn replace_keyword_case_insensitive(input: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return input.to_string();
    }

    let lower_input = input.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();
    let mut output = String::new();
    let mut cursor = 0;
    while let Some(relative) = lower_input[cursor..].find(&lower_needle) {
        let start = cursor + relative;
        let end = start + needle.len();
        output.push_str(&input[cursor..start]);
        output.push_str(replacement);
        cursor = end;
    }
    output.push_str(&input[cursor..]);
    output
}

fn refers_to_selected_product(text: &str) -> bool {
    contains_any(
        text,
        &config_string_list("/query/selected_product_triggers"),
    )
}

fn is_result_count_context(before: &str, after: &str) -> bool {
    let trimmed_after = after.trim_start();
    let has_after_result_unit = starts_with_any(
        trimmed_after,
        &config_string_list("/max_results/after_result_units"),
    );
    let has_before_limit_word = contains_any(
        before,
        &config_string_list("/max_results/before_limit_keywords"),
    );
    let has_after_short_unit = starts_with_any(
        trimmed_after,
        &config_string_list("/max_results/after_short_units"),
    );

    has_after_result_unit || (has_before_limit_word && has_after_short_unit)
}

fn configured_strength_max_results(text: &str) -> Option<i64> {
    let rules = worker_policies()
        .inference
        .section("search")?
        .pointer("/max_results/strength_defaults")?
        .as_array()?;
    rules.iter().find_map(|rule| {
        let keywords = rule
            .get("keywords")
            .and_then(serde_json::Value::as_array)?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        if !contains_any(text, &keywords) {
            return None;
        }
        rule.get("max_results")
            .and_then(serde_json::Value::as_i64)
            .map(|value| {
                value.clamp(
                    config_i64("/max_results/min", 1),
                    config_i64("/max_results/max", 12),
                )
            })
    })
}

fn configured_response_mode_max_results(task: &Task) -> Option<i64> {
    let response_mode = task_response_mode(task)?;
    let rules = worker_policies()
        .inference
        .section("search")?
        .pointer("/max_results/response_mode_defaults")?
        .as_array()?;
    rules.iter().find_map(|rule| {
        let configured_mode = rule
            .get("response_mode")
            .and_then(serde_json::Value::as_str)?;
        if configured_mode != response_mode {
            return None;
        }
        rule.get("max_results")
            .and_then(serde_json::Value::as_i64)
            .map(|value| {
                value.clamp(
                    config_i64("/max_results/min", 1),
                    config_i64("/max_results/max", 12),
                )
            })
    })
}

fn task_response_mode(task: &Task) -> Option<&str> {
    task.context
        .metadata
        .get("strategy.response_mode")
        .or_else(|| task.context.metadata.get("strategy.id"))
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
}

fn match_configured_label(text: &str, field: &str, fallback: &'static str) -> &'static str {
    let rule_set = config_string(&format!("/label_fields/{field}/rule_set"), "");
    let default = config_string(&format!("/label_fields/{field}/default"), fallback);
    worker_policies()
        .inference
        .match_label(text, &rule_set)
        .map(str::to_string)
        .unwrap_or(default)
        .leak()
}

fn contains_any<T: AsRef<str>>(text: &str, needles: &[T]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles.iter().any(|needle| {
        let needle = needle.as_ref();
        text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
    })
}

fn starts_with_any<T: AsRef<str>>(text: &str, prefixes: &[T]) -> bool {
    prefixes
        .iter()
        .any(|prefix| text.starts_with(prefix.as_ref()))
}

fn config_string(pointer: &str, fallback: &str) -> String {
    worker_policies()
        .inference
        .string_or("search", pointer, fallback)
}

fn config_string_list(pointer: &str) -> Vec<String> {
    worker_policies().inference.string_list("search", pointer)
}

fn config_i64(pointer: &str, fallback: i64) -> i64 {
    worker_policies()
        .inference
        .i64_or("search", pointer, fallback)
}

fn config_usize(pointer: &str, fallback: usize) -> usize {
    config_i64(pointer, fallback as i64).max(0) as usize
}

fn split_delimiters(pointer: &str) -> Vec<char> {
    config_string_list(pointer)
        .iter()
        .filter_map(|value| {
            let mut chars = value.chars();
            let ch = chars.next()?;
            chars.next().is_none().then_some(ch)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        agent::worker::{Task, runner::inference::search},
        core::protocol::{AgentId, MessageContext},
    };

    fn task(description: &str) -> Task {
        Task {
            id: "task-1".to_string(),
            context: MessageContext::default(),
            description: description.to_string(),
            requester: AgentId("commander".to_string()),
            assignment: None,
        }
    }

    fn task_with_response_mode(description: &str, response_mode: &str) -> Task {
        let mut task = task(description);
        task.context.metadata.insert(
            "strategy.response_mode".to_string(),
            response_mode.to_string(),
        );
        task
    }

    fn graph_task(root_task: &str, objective: &str) -> Task {
        use crate::core::protocol::{TaskGraphId, TaskId, TaskNodeId, WorkerAssignment, WorkerId};

        Task {
            id: "task-1".to_string(),
            context: MessageContext::default(),
            description: objective.to_string(),
            requester: AgentId("commander".to_string()),
            assignment: Some(WorkerAssignment {
                graph_id: TaskGraphId("graph-1".to_string()),
                node_id: TaskNodeId("ops_search".to_string()),
                task_id: TaskId("task-1".to_string()),
                worker_id: WorkerId("worker.ops".to_string()),
                role: "ops".to_string(),
                attempt: 1,
                root_task: root_task.to_string(),
                objective: objective.to_string(),
                llm_usage: None,
                skill_autosuggest: None,
                timeout_sec: None,
                inputs: Vec::new(),
                rework_instruction: None,
            }),
        }
    }

    #[test]
    fn policy_query_uses_task_description_instead_of_worker_objective() {
        let task = task(
            "请联网搜索并给出来源链接：最近30天抖音电商关于价格合规、低价引流、\
             虚假宣传的官方公开规则或公告；只输出来源标题、链接、发布日期和适用风险点。",
        );
        let query = search::policy_query_for_task(
            "通过搜索 skill \
             获取用户要求的公开来源证据，并按用户要求输出来源标题、链接、发布日期和适用风险点。",
            &task,
        );

        assert!(query.contains("价格合规"));
        assert!(query.contains("低价引流"));
        assert!(query.contains("虚假宣传"));
        assert!(!query.contains("通过搜索 skill"));
        assert!(!query.contains("按用户要求输出"));
    }

    #[test]
    fn policy_type_uses_task_description_before_worker_objective() {
        let task = task("抖音电商价格合规、低价引流、虚假宣传官方规则");
        let label = search::policy_type_for_task("搜索算法 流量规则", &task);

        assert_eq!(label, "content");
    }

    #[test]
    fn policy_type_prefers_specific_policy_keywords_over_generic_search_word() {
        let task = task(
            "请联网搜索并给出来源链接：最近30天抖音电商关于价格合规、低价引流、\
             虚假宣传的官方公开规则或公告。",
        );
        let label = search::policy_type_for_task("", &task);

        assert_eq!(label, "content");
    }

    #[test]
    fn policy_query_uses_graph_root_task_before_node_objective() {
        let task = graph_task(
            "最近30天抖音电商价格合规、低价引流、虚假宣传官方规则",
            "通过搜索 skill \
             获取用户要求的公开来源证据，并按用户要求输出来源标题、链接、发布日期和适用风险点。",
        );
        let query = search::policy_query_for_task(
            "通过搜索 skill \
             获取用户要求的公开来源证据，并按用户要求输出来源标题、链接、发布日期和适用风险点。",
            &task,
        );

        assert!(query.contains("价格合规"));
        assert!(!query.contains("通过搜索 skill"));
    }

    #[test]
    fn query_for_task_preserves_specific_competitor_terms_before_category() {
        let task = task("请搜索并对比校园冷萃咖啡竞品定价");
        let query = search::query_for_task(&task.description, &task);

        assert!(query.contains("校园冷萃咖啡"));
        assert!(query.contains("竞品定价"));
        assert_ne!(query, "咖啡饮品");
    }

    #[test]
    fn max_results_uses_configured_search_strength_when_no_explicit_count() {
        assert_eq!(
            search::max_results("请联网搜索校园冷萃咖啡竞品价格"),
            Some(6)
        );
        assert_eq!(search::max_results("请深度搜索并整理公开资料"), Some(10));
        assert_eq!(search::max_results("简单搜一下咖啡趋势"), Some(3));
    }

    #[test]
    fn max_results_for_task_uses_response_mode_default_without_explicit_strength() {
        assert_eq!(
            search::max_results_for_task(
                "请搜索校园冷萃咖啡消费趋势",
                &task_with_response_mode("请搜索校园冷萃咖啡消费趋势", "quick")
            ),
            Some(3)
        );
        assert_eq!(
            search::max_results_for_task(
                "请搜索校园冷萃咖啡消费趋势",
                &task_with_response_mode("请搜索校园冷萃咖啡消费趋势", "execution")
            ),
            Some(6)
        );
    }

    #[test]
    fn max_results_for_task_prefers_explicit_strength_over_response_mode_default() {
        let task = task_with_response_mode("请深度搜索校园冷萃咖啡消费趋势", "quick");

        assert_eq!(
            search::max_results_for_task(&task.description, &task),
            Some(10)
        );
    }
}
