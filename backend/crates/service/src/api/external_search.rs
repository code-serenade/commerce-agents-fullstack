use std::collections::BTreeSet;

use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::app_users::{AppUserService, AppUserStatus};
use serde_json::Value;
use time::{
    Date, Duration, OffsetDateTime, format_description::well_known::Rfc3339,
    macros::format_description,
};
use toolcraft_request::{HeaderMap, Request};
use uuid::Uuid;

use crate::dto::external_search::{
    ExternalSearchConfig, ExternalSearchData, ExternalSearchEvidence, ExternalSearchProvenance,
    ExternalSearchQueryRequest, ExternalSearchResponse, ExternalSearchSource, ExternalSearchSpec,
    ExternalSearchStatusResponse,
};

const ADAPTER_NAME: &str = "external_search";
const REQUIRED_CONFIG: &str =
    "external_search.enabled=true, base_url and search_endpoint are required";
const SEARCH_NOT_CONFIGURED_REASON: &str =
    "External search adapter is not configured; no live sources were queried.";
const SEARCH_PROVIDER_STALE_REASON: &str =
    "External search provider returned only sources outside the requested freshness window.";
const VALID_TOPICS: &[&str] = &["general", "news", "finance"];
const DEFAULT_TOPIC: &str = "general";
const DEFAULT_DAYS: u64 = 30;
const DEFAULT_MAX_RESULTS: u64 = 6;
const MAX_DAYS: u64 = 365;
const MAX_RESULTS: u64 = 12;

pub struct ExternalSearchApi {
    app_user_svc: AppUserService,
    config: ExternalSearchConfig,
}

#[derive(Debug, Clone, Default)]
struct ExternalSearchSourceSet {
    sources: Vec<ExternalSearchSource>,
    stale_filtered_count: usize,
}

impl ExternalSearchApi {
    pub fn new(db: DbContext, config: ExternalSearchConfig) -> Self {
        Self {
            app_user_svc: AppUserService::new(db),
            config,
        }
    }

    pub async fn status(&self, user_id: String) -> BizResult<ExternalSearchStatusResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        Ok(status_from_config(&self.config))
    }

    pub async fn query_external_search(
        &self,
        req: ExternalSearchQueryRequest,
    ) -> BizResult<ExternalSearchResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let spec = normalize_query(req, &self.config)?;

        if !self.config.is_configured() {
            return Ok(degraded_response(
                &self.config,
                &spec,
                SEARCH_NOT_CONFIGURED_REASON,
            ));
        }

        match self.query_provider(&spec).await {
            Ok(response) => Ok(response),
            Err(reason) => Ok(degraded_response(&self.config, &spec, &reason)),
        }
    }

    async fn query_provider(
        &self,
        spec: &ExternalSearchSpec,
    ) -> Result<ExternalSearchResponse, String> {
        let base_url = self
            .config
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| REQUIRED_CONFIG.to_string())?;
        let endpoint = self
            .config
            .search_endpoint
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| REQUIRED_CONFIG.to_string())?;

        let mut client = Request::with_timeout(self.config.timeout_sec.unwrap_or(10).max(1))
            .map_err(|err| format!("init search request client failed: {err}"))?;
        client
            .set_base_url(base_url)
            .map_err(|err| format!("invalid search base_url: {err}"))?;

        let headers = build_provider_headers(&self.config).map_err(|err| err.to_string())?;
        let query = vec![
            ("q".to_string(), spec.query.clone()),
            ("topic".to_string(), spec.topic.clone()),
            ("days".to_string(), spec.days.to_string()),
            ("max_results".to_string(), spec.max_results.to_string()),
            ("bypass_cache".to_string(), spec.bypass_cache.to_string()),
        ];
        let response = client
            .get(endpoint, Some(query), headers)
            .await
            .map_err(|err| format!("external search provider request failed: {err}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable body>".to_string());
            return Err(format!(
                "external search provider returned status {}: {}",
                status.as_u16(),
                compact_text(&body, 240)
            ));
        }
        let payload = response
            .json::<Value>()
            .await
            .map_err(|err| format!("decode external search provider response failed: {err}"))?;
        Ok(normalize_provider_response(&self.config, spec, &payload))
    }

    async fn ensure_app_user_enabled(&self, user_id: Uuid) -> BizResult<()> {
        let user = self
            .app_user_svc
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::APP_USER_NOT_FOUND,
                    "app user not found".to_string(),
                )
            })?;
        if user.status != AppUserStatus::Enabled {
            return Err(BizError::new(
                app_error::APP_USER_DISABLED,
                "app user is disabled".to_string(),
            ));
        }
        Ok(())
    }
}

fn status_from_config(config: &ExternalSearchConfig) -> ExternalSearchStatusResponse {
    let configured = config.is_configured();
    let readiness = config.readiness();
    ExternalSearchStatusResponse {
        adapter: ADAPTER_NAME.to_string(),
        enabled: config.enabled,
        configured,
        provider: configured.then(|| config.provider_name()),
        required_config: (!configured).then(|| REQUIRED_CONFIG.to_string()),
        status: if configured {
            "configured".to_string()
        } else {
            "degraded".to_string()
        },
        readiness,
    }
}

fn normalize_query(
    req: ExternalSearchQueryRequest,
    config: &ExternalSearchConfig,
) -> BizResult<ExternalSearchSpec> {
    let query = req.q.trim().to_string();
    if query.is_empty() {
        return Err(BizError::new(
            app_error::EXTERNAL_SEARCH_REQUEST_INVALID,
            "q cannot be empty".to_string(),
        ));
    }
    if query.chars().count() > 500 {
        return Err(BizError::new(
            app_error::EXTERNAL_SEARCH_REQUEST_INVALID,
            "q is too long; max 500 characters".to_string(),
        ));
    }

    let topic = req
        .topic
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or(config.default_topic.as_deref())
        .unwrap_or(DEFAULT_TOPIC)
        .to_ascii_lowercase();
    if !VALID_TOPICS.contains(&topic.as_str()) {
        return Err(BizError::new(
            app_error::EXTERNAL_SEARCH_REQUEST_INVALID,
            format!("invalid topic: {topic}"),
        ));
    }

    let days = req.days.unwrap_or_else(|| {
        config
            .default_days
            .unwrap_or(DEFAULT_DAYS)
            .clamp(1, MAX_DAYS)
    });
    if !(1..=MAX_DAYS).contains(&days) {
        return Err(BizError::new(
            app_error::EXTERNAL_SEARCH_REQUEST_INVALID,
            format!("days must be between 1 and {MAX_DAYS}"),
        ));
    }

    let max_results = req.max_results.unwrap_or_else(|| {
        config
            .default_max_results
            .unwrap_or(DEFAULT_MAX_RESULTS)
            .clamp(1, MAX_RESULTS)
    });
    if !(1..=MAX_RESULTS).contains(&max_results) {
        return Err(BizError::new(
            app_error::EXTERNAL_SEARCH_REQUEST_INVALID,
            format!("maxResults must be between 1 and {MAX_RESULTS}"),
        ));
    }

    Ok(ExternalSearchSpec {
        query,
        topic,
        days,
        max_results,
        bypass_cache: req.bypass_cache.unwrap_or(false),
    })
}

fn degraded_response(
    config: &ExternalSearchConfig,
    spec: &ExternalSearchSpec,
    reason: &str,
) -> ExternalSearchResponse {
    ExternalSearchResponse {
        adapter: ADAPTER_NAME.to_string(),
        key: "search".to_string(),
        status: "degraded".to_string(),
        query: spec.query.clone(),
        engine: config.enabled.then(|| config.provider_name()),
        requested_at: utc_now_rfc3339(),
        topic: spec.topic.clone(),
        freshness_days: spec.days,
        max_results: spec.max_results,
        bypass_cache: spec.bypass_cache,
        confidence: 0.0,
        provenance: build_provenance(&[], 0),
        data: ExternalSearchData { items: Vec::new() },
        evidence: build_search_evidence(&[], 0),
        readiness: config.readiness(),
        notice: Some("未获取到可靠公开来源，当前不能伪造实时搜索结果。".to_string()),
        degraded: true,
        degraded_reason: Some(reason.to_string()),
    }
}

fn normalize_provider_response(
    config: &ExternalSearchConfig,
    spec: &ExternalSearchSpec,
    payload: &Value,
) -> ExternalSearchResponse {
    let source_set = normalize_sources(payload, spec.max_results, spec.days);
    let sources = source_set.sources;
    let provenance = build_provenance(&sources, source_set.stale_filtered_count);
    let evidence = build_search_evidence(&sources, source_set.stale_filtered_count);
    let confidence = payload
        .get("confidence")
        .and_then(Value::as_f64)
        .map(|value| (value * 1000.0).round() / 1000.0)
        .unwrap_or_else(|| estimate_confidence(&sources));
    let engine = payload
        .get("engine")
        .or_else(|| payload.get("_search_engine"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| Some(config.provider_name()));
    let status = if sources.is_empty() {
        "degraded"
    } else {
        payload
            .get("status")
            .and_then(Value::as_str)
            .filter(|value| *value == "live")
            .unwrap_or("live")
    };

    ExternalSearchResponse {
        adapter: ADAPTER_NAME.to_string(),
        key: payload
            .get("key")
            .and_then(Value::as_str)
            .unwrap_or("search")
            .to_string(),
        status: status.to_string(),
        query: spec.query.clone(),
        engine,
        requested_at: payload
            .get("requested_at")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(utc_now_rfc3339),
        topic: spec.topic.clone(),
        freshness_days: spec.days,
        max_results: spec.max_results,
        bypass_cache: spec.bypass_cache,
        confidence,
        provenance,
        data: ExternalSearchData {
            items: sources.clone(),
        },
        evidence,
        readiness: config.readiness(),
        notice: sources.is_empty().then(|| {
            if source_set.stale_filtered_count > 0 {
                "Provider returned only stale sources outside the requested freshness window."
                    .to_string()
            } else {
                "Provider returned no normalized sources.".to_string()
            }
        }),
        degraded: sources.is_empty(),
        degraded_reason: sources.is_empty().then(|| {
            if source_set.stale_filtered_count > 0 {
                SEARCH_PROVIDER_STALE_REASON.to_string()
            } else {
                "External search provider returned no normalized sources.".to_string()
            }
        }),
    }
}

fn normalize_sources(payload: &Value, max_results: u64, days: u64) -> ExternalSearchSourceSet {
    let Some(items) = source_items(payload) else {
        return ExternalSearchSourceSet::default();
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
    ExternalSearchSourceSet {
        sources,
        stale_filtered_count,
    }
}

fn normalize_source(rank: u64, item: &Value) -> Option<ExternalSearchSource> {
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

    Some(ExternalSearchSource {
        rank,
        title,
        snippet,
        url,
        domain,
        published_at,
    })
}

fn normalized_source_key(source: &ExternalSearchSource) -> Option<String> {
    let url = source.url.trim();
    let title = source.title.trim();
    if !url.is_empty() {
        Some(url.trim_end_matches('/').to_ascii_lowercase())
    } else if !title.is_empty() {
        Some(title.to_ascii_lowercase())
    } else {
        None
    }
}

fn build_provenance(
    sources: &[ExternalSearchSource],
    stale_filtered_count: usize,
) -> ExternalSearchProvenance {
    let domains = sources
        .iter()
        .filter_map(|item| {
            let domain = item.domain.trim();
            (!domain.is_empty()).then(|| domain.to_string())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(12)
        .collect::<Vec<_>>();
    let published = sources
        .iter()
        .filter_map(|item| item.published_at.as_ref())
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    let latest_published_at = published.iter().max().cloned();

    ExternalSearchProvenance {
        source_count: sources.len(),
        domain_count: domains.len(),
        domains,
        published_count: published.len(),
        latest_published_at,
        stale_filtered_count,
    }
}

fn build_search_evidence(
    sources: &[ExternalSearchSource],
    stale_filtered_count: usize,
) -> ExternalSearchEvidence {
    if sources.is_empty() {
        let summary = if stale_filtered_count > 0 {
            format!(
                "没有可采信实时来源；已过滤 {stale_filtered_count} 条超出请求时间范围的来源，当前结果只能作为降级状态。"
            )
        } else {
            "没有可采信实时来源；当前结果只能作为降级状态，不可当作联网证据。".to_string()
        };
        return ExternalSearchEvidence {
            summary,
            source_titles: Vec::new(),
            source_domains: Vec::new(),
            llm_context: "（本次没有可采信实时来源。回答时必须说明外部搜索未返回来源，不能引用不存在的联网数据。）".to_string(),
        };
    }

    let source_titles = sources
        .iter()
        .map(|source| compact_text(&source.title, 120))
        .filter(|value| !value.trim().is_empty())
        .take(5)
        .collect::<Vec<_>>();
    let source_domains = sources
        .iter()
        .map(|source| source.domain.trim())
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
        source_domains.len(),
        stale_note
    );

    ExternalSearchEvidence {
        summary,
        source_titles,
        source_domains,
        llm_context: format_sources_for_llm(sources, stale_filtered_count),
    }
}

fn format_sources_for_llm(sources: &[ExternalSearchSource], stale_filtered_count: usize) -> String {
    if sources.is_empty() {
        return "（本次没有可采信实时来源。回答时必须说明外部搜索未返回来源，不能引用不存在的联网数据。）".to_string();
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
        lines.push(format!(
            "\n[{}] {}",
            index + 1,
            compact_text(&source.title, 120)
        ));
        if let Some(published_at) = &source.published_at {
            lines.push(format!("发布时间：{}", compact_text(published_at, 64)));
        }
        if !source.snippet.trim().is_empty() {
            lines.push(compact_text(&source.snippet, 320));
        }
        if !source.url.trim().is_empty() {
            lines.push(format!("来源：{}", compact_text(&source.url, 300)));
        }
    }
    lines.push(
        "\n注：遇到实时来源与通用经验不一致时，以实时来源为准，并说明来源数量、时效性和不确定性。"
            .to_string(),
    );
    lines.join("\n")
}

fn estimate_confidence(sources: &[ExternalSearchSource]) -> f64 {
    if sources.is_empty() {
        return 0.0;
    }
    let source_score = (sources.len() as f64 / DEFAULT_MAX_RESULTS as f64).min(1.0) * 0.55;
    let domains = sources
        .iter()
        .filter(|item| !item.domain.trim().is_empty())
        .map(|item| item.domain.as_str())
        .collect::<BTreeSet<_>>();
    let diversity_score = (domains.len() as f64 / sources.len() as f64).min(1.0) * 0.25;
    let freshness_score = if sources.iter().any(|item| item.published_at.is_some()) {
        0.2
    } else {
        0.08
    };
    ((source_score + diversity_score + freshness_score).min(1.0) * 1000.0).round() / 1000.0
}

fn is_source_outside_freshness_window(source: &ExternalSearchSource, days: u64) -> bool {
    let Some(published_at) = source
        .published_at
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    let Some(published_at) = parse_published_at(published_at) else {
        return false;
    };
    let cutoff = OffsetDateTime::now_utc() - Duration::days(days.clamp(1, MAX_DAYS) as i64);
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

fn build_provider_headers(config: &ExternalSearchConfig) -> Result<Option<HeaderMap>, String> {
    let Some(api_key) = config
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

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    Uuid::parse_str(user_id).map_err(|_| {
        BizError::new(
            app_error::EXTERNAL_SEARCH_REQUEST_INVALID,
            "invalid user_id: must be uuid".to_string(),
        )
    })
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

fn utc_now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn disabled_config_reports_degraded_status() {
        let config = ExternalSearchConfig::default();
        let status = status_from_config(&config);

        assert_eq!(status.adapter, ADAPTER_NAME);
        assert!(!status.enabled);
        assert!(!status.configured);
        assert_eq!(status.status, "degraded");
        assert!(status.required_config.is_some());
        assert_eq!(status.readiness.status, "disabled");
        assert!(!status.readiness.configured);
        assert!(
            status
                .readiness
                .missing_config
                .contains(&"external_search.enabled".to_string())
        );
        assert_eq!(status.readiness.max_results_limit, MAX_RESULTS);
    }

    #[test]
    fn enabled_config_reports_missing_config_until_provider_endpoint_is_complete() {
        let config = ExternalSearchConfig {
            enabled: true,
            provider: Some("custom".to_string()),
            base_url: Some("https://search.example.test".to_string()),
            search_endpoint: None,
            timeout_sec: Some(0),
            default_days: Some(1000),
            default_max_results: Some(100),
            ..ExternalSearchConfig::default()
        };
        let status = status_from_config(&config);

        assert!(status.enabled);
        assert!(!status.configured);
        assert_eq!(status.status, "degraded");
        assert_eq!(status.readiness.status, "missing_config");
        assert_eq!(status.readiness.provider, "custom");
        assert_eq!(status.readiness.timeout_sec, 1);
        assert_eq!(status.readiness.default_days, MAX_DAYS);
        assert_eq!(status.readiness.default_max_results, MAX_RESULTS);
        assert_eq!(
            status.readiness.missing_config,
            vec!["external_search.search_endpoint".to_string()]
        );
    }

    #[test]
    fn normalize_query_uses_safe_defaults() {
        let req = ExternalSearchQueryRequest {
            user_id: Uuid::new_v4().to_string(),
            q: " 咖啡 趋势 ".to_string(),
            topic: None,
            days: None,
            max_results: None,
            bypass_cache: None,
        };
        let spec = normalize_query(req, &ExternalSearchConfig::default()).expect("valid query");

        assert_eq!(spec.query, "咖啡 趋势");
        assert_eq!(spec.topic, DEFAULT_TOPIC);
        assert_eq!(spec.days, DEFAULT_DAYS);
        assert_eq!(spec.max_results, DEFAULT_MAX_RESULTS);
        assert!(!spec.bypass_cache);
    }

    #[test]
    fn degraded_response_preserves_request_controls_and_empty_evidence() {
        let config = ExternalSearchConfig::default();
        let spec = ExternalSearchSpec {
            query: "咖啡 趋势".to_string(),
            topic: "general".to_string(),
            days: 7,
            max_results: 3,
            bypass_cache: true,
        };

        let response = degraded_response(&config, &spec, SEARCH_NOT_CONFIGURED_REASON);

        assert_eq!(response.status, "degraded");
        assert_eq!(response.max_results, 3);
        assert!(response.bypass_cache);
        assert_eq!(response.evidence.source_titles.len(), 0);
        assert!(response.evidence.llm_context.contains("没有可采信实时来源"));
        assert_eq!(response.readiness.status, "disabled");
        assert!(!response.readiness.configured);
    }

    #[test]
    fn provider_payload_is_normalized_into_sources_and_provenance() {
        let config = ExternalSearchConfig {
            enabled: true,
            provider: Some("legacy".to_string()),
            base_url: Some("http://example.test".to_string()),
            search_endpoint: Some("/search".to_string()),
            ..ExternalSearchConfig::default()
        };
        let spec = ExternalSearchSpec {
            query: "咖啡".to_string(),
            topic: "news".to_string(),
            days: 14,
            max_results: 6,
            bypass_cache: false,
        };
        let fresh_date = (OffsetDateTime::now_utc() - Duration::days(1))
            .format(&Rfc3339)
            .expect("fresh date should format");
        let payload = json!({
            "status": "live",
            "engine": "free_web",
            "confidence": 0.72,
            "data": {
                "items": [
                    {
                        "title": "咖啡消费趋势",
                        "snippet": "冷萃咖啡增长",
                        "url": "https://www.example.com/a",
                        "published_at": fresh_date
                    },
                    {
                        "title": "咖啡行业报告",
                        "body": "即饮咖啡继续增长",
                        "url": "https://report.example.cn/b"
                    }
                ]
            }
        });

        let response = normalize_provider_response(&config, &spec, &payload);

        assert_eq!(response.status, "live");
        assert!(!response.degraded);
        assert_eq!(response.max_results, 6);
        assert!(!response.bypass_cache);
        assert_eq!(response.engine.as_deref(), Some("free_web"));
        assert_eq!(response.confidence, 0.72);
        assert_eq!(response.data.items.len(), 2);
        assert_eq!(response.data.items[0].domain, "example.com");
        assert_eq!(response.provenance.source_count, 2);
        assert_eq!(response.provenance.domain_count, 2);
        assert_eq!(response.provenance.published_count, 1);
        assert!(response.evidence.summary.contains("2 条实时来源"));
        assert!(response.evidence.llm_context.contains("咖啡消费趋势"));
        assert_eq!(response.readiness.status, "configured");
        assert!(response.readiness.configured);
        assert_eq!(response.readiness.provider, "legacy");
    }

    #[test]
    fn provider_payload_deduplicates_sources_by_url() {
        let payload = json!({
            "results": [
                {
                    "title": "咖啡消费趋势",
                    "snippet": "冷萃咖啡增长",
                    "url": "https://www.example.com/a"
                },
                {
                    "title": "重复来源",
                    "snippet": "重复 URL",
                    "url": "https://www.example.com/a/"
                },
                {
                    "title": "咖啡行业报告",
                    "body": "即饮咖啡继续增长",
                    "url": "https://report.example.cn/b"
                }
            ]
        });

        let sources = normalize_sources(&payload, MAX_RESULTS, MAX_DAYS).sources;

        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].rank, 1);
        assert_eq!(sources[1].rank, 2);
        assert_eq!(sources[0].domain, "example.com");
        assert_eq!(sources[1].domain, "report.example.cn");
    }

    #[test]
    fn provider_payload_accepts_nested_common_search_shapes() {
        let payload = json!({
            "data": {
                "webPages": {
                    "value": [
                        {
                            "name": "咖啡行业公开报告",
                            "description": "即饮咖啡线上增速继续提升",
                            "url": "https://www.research.example.cn/report",
                            "datePublished": "2026-05-02"
                        }
                    ]
                }
            }
        });

        let sources = normalize_sources(&payload, MAX_RESULTS, MAX_DAYS).sources;

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].rank, 1);
        assert_eq!(sources[0].title, "咖啡行业公开报告");
        assert_eq!(sources[0].snippet, "即饮咖啡线上增速继续提升");
        assert_eq!(sources[0].domain, "research.example.cn");
        assert_eq!(sources[0].published_at.as_deref(), Some("2026-05-02"));
    }

    #[test]
    fn provider_payload_rejects_sources_without_traceable_url() {
        let payload = json!({
            "organic_results": [
                {
                    "title": "只有标题的弱来源",
                    "snippet": "没有 URL，不能作为联网证据",
                    "domain": "example.com"
                },
                {
                    "title": "相对路径弱来源",
                    "snippet": "相对路径不能作为可追溯来源",
                    "href": "/relative/path",
                    "domain": "example.com"
                },
                {
                    "headline": "可追溯来源",
                    "excerpt": "有 href 可核验",
                    "href": "https://verified.example.com/article"
                }
            ]
        });

        let sources = normalize_sources(&payload, MAX_RESULTS, MAX_DAYS).sources;

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].rank, 1);
        assert_eq!(sources[0].title, "可追溯来源");
        assert_eq!(sources[0].domain, "verified.example.com");
    }

    #[test]
    fn provider_payload_filters_sources_outside_requested_freshness() {
        let config = ExternalSearchConfig {
            enabled: true,
            provider: Some("legacy".to_string()),
            base_url: Some("http://example.test".to_string()),
            search_endpoint: Some("/search".to_string()),
            ..ExternalSearchConfig::default()
        };
        let spec = ExternalSearchSpec {
            query: "咖啡 最新趋势".to_string(),
            topic: "news".to_string(),
            days: 7,
            max_results: 6,
            bypass_cache: false,
        };
        let stale_date = (OffsetDateTime::now_utc() - Duration::days(90))
            .format(&Rfc3339)
            .expect("stale date should format");
        let fresh_date = (OffsetDateTime::now_utc() - Duration::days(3))
            .format(&Rfc3339)
            .expect("fresh date should format");
        let payload = json!({
            "results": [
                {
                    "title": "旧咖啡报告",
                    "snippet": "超出最近7天",
                    "url": "https://old.example.com/report",
                    "published_at": stale_date
                },
                {
                    "title": "新咖啡趋势",
                    "snippet": "最近3天内发布",
                    "url": "https://fresh.example.com/trend",
                    "published_at": fresh_date
                }
            ]
        });

        let response = normalize_provider_response(&config, &spec, &payload);

        assert_eq!(response.status, "live");
        assert_eq!(response.data.items.len(), 1);
        assert_eq!(response.data.items[0].title, "新咖啡趋势");
        assert_eq!(response.provenance.stale_filtered_count, 1);
        assert!(response.evidence.summary.contains("已过滤 1 条"));
        assert!(!response.evidence.llm_context.contains("旧咖啡报告"));
    }

    #[test]
    fn provider_payload_respects_requested_max_results() {
        let config = ExternalSearchConfig {
            enabled: true,
            provider: Some("legacy".to_string()),
            base_url: Some("http://example.test".to_string()),
            search_endpoint: Some("/search".to_string()),
            ..ExternalSearchConfig::default()
        };
        let spec = ExternalSearchSpec {
            query: "咖啡".to_string(),
            topic: "general".to_string(),
            days: 30,
            max_results: 2,
            bypass_cache: false,
        };
        let payload = json!({
            "results": [
                {
                    "title": "咖啡消费趋势",
                    "snippet": "冷萃咖啡增长",
                    "url": "https://www.example.com/a"
                },
                {
                    "title": "咖啡行业报告",
                    "snippet": "即饮咖啡继续增长",
                    "url": "https://report.example.cn/b"
                },
                {
                    "title": "第三条应被截断",
                    "snippet": "超过本次请求 maxResults",
                    "url": "https://extra.example.org/c"
                }
            ]
        });

        let response = normalize_provider_response(&config, &spec, &payload);

        assert_eq!(response.max_results, 2);
        assert_eq!(response.data.items.len(), 2);
        assert_eq!(response.provenance.source_count, 2);
        assert_eq!(response.evidence.source_titles.len(), 2);
        assert!(!response.evidence.llm_context.contains("第三条应被截断"));
    }
}
