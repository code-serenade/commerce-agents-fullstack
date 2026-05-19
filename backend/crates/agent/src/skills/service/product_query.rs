use super::*;

pub struct ServiceQueryProduct {
    spec: SkillSpec,
}

impl ServiceQueryProduct {
    pub fn new() -> Self {
        Self {
            spec: service_query_product_spec(),
        }
    }
}

impl Default for ServiceQueryProduct {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceQueryProduct {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let query_product_ref = optional_string_param(&params, "product_ref")?;
        let query_product_id = optional_product_id_param(&params, "product_id")?;
        let query_product_name = optional_string_param(&params, "product_name")?;
        let product_context = explicit_product_context(&context.metadata);

        if !product_context.has_explicit_context() {
            return outcome(
                json!({
                    "status": "deferred",
                    "has_data": false,
                    "adapter_status": "context_required",
                    "data_readiness": "explicit_product_context_required",
                    "summary": "未发现本轮显式选择的商品上下文",
                    "skill": self.spec.id,
                    "query": {
                        "product_ref": query_product_ref,
                        "product_id": query_product_id,
                        "product_name": query_product_name,
                    },
                    "deferred_reason": "当前 Agent 上下文没有显式选择的商品 ID、商品引用、商品摘要或商品来源；为避免默认注入错误商品，已停止商品查询。",
                    "required_action": "请用户在前端引用商品中手动勾选要分析的商品，或上传/注入明确商品资料后重试。",
                    "answerPolicy": {
                        "source": "explicit_agent_product_context_required",
                        "no_default_product": true,
                        "live_platform_lookup": false
                    }
                }),
                "未发现本轮显式选择的商品上下文",
            );
        }

        let match_result = product_context.match_query(
            query_product_ref.as_deref(),
            query_product_id.as_deref(),
            query_product_name.as_deref(),
        );
        if !match_result.matched {
            return outcome(
                json!({
                    "status": "deferred",
                    "has_data": false,
                    "adapter_status": "context_filter_not_matched",
                    "data_readiness": "explicit_product_context_not_matched",
                    "summary": "已读取商品上下文，但没有匹配请求的商品条件",
                    "skill": self.spec.id,
                    "query": {
                        "product_ref": query_product_ref,
                        "product_id": query_product_id,
                        "product_name": query_product_name,
                    },
                    "availableProductIds": product_context.product_ids,
                    "refs": product_context.refs,
                    "sources": product_context.sources,
                    "productContextSummary": product_context.summary,
                    "deferred_reason": "本轮确实有显式商品上下文，但输入的商品ID或商品名没有在该上下文中命中；不会改用默认商品或隐式搜索其它商品。",
                    "answerPolicy": {
                        "source": "explicit_agent_product_context",
                        "no_default_product": true,
                        "live_platform_lookup": false
                    }
                }),
                "已读取商品上下文，但没有匹配请求的商品条件",
            );
        }

        let summary = product_context.summary.clone();
        let product_ids = if match_result.product_ids.is_empty() {
            product_context.product_ids.clone()
        } else {
            match_result.product_ids
        };
        let refs = if match_result.refs.is_empty() {
            product_context.refs.clone()
        } else {
            match_result.refs
        };
        let sources = if match_result.sources.is_empty() {
            product_context.sources.clone()
        } else {
            match_result.sources
        };

        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "context_metadata",
                "data_readiness": "explicit_product_context",
                "summary": "商品上下文查询已完成",
                "skill": self.spec.id,
                "query": {
                    "product_ref": query_product_ref,
                    "product_id": query_product_id,
                    "product_name": query_product_name,
                },
                "matchedProductIds": product_ids,
                "refs": refs,
                "sources": sources,
                "productContextSummary": summary,
                "answerPolicy": {
                    "source": "当前 Agent 运行时显式注入的商品上下文",
                    "no_default_product": true,
                    "live_platform_lookup": false,
                    "usage": "客服、运营或内容岗位可基于这些商品资料回答，但不能声称已查询实时平台数据库。"
                }
            }),
            "商品上下文查询已完成",
        )
    }
}

#[derive(Clone, Debug, Default)]
struct ExplicitProductContext {
    product_ids: Vec<String>,
    refs: Vec<String>,
    sources: Vec<Value>,
    source_titles: Vec<String>,
    summary: String,
    explicit_signal: bool,
}

#[derive(Clone, Debug, Default)]
struct ProductContextMatch {
    matched: bool,
    product_ids: Vec<String>,
    refs: Vec<String>,
    sources: Vec<Value>,
}

impl ExplicitProductContext {
    fn has_explicit_context(&self) -> bool {
        self.explicit_signal
    }

    fn match_query(
        &self,
        product_ref: Option<&str>,
        product_id: Option<&str>,
        product_name: Option<&str>,
    ) -> ProductContextMatch {
        if let Some(product_ref) = product_ref.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_ref(product_ref) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: matching_product_ids_by_ref(&self.product_ids, product_ref),
                refs: matching_refs(&self.refs, product_ref),
                sources: matching_sources_by_product_ref(&self.sources, product_ref),
            };
        }

        if let Some(product_id) = product_id.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_id(product_id) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: matching_product_ids(&self.product_ids, product_id),
                refs: matching_refs(&self.refs, product_id),
                sources: matching_sources_by_product_id(&self.sources, product_id),
            };
        }

        if let Some(product_name) = product_name.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_name(product_name) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: self.product_ids.clone(),
                refs: self.refs.clone(),
                sources: matching_sources_by_text(&self.sources, product_name),
            };
        }

        ProductContextMatch {
            matched: true,
            product_ids: self.product_ids.clone(),
            refs: self.refs.clone(),
            sources: self.sources.clone(),
        }
    }

    fn matches_product_ref(&self, product_ref: &str) -> bool {
        let product_ref = product_ref.trim();
        let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
        self.product_ids
            .iter()
            .any(|item| eq_ignore_ascii_case_trimmed(item, product_id))
            || self
                .refs
                .iter()
                .any(|item| eq_ignore_ascii_case_trimmed(item, product_ref))
            || self.sources.iter().any(|source| {
                source_product_id(source)
                    .as_deref()
                    .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                    || text_contains_case_insensitive(&source.to_string(), product_ref)
                    || text_contains_case_insensitive(&source.to_string(), product_id)
            })
            || text_contains_case_insensitive(&self.summary, product_ref)
            || text_contains_case_insensitive(&self.summary, product_id)
            || self.source_titles.iter().any(|item| {
                text_contains_case_insensitive(item, product_ref)
                    || text_contains_case_insensitive(item, product_id)
            })
    }

    fn matches_product_id(&self, product_id: &str) -> bool {
        let product_id = product_id.trim();
        self.product_ids
            .iter()
            .any(|item| eq_ignore_ascii_case_trimmed(item, product_id))
            || self
                .refs
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_id))
            || self.sources.iter().any(|source| {
                source_product_id(source)
                    .as_deref()
                    .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                    || text_contains_case_insensitive(&source.to_string(), product_id)
            })
            || text_contains_case_insensitive(&self.summary, product_id)
            || self
                .source_titles
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_id))
    }

    fn matches_product_name(&self, product_name: &str) -> bool {
        let product_name = product_name.trim();
        text_contains_case_insensitive(&self.summary, product_name)
            || self
                .source_titles
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_name))
            || self
                .sources
                .iter()
                .any(|source| text_contains_case_insensitive(&source.to_string(), product_name))
    }
}

fn explicit_product_context(metadata: &Map<String, Value>) -> ExplicitProductContext {
    let product_ids = collect_metadata_list(
        metadata,
        &["request.context.product_ids", "context.product_ids"],
    );
    let refs = collect_metadata_list(
        metadata,
        &[
            "request.context.refs",
            "context.refs",
            "request.context.combined_refs",
            "context.combined_refs",
        ],
    );
    let sources = collect_metadata_sources(
        metadata,
        &["request.context.sources_json", "context.sources_json"],
    );
    let source_titles = collect_metadata_text_parts(
        metadata,
        &["request.context.source_titles", "context.source_titles"],
    );
    let selected_summaries = collect_metadata_texts(
        metadata,
        &[
            "request.context.selected_summary",
            "context.selected_summary",
            "request.context.summary",
            "context.summary",
        ],
    );
    let combined_summaries = collect_metadata_texts(
        metadata,
        &[
            "request.context.combined_summary",
            "context.combined_summary",
        ],
    );
    let mut summary_parts = selected_summaries.clone();
    summary_parts.extend(combined_summaries.clone());
    let summary = truncate_chars(&summary_parts.join("\n\n"), 4000);

    let explicit_signal = !product_ids.is_empty()
        || refs.iter().any(|item| item.starts_with("product:"))
        || !sources.is_empty()
        || !source_titles.is_empty()
        || !selected_summaries.is_empty();

    ExplicitProductContext {
        product_ids,
        refs,
        sources,
        source_titles,
        summary,
        explicit_signal,
    }
}

fn optional_product_id_param(params: &Value, name: &str) -> Result<Option<String>, SkillError> {
    Ok(match object_params(params)?.get(name) {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) => {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            }
        }
        Some(Value::Number(value)) => Some(value.to_string()),
        Some(_) => {
            return Err(SkillError::invalid_parameter(
                name,
                "string or integer product id",
            ));
        }
    })
}

fn collect_metadata_texts(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for key in keys {
        if let Some(value) = metadata_string_value(metadata, key) {
            push_unique_non_empty(&mut out, value);
        }
    }
    out
}

fn collect_metadata_text_parts(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for value in collect_metadata_texts(metadata, keys) {
        for part in value.split('|') {
            push_unique_non_empty(&mut out, part.trim().to_owned());
        }
    }
    out
}

fn collect_metadata_list(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for key in keys {
        let Some(value) = metadata.get(*key) else {
            continue;
        };
        match value {
            Value::String(text) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    collect_list_value(&mut out, &parsed);
                } else {
                    for item in text.split(',') {
                        push_unique_non_empty(&mut out, item.trim().to_owned());
                    }
                }
            }
            other => collect_list_value(&mut out, other),
        }
    }
    out
}

fn collect_list_value(out: &mut Vec<String>, value: &Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_list_value(out, item);
            }
        }
        Value::String(text) => {
            for item in text.split(',') {
                push_unique_non_empty(out, item.trim().to_owned());
            }
        }
        Value::Number(number) => push_unique_non_empty(out, number.to_string()),
        Value::Bool(value) => push_unique_non_empty(out, value.to_string()),
        Value::Object(_) | Value::Null => {}
    }
}

fn collect_metadata_sources(metadata: &Map<String, Value>, keys: &[&str]) -> Vec<Value> {
    let mut out = Vec::new();
    for key in keys {
        let Some(value) = metadata.get(*key) else {
            continue;
        };
        match value {
            Value::String(text) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    collect_source_value(&mut out, parsed);
                }
            }
            other => collect_source_value(&mut out, other.clone()),
        }
    }
    out
}

fn collect_source_value(out: &mut Vec<Value>, value: Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_source_value(out, item);
            }
        }
        Value::Object(_) | Value::String(_) => {
            if !out.iter().any(|item| item == &value) {
                out.push(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn matching_product_ids(product_ids: &[String], product_id: &str) -> Vec<String> {
    product_ids
        .iter()
        .filter(|item| eq_ignore_ascii_case_trimmed(item, product_id))
        .cloned()
        .collect()
}

fn matching_product_ids_by_ref(product_ids: &[String], product_ref: &str) -> Vec<String> {
    let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
    matching_product_ids(product_ids, product_id)
}

fn matching_refs(refs: &[String], product_id: &str) -> Vec<String> {
    refs.iter()
        .filter(|item| text_contains_case_insensitive(item, product_id))
        .cloned()
        .collect()
}

fn matching_sources_by_product_ref(sources: &[Value], product_ref: &str) -> Vec<Value> {
    let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
    sources
        .iter()
        .filter(|source| {
            source_product_id(source)
                .as_deref()
                .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                || text_contains_case_insensitive(&source.to_string(), product_ref)
                || text_contains_case_insensitive(&source.to_string(), product_id)
        })
        .cloned()
        .collect()
}

fn matching_sources_by_product_id(sources: &[Value], product_id: &str) -> Vec<Value> {
    sources
        .iter()
        .filter(|source| {
            source_product_id(source)
                .as_deref()
                .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                || text_contains_case_insensitive(&source.to_string(), product_id)
        })
        .cloned()
        .collect()
}

fn matching_sources_by_text(sources: &[Value], text: &str) -> Vec<Value> {
    sources
        .iter()
        .filter(|source| text_contains_case_insensitive(&source.to_string(), text))
        .cloned()
        .collect()
}

fn source_product_id(source: &Value) -> Option<String> {
    source
        .get("product_id")
        .or_else(|| source.get("productId"))
        .and_then(|value| match value {
            Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_owned()),
            Value::Number(number) => Some(number.to_string()),
            _ => None,
        })
}

pub(super) fn option_string_json(value: Option<String>) -> Value {
    value
        .filter(|value| !value.trim().is_empty())
        .map(Value::String)
        .unwrap_or(Value::Null)
}

pub(super) fn option_i64_json(value: Option<i64>) -> Value {
    value.map(|value| json!(value)).unwrap_or(Value::Null)
}

pub(super) fn strip_product_ref_prefix(value: &str) -> Option<&str> {
    value
        .strip_prefix("product:")
        .or_else(|| value.strip_prefix("product/"))
        .filter(|value| !value.trim().is_empty())
}

pub(super) fn metadata_string_value(metadata: &Map<String, Value>, key: &str) -> Option<String> {
    match metadata.get(key)? {
        Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_owned()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}
