use std::collections::BTreeSet;

use serde_json::{Value, json};
use time::{
    Date, Duration, OffsetDateTime, format_description::well_known::Rfc3339,
    macros::format_description,
};
use toolcraft_request::HeaderMap;

use super::{
    config::{
        DEFAULT_MAX_RESULTS, MAX_DAYS, MAX_RESULTS, SEARCH_ADAPTER_REQUIRED_CONFIG,
        SearchAdapterConfig, VALID_TOPICS,
    },
    output::{SearchPlan, SearchSourceSet},
};
use crate::{
    http::request as http_request,
    skills::{optional_bool_param, optional_i64_param, optional_string_param},
};

pub(super) async fn query_provider(
    adapter: &SearchAdapterConfig,
    plan: &SearchPlan,
) -> Result<Value, String> {
    let base_url = adapter
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| SEARCH_ADAPTER_REQUIRED_CONFIG.to_string())?;
    let endpoint = adapter
        .search_endpoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| SEARCH_ADAPTER_REQUIRED_CONFIG.to_string())?;
    let query = vec![
        ("q".to_string(), plan.planned.clone()),
        ("topic".to_string(), plan.topic.clone()),
        ("days".to_string(), plan.days.to_string()),
        ("max_results".to_string(), plan.max_results.to_string()),
        ("bypass_cache".to_string(), plan.bypass_cache.to_string()),
    ];
    let payload = http_request::get_json(
        base_url,
        endpoint,
        Some(query),
        build_provider_headers(adapter)?,
        adapter.effective_timeout_sec(),
    )
    .await
    .map_err(|err| format!("external search provider request failed: {err}"))?;
    unwrap_provider_payload(payload)
}

fn unwrap_provider_payload(payload: Value) -> Result<Value, String> {
    let Some(code) = payload.get("code") else {
        return Ok(payload);
    };

    let success = match code {
        Value::Number(number) => number.as_i64() == Some(0),
        Value::String(text) => text.trim() == "0" || text.trim().eq_ignore_ascii_case("success"),
        _ => false,
    };
    let message = payload
        .get("message")
        .or_else(|| payload.get("msg"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("external search provider returned an error envelope");

    if !success {
        return Err(format!(
            "external search provider returned code {}: {}",
            code, message
        ));
    }

    payload
        .get("data")
        .cloned()
        .ok_or_else(|| "external search provider success envelope missing data".to_string())
}

pub(super) fn normalize_sources(payload: &Value, max_results: i64, days: i64) -> SearchSourceSet {
    let Some(items) = source_items(payload) else {
        return SearchSourceSet::default();
    };

    let result_limit = max_results.clamp(1, MAX_RESULTS) as usize;
    let requested_days = days.clamp(1, MAX_DAYS);
    let mut seen = BTreeSet::new();
    let mut sources = Vec::new();
    let mut stale_filtered_count = 0;
    for item in items {
        let rank = sources.len() as u64 + 1;
        let Some(source) = normalize_source(rank, item) else {
            continue;
        };
        if is_source_outside_freshness_window(&source, requested_days) {
            stale_filtered_count += 1;
            continue;
        }
        let Some(key) = normalized_source_key(&source) else {
            continue;
        };
        if seen.insert(key) {
            sources.push(source);
        }
        if sources.len() >= result_limit {
            break;
        }
    }
    SearchSourceSet {
        sources,
        stale_filtered_count,
    }
}

fn normalize_source(rank: u64, item: &Value) -> Option<Value> {
    let title = string_field(item, &["title", "name", "headline"])
        .map(|value| compact_text(&value, 180))
        .unwrap_or_else(|| "未命名来源".to_string());
    let url = string_field(item, &["url", "link", "href", "source_url", "sourceUrl"])
        .map(|value| compact_text(&value, 400))
        .unwrap_or_default();
    if url.trim().is_empty() {
        return None;
    }
    let domain = extract_domain(&url);
    if !is_traceable_domain(&domain) {
        return None;
    }
    let snippet = string_field(
        item,
        &[
            "snippet",
            "body",
            "summary",
            "content",
            "description",
            "text",
            "excerpt",
        ],
    )
    .map(|value| compact_text(&value, 320))
    .unwrap_or_default();
    let published_at = string_field(
        item,
        &[
            "published_at",
            "publishedAt",
            "pubDate",
            "published",
            "datePublished",
            "date",
        ],
    )
    .map(|value| compact_text(&value, 64))
    .filter(|value| !value.is_empty());

    Some(json!({
        "rank": rank,
        "title": title,
        "snippet": snippet,
        "url": url,
        "domain": domain,
        "published_at": published_at
    }))
}

fn normalized_source_key(source: &Value) -> Option<String> {
    let url = source
        .get("url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let title = source
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    url.or(title)
        .map(|value| value.trim_end_matches('/').to_ascii_lowercase())
}

pub(super) fn build_provenance(sources: &[Value], stale_filtered_count: usize) -> Value {
    let domains = sources
        .iter()
        .filter_map(|item| item.get("domain").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(12)
        .collect::<Vec<_>>();
    let published = sources
        .iter()
        .filter_map(|item| item.get("published_at").and_then(Value::as_str))
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let latest_published_at = published.iter().max().cloned();

    json!({
        "source_count": sources.len(),
        "domain_count": domains.len(),
        "domains": domains,
        "published_count": published.len(),
        "latest_published_at": latest_published_at,
        "stale_filtered_count": stale_filtered_count
    })
}

pub(super) fn build_search_evidence(sources: &[Value], stale_filtered_count: usize) -> Value {
    if sources.is_empty() {
        let summary = if stale_filtered_count > 0 {
            format!(
                "没有可采信实时来源；已过滤 {stale_filtered_count} \
                 条超出请求时间范围的来源，当前结果只能作为降级状态。"
            )
        } else {
            "没有可采信实时来源；当前结果只能作为降级状态，不可当作联网证据。".to_string()
        };
        return json!({
            "summary": summary,
            "source_titles": [],
            "source_domains": [],
            "llm_context": "（本次没有可采信实时来源。回答时必须说明外部搜索未返回来源，不能引用不存在的联网数据。）"
        });
    }

    let titles = sources
        .iter()
        .filter_map(|item| item.get("title").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 120))
        .take(5)
        .collect::<Vec<_>>();
    let domains = sources
        .iter()
        .filter_map(|item| item.get("domain").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(8)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let stale_note = if stale_filtered_count > 0 {
        format!("已过滤 {stale_filtered_count} 条超出请求时间范围的来源；")
    } else {
        String::new()
    };
    let summary = format!(
        "共 {} 条实时来源，覆盖 {} 个域名；{}回答应优先引用这些来源，并注明时效性和置信度。",
        sources.len(),
        domains.len(),
        stale_note
    );

    json!({
        "summary": summary,
        "source_titles": titles,
        "source_domains": domains,
        "llm_context": format_sources_for_llm(sources, stale_filtered_count)
    })
}

pub(super) fn build_search_evidence_packet(
    skill_id: &str,
    plan: &SearchPlan,
    sources: &[Value],
    stale_filtered_count: usize,
    confidence: f64,
    adapter_status: &str,
    degraded_reason: Option<&str>,
) -> Value {
    let strength = search_strength(plan.max_results);
    let usable_claims = usable_claims_from_sources(sources);
    let gaps = search_gaps(sources, stale_filtered_count, degraded_reason);
    let adapter_status = normalized_adapter_status(adapter_status);
    let source_state = search_source_state(sources, adapter_status, degraded_reason);

    json!({
        "intent": search_intent(skill_id),
        "skill": skill_id,
        "adapter_status": adapter_status,
        "source_state": source_state,
        "query": plan.planned,
        "raw_query": plan.raw,
        "search_strength": strength,
        "requested": {
            "topic": plan.topic,
            "days": plan.days,
            "max_results": plan.max_results,
            "bypass_cache": plan.bypass_cache,
            "product_ref": plan.product_ref,
            "product_name": plan.product_name,
            "product_category": plan.product_category
        },
        "sources": evidence_packet_sources(sources),
        "usable_claims": usable_claims,
        "gaps": gaps,
        "confidence": confidence,
        "usage": "搜索岗只把本字段作为证据包交给下游岗位；业务岗位必须先看 adapter_status/source_state：live/has_reliable_sources 才能当作实时来源引用；degraded、adapter_required 或 no_reliable_sources 只能说明未拿到可采信来源和核验路径，不能编造来源、价格、政策或趋势。"
    })
}

fn normalized_adapter_status(status: &str) -> &'static str {
    match status.trim() {
        "live" => "live",
        "degraded" => "degraded",
        "adapter_required" => "adapter_required",
        _ => "unknown",
    }
}

fn search_source_state(
    sources: &[Value],
    adapter_status: &str,
    degraded_reason: Option<&str>,
) -> &'static str {
    if adapter_status == "live" && !sources.is_empty() {
        "has_reliable_sources"
    } else if adapter_status == "adapter_required" {
        "adapter_required"
    } else if degraded_reason.is_some() || adapter_status == "degraded" {
        "no_reliable_sources"
    } else {
        "unknown"
    }
}

fn search_intent(skill_id: &str) -> &'static str {
    match skill_id {
        "search_competitor" => "competitor_price_or_benchmark",
        "search_platform_policy" => "platform_policy_or_compliance",
        "search_trends" => "market_trend",
        "search_product_reviews" => "product_review_or_reputation",
        "search_industry_benchmarks" => "industry_metric_benchmark",
        "search_market_info" => "market_context",
        _ => "general_search_evidence",
    }
}

fn search_strength(max_results: i64) -> &'static str {
    if max_results >= 8 {
        "deep"
    } else if max_results >= 4 {
        "standard"
    } else {
        "light"
    }
}

fn evidence_packet_sources(sources: &[Value]) -> Vec<Value> {
    sources
        .iter()
        .take(12)
        .map(|source| {
            json!({
                "title": source.get("title").and_then(Value::as_str).unwrap_or("未命名来源"),
                "url": source.get("url").and_then(Value::as_str).unwrap_or(""),
                "published_at": source.get("published_at").cloned().unwrap_or(Value::Null),
                "source_type": classify_source_type(source),
                "claim": source_claim(source),
                "relevance": source_relevance(source)
            })
        })
        .collect()
}

fn usable_claims_from_sources(sources: &[Value]) -> Vec<String> {
    sources
        .iter()
        .filter_map(|source| {
            let title = source
                .get("title")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("未命名来源");
            let snippet = source
                .get("snippet")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let domain = source
                .get("domain")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());
            snippet
                .or(domain)
                .map(|detail| compact_text(&format!("{title}：{detail}"), 220))
        })
        .fold(Vec::<String>::new(), |mut claims, claim| {
            if !claims.iter().any(|item| item == &claim) {
                claims.push(claim);
            }
            claims
        })
}

fn search_gaps(
    sources: &[Value],
    stale_filtered_count: usize,
    degraded_reason: Option<&str>,
) -> Vec<String> {
    let mut gaps = Vec::new();
    if sources.is_empty() {
        gaps.push("本次没有可采信公开来源，不能当作实时搜索结论。".to_string());
    }
    if stale_filtered_count > 0 {
        gaps.push(format!(
            "已过滤 {stale_filtered_count} 条超出请求时间范围的来源，过期来源不能引用。"
        ));
    }
    if let Some(reason) = degraded_reason
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        gaps.push(format!("搜索降级原因：{reason}"));
    }
    if !sources.is_empty() && sources.len() < 3 {
        gaps.push("可采信来源少于 3 条，只能作为低置信参考。".to_string());
    }
    if sources.iter().any(|source| {
        source
            .get("published_at")
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
    }) {
        gaps.push("部分来源没有发布时间，使用时需要说明时效性不确定。".to_string());
    }

    gaps
}

fn classify_source_type(source: &Value) -> &'static str {
    let domain = source
        .get("domain")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if domain.contains("gov.") || domain.ends_with(".gov") {
        "official_government"
    } else if domain.contains("edu.") || domain.ends_with(".edu") {
        "education"
    } else if domain.contains("tmall")
        || domain.contains("taobao")
        || domain.contains("jd.")
        || domain.contains("douyin")
        || domain.contains("pinduoduo")
    {
        "platform_or_marketplace"
    } else {
        "public_web"
    }
}

fn source_claim(source: &Value) -> String {
    let snippet = source
        .get("snippet")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let title = source
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("未命名来源");
    compact_text(snippet.unwrap_or(title), 220)
}

fn source_relevance(source: &Value) -> &'static str {
    if source
        .get("snippet")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        "direct"
    } else {
        "context"
    }
}

fn format_sources_for_llm(sources: &[Value], stale_filtered_count: usize) -> String {
    if sources.is_empty() {
        return "（本次没有可采信实时来源。回答时必须说明外部搜索未返回来源，\
                不能引用不存在的联网数据。）"
            .to_string();
    }

    let mut lines = vec![format!(
        "（共{}条实时来源，优先从中提取具体数字、结论和出处；若来源不足，明确说明数据不足。）",
        sources.len().min(5)
    )];
    if stale_filtered_count > 0 {
        lines.push(format!(
            "已过滤{}条超出请求时间范围的来源；不要引用这些过期来源。",
            stale_filtered_count
        ));
    }
    for (index, source) in sources.iter().take(5).enumerate() {
        let title = source
            .get("title")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("未命名来源");
        let snippet = source
            .get("snippet")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        let url = source
            .get("url")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        let published_at = source
            .get("published_at")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());

        lines.push(format!("\n[{}] {}", index + 1, compact_text(title, 120)));
        if let Some(value) = published_at {
            lines.push(format!("发布时间：{}", compact_text(value, 64)));
        }
        if !snippet.is_empty() {
            lines.push(compact_text(snippet, 320));
        }
        if !url.is_empty() {
            lines.push(format!("来源：{}", compact_text(url, 300)));
        }
    }
    lines.push(
        "\n注：遇到实时来源与通用经验不一致时，以实时来源为准，并说明来源数量、时效性和不确定性。"
            .to_string(),
    );
    lines.join("\n")
}

pub(super) fn estimate_confidence(sources: &[Value]) -> f64 {
    if sources.is_empty() {
        return 0.0;
    }
    let source_score = (sources.len() as f64 / DEFAULT_MAX_RESULTS as f64).min(1.0) * 0.55;
    let domains = sources
        .iter()
        .filter_map(|item| item.get("domain").and_then(Value::as_str))
        .filter(|value| !value.trim().is_empty())
        .collect::<BTreeSet<_>>();
    let diversity_score = (domains.len() as f64 / sources.len() as f64).min(1.0) * 0.25;
    let freshness_score = if sources
        .iter()
        .any(|item| item.get("published_at").and_then(Value::as_str).is_some())
    {
        0.2
    } else {
        0.08
    };
    ((source_score + diversity_score + freshness_score).min(1.0) * 1000.0).round() / 1000.0
}

fn is_source_outside_freshness_window(source: &Value, days: i64) -> bool {
    let Some(published_at) = source
        .get("published_at")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    let Some(published_at) = parse_published_at(published_at) else {
        return false;
    };
    let cutoff = OffsetDateTime::now_utc() - Duration::days(days.clamp(1, MAX_DAYS));
    published_at < cutoff
}

fn parse_published_at(value: &str) -> Option<OffsetDateTime> {
    let text = value.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(datetime) = OffsetDateTime::parse(text, &Rfc3339) {
        return Some(datetime);
    }
    Date::parse(text, format_description!("[year]-[month]-[day]"))
        .ok()
        .map(|date| date.midnight().assume_utc())
}

fn build_provider_headers(adapter: &SearchAdapterConfig) -> Result<Option<HeaderMap>, String> {
    let Some(api_key) = adapter
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let mut headers = HeaderMap::new();
    headers
        .insert("Authorization", format!("Bearer {api_key}"))
        .map_err(|err| format!("invalid search auth header: {err}"))?;
    Ok(Some(headers))
}

pub(super) fn normalize_topic(
    topic: &str,
    fallback: Option<&str>,
) -> Result<String, crate::skills::SkillError> {
    let normalized = topic.trim().to_ascii_lowercase();
    let normalized = if normalized.is_empty() {
        fallback
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("general")
            .to_ascii_lowercase()
    } else {
        normalized
    };
    if !VALID_TOPICS.contains(&normalized.as_str()) {
        return Err(crate::skills::SkillError::invalid_parameter(
            "topic",
            "general/news/finance",
        ));
    }
    Ok(normalized)
}

pub(super) fn search_bypass_cache(params: &Value) -> Result<bool, crate::skills::SkillError> {
    Ok(optional_bool_param(params, "bypass_cache")?
        .or(optional_bool_param(params, "bypassCache")?)
        .or(optional_bool_param(params, "_needs_fresh_data")?)
        .unwrap_or(false))
}

pub(super) fn search_max_results(params: &Value) -> Result<Option<i64>, crate::skills::SkillError> {
    Ok(optional_i64_param(params, "max_results")?.or(optional_i64_param(params, "maxResults")?))
}

pub(super) fn optional_search_string_param(
    params: &Value,
    field: &str,
) -> Result<Option<String>, crate::skills::SkillError> {
    optional_string_param(params, field)
}

fn string_field(item: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| item.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn source_items(payload: &Value) -> Option<&[Value]> {
    [
        "/data/items",
        "/data/results",
        "/data/sources",
        "/data/organic",
        "/data/organic_results",
        "/data/webPages/value",
        "/items",
        "/sources",
        "/results",
        "/organic",
        "/organic_results",
        "/webPages/value",
        "/value",
    ]
    .iter()
    .find_map(|pointer| {
        payload
            .pointer(pointer)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
    })
}

fn extract_domain(url: &str) -> String {
    let text = url.trim();
    if text.is_empty() {
        return String::new();
    }
    let without_scheme = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
        .unwrap_or(text);
    without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim()
        .trim_end_matches('.')
        .trim_start_matches("www.")
        .to_ascii_lowercase()
}

fn is_traceable_domain(value: &str) -> bool {
    let text = value.trim();
    !text.is_empty() && text.contains('.') && !text.chars().any(char::is_whitespace)
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    trimmed.chars().take(max_chars).collect()
}

pub(super) fn utc_now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{normalize_sources, unwrap_provider_payload};

    #[test]
    fn unwrap_provider_payload_accepts_common_envelope() {
        let payload = unwrap_provider_payload(json!({
            "code": 0,
            "message": "Success",
            "data": {
                "key": "search",
                "engine": "codex-web-search",
                "data": {
                    "items": [{
                        "title": "Rust release",
                        "url": "https://blog.rust-lang.org/release",
                        "snippet": "Release notes"
                    }]
                }
            }
        }))
        .expect("success envelope should unwrap");

        let sources = normalize_sources(&payload, 6, 30);
        assert_eq!(sources.sources.len(), 1);
        assert_eq!(sources.sources[0]["domain"], "blog.rust-lang.org");
    }

    #[test]
    fn unwrap_provider_payload_rejects_error_envelope() {
        let err = unwrap_provider_payload(json!({
            "code": 500,
            "message": "provider unavailable"
        }))
        .expect_err("error envelope should fail");

        assert!(err.contains("provider unavailable"));
    }
}
