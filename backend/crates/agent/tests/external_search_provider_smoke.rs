use std::env;

use agent::api::{SearchAdapterConfig, SearchProviderSmokeRequest, run_search_provider_smoke};

const ENABLE_ENV: &str = "CM_AGENT_RUN_EXTERNAL_SEARCH_SMOKE";
const BASE_URL_ENV: &str = "CM_AGENT_SEARCH_BASE_URL";
const ENDPOINT_ENV: &str = "CM_AGENT_SEARCH_ENDPOINT";
const PROVIDER_ENV: &str = "CM_AGENT_SEARCH_PROVIDER";
const API_KEY_ENV: &str = "CM_AGENT_SEARCH_API_KEY";
const QUERY_ENV: &str = "CM_AGENT_SEARCH_QUERY";
const TOPIC_ENV: &str = "CM_AGENT_SEARCH_TOPIC";
const DAYS_ENV: &str = "CM_AGENT_SEARCH_DAYS";
const MAX_RESULTS_ENV: &str = "CM_AGENT_SEARCH_MAX_RESULTS";
const BYPASS_CACHE_ENV: &str = "CM_AGENT_SEARCH_BYPASS_CACHE";
const TIMEOUT_SEC_ENV: &str = "CM_AGENT_SEARCH_TIMEOUT_SEC";

const DEFAULT_ENDPOINT: &str = "/api/external/search";
const DEFAULT_PROVIDER: &str = "codex-web-search";
const DEFAULT_QUERY: &str = "校园冷萃咖啡 价格 竞品 卖点";
const DEFAULT_TOPIC: &str = "general";
const DEFAULT_DAYS: i64 = 30;
const DEFAULT_MAX_RESULTS: i64 = 6;
const DEFAULT_TIMEOUT_SEC: u64 = 10;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn external_search_provider_contract_smoke() {
    if env::var(ENABLE_ENV).as_deref() != Ok("1") {
        eprintln!("skipped: set {ENABLE_ENV}=1 to run external search provider smoke test");
        return;
    }

    let base_url = env::var(BASE_URL_ENV)
        .expect("CM_AGENT_SEARCH_BASE_URL is required when external search smoke is enabled");
    let adapter = SearchAdapterConfig {
        enabled: true,
        provider: Some(env_or_default(PROVIDER_ENV, DEFAULT_PROVIDER)),
        base_url: Some(base_url),
        search_endpoint: Some(env_or_default(ENDPOINT_ENV, DEFAULT_ENDPOINT)),
        api_key: env::var(API_KEY_ENV).ok(),
        timeout_sec: Some(env_u64(TIMEOUT_SEC_ENV, DEFAULT_TIMEOUT_SEC)),
        default_topic: Some(env_or_default(TOPIC_ENV, DEFAULT_TOPIC)),
        default_days: Some(env_i64(DAYS_ENV, DEFAULT_DAYS) as u64),
        default_max_results: Some(env_i64(MAX_RESULTS_ENV, DEFAULT_MAX_RESULTS) as u64),
    };
    let request = SearchProviderSmokeRequest {
        query: env_or_default(QUERY_ENV, DEFAULT_QUERY),
        topic: env_or_default(TOPIC_ENV, DEFAULT_TOPIC),
        days: env_i64(DAYS_ENV, DEFAULT_DAYS),
        max_results: Some(env_i64(MAX_RESULTS_ENV, DEFAULT_MAX_RESULTS)),
        bypass_cache: env_bool(BYPASS_CACHE_ENV, true),
    };

    let report = run_search_provider_smoke(&adapter, request)
        .await
        .expect("external search provider smoke should complete");
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&report.as_json())
            .expect("search smoke report should serialize")
    );

    assert!(
        matches!(
            report.adapter_status.as_str(),
            "live" | "degraded" | "adapter_required"
        ),
        "unexpected adapter_status: {}",
        report.adapter_status
    );
    if report.adapter_status == "live" {
        assert!(
            report.source_count > 0,
            "live search must include at least one accepted source"
        );
        assert_eq!(report.source_state, "has_reliable_sources");
    } else {
        assert_ne!(
            report.source_state, "has_reliable_sources",
            "degraded search must not claim reliable live sources"
        );
    }
    assert!(
        report.contract.ok,
        "external search provider response must satisfy evidence contract: {:?}",
        report.contract.failures
    );
    assert!(
        report
            .contract
            .evidence_packet_marker
            .starts_with("search_evidence_packet:"),
        "smoke report must include a replayable search_evidence_packet marker"
    );
    assert!(
        report
            .contract
            .evidence_trace
            .starts_with("search_evidence_trace:"),
        "smoke report must include a replayable search_evidence_trace marker"
    );
}

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn env_i64(key: &str, default: i64) -> i64 {
    env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .and_then(|value| match value.as_str() {
            "1" | "true" | "yes" | "y" | "on" => Some(true),
            "0" | "false" | "no" | "n" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}
