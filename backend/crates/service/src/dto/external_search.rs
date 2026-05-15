use serde::{Deserialize, Serialize};

const DEFAULT_TIMEOUT_SEC: u64 = 10;
const DEFAULT_TOPIC: &str = "general";
const DEFAULT_DAYS: u64 = 30;
const DEFAULT_MAX_RESULTS: u64 = 6;
const MAX_DAYS: u64 = 365;
const MAX_RESULTS: u64 = 12;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ExternalSearchConfig {
    pub enabled: bool,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub search_endpoint: Option<String>,
    pub api_key: Option<String>,
    pub timeout_sec: Option<u64>,
    pub default_topic: Option<String>,
    pub default_days: Option<u64>,
    pub default_max_results: Option<u64>,
}

impl Default for ExternalSearchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: None,
            base_url: None,
            search_endpoint: None,
            api_key: None,
            timeout_sec: Some(10),
            default_topic: Some("general".to_string()),
            default_days: Some(30),
            default_max_results: Some(6),
        }
    }
}

impl ExternalSearchConfig {
    pub fn is_configured(&self) -> bool {
        self.enabled
            && self
                .base_url
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                .search_endpoint
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
    }

    pub fn provider_name(&self) -> String {
        self.provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("external_search")
            .to_string()
    }

    pub fn readiness(&self) -> ExternalSearchReadiness {
        let mut missing_config = Vec::new();
        if !self.enabled {
            missing_config.push("external_search.enabled".to_string());
        }
        if self
            .base_url
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            missing_config.push("external_search.base_url".to_string());
        }
        if self
            .search_endpoint
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            missing_config.push("external_search.search_endpoint".to_string());
        }

        let configured = missing_config.is_empty();
        let status = if configured {
            "configured"
        } else if self.enabled {
            "missing_config"
        } else {
            "disabled"
        }
        .to_string();
        let message = match status.as_str() {
            "configured" => "外部搜索 adapter 已配置，后端会尝试请求真实 provider；provider 仍可能因无可信来源而降级。",
            "missing_config" => "外部搜索 adapter 已启用但配置不完整，当前不会请求 provider。",
            _ => "外部搜索 adapter 未启用，当前不会查询外部实时来源。",
        }
        .to_string();

        ExternalSearchReadiness {
            status,
            enabled: self.enabled,
            configured,
            provider: self.provider_name(),
            missing_config,
            has_api_key: self
                .api_key
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            timeout_sec: self.timeout_sec.unwrap_or(DEFAULT_TIMEOUT_SEC).max(1),
            default_topic: self
                .default_topic
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(DEFAULT_TOPIC)
                .to_string(),
            default_days: self.default_days.unwrap_or(DEFAULT_DAYS).clamp(1, MAX_DAYS),
            default_max_results: self
                .default_max_results
                .unwrap_or(DEFAULT_MAX_RESULTS)
                .clamp(1, MAX_RESULTS),
            max_results_limit: MAX_RESULTS,
            message,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalSearchQueryRequest {
    pub user_id: String,
    pub q: String,
    pub topic: Option<String>,
    pub days: Option<u64>,
    pub max_results: Option<u64>,
    pub bypass_cache: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ExternalSearchSpec {
    pub query: String,
    pub topic: String,
    pub days: u64,
    pub max_results: u64,
    pub bypass_cache: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSearchStatusResponse {
    pub adapter: String,
    pub enabled: bool,
    pub configured: bool,
    pub provider: Option<String>,
    pub required_config: Option<String>,
    pub status: String,
    pub readiness: ExternalSearchReadiness,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSearchReadiness {
    pub status: String,
    pub enabled: bool,
    pub configured: bool,
    pub provider: String,
    pub missing_config: Vec<String>,
    pub has_api_key: bool,
    pub timeout_sec: u64,
    pub default_topic: String,
    pub default_days: u64,
    pub default_max_results: u64,
    pub max_results_limit: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSearchResponse {
    pub adapter: String,
    pub key: String,
    pub status: String,
    pub query: String,
    pub engine: Option<String>,
    pub requested_at: String,
    pub topic: String,
    pub freshness_days: u64,
    pub max_results: u64,
    pub bypass_cache: bool,
    pub confidence: f64,
    pub provenance: ExternalSearchProvenance,
    pub data: ExternalSearchData,
    pub evidence: ExternalSearchEvidence,
    pub readiness: ExternalSearchReadiness,
    pub notice: Option<String>,
    pub degraded: bool,
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSearchData {
    pub items: Vec<ExternalSearchSource>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSearchSource {
    pub rank: u64,
    pub title: String,
    pub snippet: String,
    pub url: String,
    pub domain: String,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSearchEvidence {
    pub summary: String,
    pub source_titles: Vec<String>,
    pub source_domains: Vec<String>,
    pub llm_context: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSearchProvenance {
    pub source_count: usize,
    pub domain_count: usize,
    pub domains: Vec<String>,
    pub published_count: usize,
    pub latest_published_at: Option<String>,
    pub stale_filtered_count: usize,
}
