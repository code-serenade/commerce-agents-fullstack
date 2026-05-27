use serde::Deserialize;
use serde_json::{Value, json};

use crate::skills::SkillCategory;

pub(super) fn category() -> SkillCategory {
    SkillCategory::Custom("search".to_owned())
}

pub(super) const SEARCH_ADAPTER_REQUIRED_CONFIG: &str =
    "search.provider / search endpoint not configured";
pub(super) const SEARCH_DEFERRED_REASON: &str = "外部搜索 adapter 尚未接入，不能伪造实时来源";
pub(super) const SEARCH_DEGRADED_REASON: &str =
    "Search adapter is not configured; planned query is returned for traceability.";
pub(super) const SEARCH_PROVIDER_EMPTY_REASON: &str =
    "External search provider returned no normalized sources; live sources were not accepted.";
pub(super) const SEARCH_PROVIDER_STALE_REASON: &str =
    "External search provider returned only sources outside the requested freshness window.";
pub(super) const SEARCH_PROVIDER_UNRELIABLE_REASON: &str = "External search provider returned sources, but none passed traceability, freshness, or deduplication filters.";
pub(super) const VALID_TOPICS: &[&str] = &["general", "news", "finance"];
pub(super) const MAX_DAYS: i64 = 365;
pub(super) const MAX_RESULTS: i64 = 12;
pub(super) const DEFAULT_MAX_RESULTS: i64 = 6;
const DEFAULT_TIMEOUT_SEC: u64 = 10;
const INTERACTIVE_TIMEOUT_CAP_SEC: u64 = 25;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SearchAdapterConfig {
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

impl Default for SearchAdapterConfig {
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
            default_max_results: Some(DEFAULT_MAX_RESULTS as u64),
        }
    }
}

impl SearchAdapterConfig {
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

    pub(super) fn effective_timeout_sec(&self) -> u64 {
        self.timeout_sec
            .unwrap_or(DEFAULT_TIMEOUT_SEC)
            .clamp(1, INTERACTIVE_TIMEOUT_CAP_SEC)
    }

    pub(super) fn readiness(&self) -> Value {
        let mut missing_config = Vec::new();
        if !self.enabled {
            missing_config.push("external_search.enabled");
        }
        if self
            .base_url
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            missing_config.push("external_search.base_url");
        }
        if self
            .search_endpoint
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            missing_config.push("external_search.search_endpoint");
        }

        let configured = missing_config.is_empty();
        let status = if configured {
            "configured"
        } else if self.enabled {
            "missing_config"
        } else {
            "disabled"
        };
        let message = match status {
            "configured" => {
                "外部搜索 adapter 已配置，后端会尝试请求真实 provider；provider \
                 仍可能因无可信来源而降级。"
            }
            "missing_config" => "外部搜索 adapter 已启用但配置不完整，当前不会请求 provider。",
            _ => "外部搜索 adapter 未启用，当前不会查询外部实时来源。",
        };

        json!({
            "status": status,
            "enabled": self.enabled,
            "configured": configured,
            "provider": self.provider_name(),
            "missing_config": missing_config,
            "has_api_key": self
                .api_key
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            "timeout_sec": self.effective_timeout_sec(),
            "configured_timeout_sec": self.timeout_sec.unwrap_or(DEFAULT_TIMEOUT_SEC).max(1),
            "default_topic": self
                .default_topic
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("general"),
            "default_days": self.default_days.unwrap_or(30).clamp(1, MAX_DAYS as u64),
            "default_max_results": self
                .default_max_results
                .unwrap_or(DEFAULT_MAX_RESULTS as u64)
                .clamp(1, MAX_RESULTS as u64),
            "max_results_limit": MAX_RESULTS,
            "message": message
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{INTERACTIVE_TIMEOUT_CAP_SEC, SearchAdapterConfig};

    #[test]
    fn effective_timeout_caps_overlong_interactive_search_waits() {
        let config = SearchAdapterConfig {
            timeout_sec: Some(180),
            ..SearchAdapterConfig::default()
        };

        assert_eq!(config.effective_timeout_sec(), INTERACTIVE_TIMEOUT_CAP_SEC);
    }
}
