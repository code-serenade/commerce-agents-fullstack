use serde::{Deserialize, Serialize};

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
