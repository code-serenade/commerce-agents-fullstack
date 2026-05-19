use serde_json::{Map, Value, json};

use super::{context::ExplicitProductContext, matching::ProductContextMatch};

pub(super) fn missing_context_output(
    skill_id: &str,
    query_product_ref: &Option<String>,
    query_product_id: &Option<String>,
    query_product_name: &Option<String>,
) -> Value {
    json!({
        "status": "deferred",
        "has_data": false,
        "adapter_status": "context_required",
        "data_readiness": "explicit_product_context_required",
        "summary": "未发现本轮显式选择的商品上下文",
        "skill": skill_id,
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
    })
}

pub(super) fn unmatched_context_output(
    skill_id: &str,
    query_product_ref: &Option<String>,
    query_product_id: &Option<String>,
    query_product_name: &Option<String>,
    product_context: ExplicitProductContext,
) -> Value {
    json!({
        "status": "deferred",
        "has_data": false,
        "adapter_status": "context_filter_not_matched",
        "data_readiness": "explicit_product_context_not_matched",
        "summary": "已读取商品上下文，但没有匹配请求的商品条件",
        "skill": skill_id,
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
    })
}

pub(super) fn matched_context_output(
    skill_id: &str,
    query_product_ref: &Option<String>,
    query_product_id: &Option<String>,
    query_product_name: &Option<String>,
    product_context: ExplicitProductContext,
    match_result: ProductContextMatch,
) -> Value {
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

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "context_metadata",
        "data_readiness": "explicit_product_context",
        "summary": "商品上下文查询已完成",
        "skill": skill_id,
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
    })
}

pub(in crate::skills::service) fn option_string_json(value: Option<String>) -> Value {
    value
        .filter(|value| !value.trim().is_empty())
        .map(Value::String)
        .unwrap_or(Value::Null)
}

pub(in crate::skills::service) fn option_i64_json(value: Option<i64>) -> Value {
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

pub(super) fn push_unique_non_empty(out: &mut Vec<String>, value: String) {
    let value = value.trim();
    if value.is_empty() || out.iter().any(|item| item == value) {
        return;
    }
    out.push(value.to_owned());
}

pub(super) fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut out = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        out.push('…');
    }
    out
}

pub(super) fn eq_ignore_ascii_case_trimmed(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}
