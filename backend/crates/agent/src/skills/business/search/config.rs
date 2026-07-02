use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
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
pub(super) const SEARCH_PROVIDER_UNRELIABLE_REASON: &str = "External search provider returned sources, but none passed traceability, freshness, or \
     deduplication filters.";
pub(super) const VALID_TOPICS: &[&str] = &["general", "news", "finance"];
pub(super) const MAX_DAYS: i64 = 365;
pub(super) const MAX_RESULTS: i64 = 12;
pub(super) const DEFAULT_MAX_RESULTS: i64 = 6;
const DEFAULT_TIMEOUT_SEC: u64 = 10;
const INTERACTIVE_TIMEOUT_CAP_SEC: u64 = 12;
const PROVIDER_FAILURE_COOLDOWN_SEC: u64 = 120;
const PROVIDER_FAILURE_THRESHOLD: u64 = 1;
const MAX_FAILURE_REASON_CHARS: usize = 220;

static SEARCH_PROVIDER_HEALTH: OnceLock<Mutex<BTreeMap<String, SearchProviderHealth>>> =
    OnceLock::new();

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchAdapterHealthStatus {
    pub configured: bool,
    pub usable: bool,
    pub status: String,
    pub provider: String,
    pub recent_failure_count: u64,
    pub last_failure_reason: Option<String>,
    pub cooldown_until_ms: Option<u128>,
    pub cooldown_remaining_sec: Option<u64>,
}

impl SearchAdapterHealthStatus {
    pub fn as_json(&self) -> Value {
        json!({
            "configured": self.configured,
            "usable": self.usable,
            "status": self.status,
            "provider": self.provider,
            "recent_failure_count": self.recent_failure_count,
            "last_failure_reason": self.last_failure_reason,
            "cooldown_until_ms": self.cooldown_until_ms,
            "cooldown_remaining_sec": self.cooldown_remaining_sec
        })
    }
}

#[derive(Debug, Clone, Default)]
struct SearchProviderHealth {
    consecutive_failure_count: u64,
    last_failure_reason: Option<String>,
    last_failure_at_ms: Option<u128>,
    last_success_at_ms: Option<u128>,
    cooldown_until_ms: Option<u128>,
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

    pub fn health_status(&self) -> SearchAdapterHealthStatus {
        let configured = self.is_configured();
        let provider = self.provider_name();
        if !configured {
            let status = if self.enabled {
                "missing_config"
            } else {
                "disabled"
            };
            return SearchAdapterHealthStatus {
                configured: false,
                usable: false,
                status: status.to_string(),
                provider,
                recent_failure_count: 0,
                last_failure_reason: None,
                cooldown_until_ms: None,
                cooldown_remaining_sec: None,
            };
        }

        let now_ms = current_unix_millis();
        let key = self.health_key();
        with_provider_health(|states| {
            let state = states.entry(key).or_default();
            let cooldown_until_ms = state
                .cooldown_until_ms
                .filter(|cooldown_until_ms| *cooldown_until_ms > now_ms);
            if state.cooldown_until_ms != cooldown_until_ms {
                state.cooldown_until_ms = cooldown_until_ms;
            }
            let cooldown_remaining_sec = cooldown_until_ms.map(|value| {
                let remaining_ms = value.saturating_sub(now_ms);
                remaining_ms.div_ceil(1000) as u64
            });
            SearchAdapterHealthStatus {
                configured: true,
                usable: cooldown_until_ms.is_none(),
                status: if cooldown_until_ms.is_some() {
                    "cooldown".to_string()
                } else {
                    "configured".to_string()
                },
                provider,
                recent_failure_count: state.consecutive_failure_count,
                last_failure_reason: state.last_failure_reason.clone(),
                cooldown_until_ms,
                cooldown_remaining_sec,
            }
        })
    }

    pub(super) fn provider_cooldown_reason(&self) -> Option<String> {
        let status = self.health_status();
        if status.usable || status.status != "cooldown" {
            return None;
        }

        Some(format!(
            "external search provider is temporarily unavailable after recent failures; retry in \
             {} seconds",
            status
                .cooldown_remaining_sec
                .unwrap_or(PROVIDER_FAILURE_COOLDOWN_SEC)
        ))
    }

    pub(super) fn record_provider_success(&self) {
        if !self.is_configured() {
            return;
        }
        let key = self.health_key();
        let now_ms = current_unix_millis();
        with_provider_health(|states| {
            let state = states.entry(key).or_default();
            state.consecutive_failure_count = 0;
            state.last_failure_reason = None;
            state.cooldown_until_ms = None;
            state.last_success_at_ms = Some(now_ms);
        });
    }

    pub(super) fn record_provider_failure(&self, reason: &str) {
        if !self.is_configured() {
            return;
        }
        let key = self.health_key();
        let now_ms = current_unix_millis();
        let reason = compact_text(reason, MAX_FAILURE_REASON_CHARS);
        with_provider_health(|states| {
            let state = states.entry(key).or_default();
            state.consecutive_failure_count = state.consecutive_failure_count.saturating_add(1);
            state.last_failure_reason = Some(reason);
            state.last_failure_at_ms = Some(now_ms);
            if state.consecutive_failure_count >= PROVIDER_FAILURE_THRESHOLD {
                state.cooldown_until_ms =
                    Some(now_ms + u128::from(PROVIDER_FAILURE_COOLDOWN_SEC) * 1000);
            }
        });
    }

    fn health_key(&self) -> String {
        format!(
            "{}|{}|{}",
            self.provider_name(),
            self.base_url.as_deref().map(str::trim).unwrap_or_default(),
            self.search_endpoint
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
        )
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

        let health = self.health_status();
        json!({
            "status": status,
            "enabled": self.enabled,
            "configured": configured,
            "usable": health.usable,
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
            "health": health.as_json(),
            "message": message
        })
    }
}

fn provider_health() -> &'static Mutex<BTreeMap<String, SearchProviderHealth>> {
    SEARCH_PROVIDER_HEALTH.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn with_provider_health<T>(f: impl FnOnce(&mut BTreeMap<String, SearchProviderHealth>) -> T) -> T {
    let mut guard = provider_health()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

fn current_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
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

    #[test]
    fn configured_adapter_health_is_usable_before_failures() {
        let config = SearchAdapterConfig {
            enabled: true,
            provider: Some("health-test".to_string()),
            base_url: Some("http://search.health.test/usable".to_string()),
            search_endpoint: Some("/api/external/search".to_string()),
            ..SearchAdapterConfig::default()
        };

        let status = config.health_status();

        assert!(status.configured);
        assert!(status.usable);
        assert_eq!(status.status, "configured");
    }

    #[test]
    fn provider_failure_trips_cooldown_and_success_restores_usable_status() {
        let config = SearchAdapterConfig {
            enabled: true,
            provider: Some("health-test".to_string()),
            base_url: Some("http://search.health.test/cooldown".to_string()),
            search_endpoint: Some("/api/external/search".to_string()),
            ..SearchAdapterConfig::default()
        };

        config.record_provider_failure("provider timeout while searching");
        let degraded = config.health_status();

        assert!(degraded.configured);
        assert!(!degraded.usable);
        assert_eq!(degraded.status, "cooldown");
        assert_eq!(degraded.recent_failure_count, 1);
        assert!(
            degraded
                .last_failure_reason
                .as_deref()
                .unwrap_or_default()
                .contains("provider timeout")
        );
        assert!(config.provider_cooldown_reason().is_some());

        config.record_provider_success();
        let restored = config.health_status();

        assert!(restored.usable);
        assert_eq!(restored.status, "configured");
        assert_eq!(restored.recent_failure_count, 0);
        assert!(restored.last_failure_reason.is_none());
    }
}
