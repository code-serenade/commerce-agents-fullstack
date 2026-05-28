use serde_json::{Value, json};

use super::{
    config::SearchAdapterConfig,
    output::{SearchPlan, live_search_output},
    provider::{normalize_sources, query_provider},
};
use crate::skills::SkillOutcome;

const DEFAULT_SMOKE_SKILL_ID: &str = "search_market_info";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchProviderSmokeRequest {
    pub query: String,
    pub topic: String,
    pub days: i64,
    pub max_results: Option<i64>,
    pub bypass_cache: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchProviderSmokeReport {
    pub raw_payload: Value,
    pub output: Value,
    pub normalized_sources: Value,
    pub filter_stats: Value,
    pub metadata: Value,
    pub contract: SearchProviderSmokeContractCheck,
    pub adapter_status: String,
    pub source_state: String,
    pub source_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchProviderSmokeContractCheck {
    pub ok: bool,
    pub matrix_state: String,
    pub evidence_packet: Value,
    pub evidence_packet_marker: String,
    pub evidence_trace: String,
    pub failures: Vec<String>,
    pub warnings: Vec<String>,
}

impl SearchProviderSmokeRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            topic: "general".to_string(),
            days: 30,
            max_results: None,
            bypass_cache: true,
        }
    }
}

impl SearchProviderSmokeReport {
    pub fn as_json(&self) -> Value {
        json!({
            "adapter_status": self.adapter_status,
            "source_state": self.source_state,
            "source_count": self.source_count,
            "contract": self.contract.as_json(),
            "filter_stats": self.filter_stats,
            "metadata": self.metadata,
            "normalized_sources": self.normalized_sources,
            "output": self.output,
            "raw_payload": self.raw_payload
        })
    }
}

impl SearchProviderSmokeContractCheck {
    pub fn as_json(&self) -> Value {
        json!({
            "ok": self.ok,
            "matrix_state": self.matrix_state,
            "evidence_packet": self.evidence_packet,
            "evidence_packet_marker": self.evidence_packet_marker,
            "evidence_trace": self.evidence_trace,
            "failures": self.failures,
            "warnings": self.warnings
        })
    }
}

pub async fn run_search_provider_smoke(
    adapter: &SearchAdapterConfig,
    request: SearchProviderSmokeRequest,
) -> Result<SearchProviderSmokeReport, String> {
    let required_input = request.query.trim().to_string();
    if required_input.is_empty() {
        return Err("search smoke query must not be empty".to_string());
    }

    let plan = SearchPlan::new(
        required_input.clone(),
        required_input.clone(),
        request.topic,
        request.days,
    )
    .with_bypass_cache(request.bypass_cache)
    .with_adapter_defaults(adapter)
    .map_err(|err| err.to_string())?
    .with_max_results(request.max_results);

    let raw_payload = query_provider(adapter, &plan).await?;
    let source_set = normalize_sources(&raw_payload, plan.max_results, plan.days);
    let normalized_sources = json!(source_set.sources.clone());
    let filter_stats = source_set.filter_stats.as_json();
    let outcome = live_search_output(
        DEFAULT_SMOKE_SKILL_ID,
        required_input,
        plan,
        adapter,
        &raw_payload,
    );
    Ok(build_search_provider_smoke_report(
        raw_payload,
        normalized_sources,
        filter_stats,
        outcome,
    ))
}

fn build_search_provider_smoke_report(
    raw_payload: Value,
    normalized_sources: Value,
    filter_stats: Value,
    outcome: SkillOutcome,
) -> SearchProviderSmokeReport {
    let output = outcome.output;
    let adapter_status = output
        .get("adapter_status")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let source_state = output
        .pointer("/evidence_packet/source_state")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let source_count = output
        .get("sources")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let metadata = Value::Object(outcome.metadata);
    let contract = build_contract_check(
        &output,
        adapter_status.as_str(),
        source_state.as_str(),
        source_count,
    );

    SearchProviderSmokeReport {
        raw_payload,
        output,
        normalized_sources,
        filter_stats,
        metadata,
        contract,
        adapter_status,
        source_state,
        source_count,
    }
}

fn build_contract_check(
    output: &Value,
    adapter_status: &str,
    source_state: &str,
    source_count: usize,
) -> SearchProviderSmokeContractCheck {
    let evidence_packet = output
        .get("evidence_packet")
        .cloned()
        .unwrap_or(Value::Null);
    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    if evidence_packet.is_null() {
        failures.push("missing output.evidence_packet".to_string());
    }

    let packet_adapter_status = value_string(&evidence_packet, "adapter_status");
    if packet_adapter_status.as_deref() != Some(adapter_status) {
        failures.push(format!(
            "adapter_status mismatch: output={adapter_status}, packet={}",
            packet_adapter_status.unwrap_or_else(|| "missing".to_string())
        ));
    }

    let packet_source_state = value_string(&evidence_packet, "source_state");
    if packet_source_state.as_deref() != Some(source_state) {
        failures.push(format!(
            "source_state mismatch: output={source_state}, packet={}",
            packet_source_state.unwrap_or_else(|| "missing".to_string())
        ));
    }

    let packet_source_count = array_len(&evidence_packet, "sources");
    let usable_claim_count = array_len(&evidence_packet, "usable_claims");
    let gap_count = array_len(&evidence_packet, "gaps");
    let filter_stats = SmokeFilterStats::from_output(output, &evidence_packet);

    if filter_stats.total_filtered_count != filter_stats.component_total() {
        failures.push(format!(
            "filter_stats total mismatch: total_filtered_count={} but component sum={}",
            filter_stats.total_filtered_count,
            filter_stats.component_total()
        ));
    }

    match adapter_status {
        "live" => {
            if source_state != "has_reliable_sources" {
                failures.push(format!(
                    "live adapter must expose has_reliable_sources, got {source_state}"
                ));
            }
            if source_count == 0 || packet_source_count == 0 {
                failures.push(
                    "live adapter must include accepted sources in output and evidence_packet"
                        .to_string(),
                );
            }
            if usable_claim_count == 0 {
                warnings.push(
                    "live adapter returned sources but no usable_claims; downstream answer may \
                     lack concrete evidence"
                        .to_string(),
                );
            }
            if source_count < 3 {
                warnings.push(format!(
                    "live adapter returned only {source_count} accepted source(s); downstream \
                     roles must treat it as low confidence"
                ));
            }
        }
        "degraded" => {
            if source_state == "has_reliable_sources" {
                failures.push("degraded adapter must not claim has_reliable_sources".to_string());
            }
            if source_count > 0 || packet_source_count > 0 {
                failures.push(
                    "degraded adapter must not expose accepted sources as reliable evidence"
                        .to_string(),
                );
            }
            if gap_count == 0 {
                failures.push("degraded adapter must include at least one gap".to_string());
            }
        }
        "adapter_required" => {
            if source_state != "adapter_required" {
                failures.push(format!(
                    "adapter_required status must expose adapter_required source_state, got \
                     {source_state}"
                ));
            }
            if source_count > 0 || packet_source_count > 0 {
                failures
                    .push("adapter_required result must not include accepted sources".to_string());
            }
        }
        _ => {
            failures.push(format!("unexpected adapter_status: {adapter_status}"));
        }
    }

    let evidence_packet_marker = evidence_packet_marker(&evidence_packet);
    let evidence_trace = evidence_trace_from_output(output, &evidence_packet, source_count);
    for required in [
        "search_evidence_trace:",
        "adapter_status=",
        "sources=",
        "filtered=",
        "stale=",
        "missing_url=",
        "untraceable=",
        "duplicate=",
    ] {
        if !evidence_trace.contains(required) {
            failures.push(format!("evidence_trace missing marker `{required}`"));
        }
    }

    SearchProviderSmokeContractCheck {
        ok: failures.is_empty(),
        matrix_state: matrix_state(adapter_status, source_state, &filter_stats),
        evidence_packet,
        evidence_packet_marker,
        evidence_trace,
        failures,
        warnings,
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct SmokeFilterStats {
    missing_url_filtered_count: usize,
    untraceable_domain_filtered_count: usize,
    duplicate_filtered_count: usize,
    stale_filtered_count: usize,
    total_filtered_count: usize,
}

impl SmokeFilterStats {
    fn from_output(output: &Value, packet: &Value) -> Self {
        let stats = packet
            .get("filter_stats")
            .or_else(|| output.get("filter_stats"))
            .unwrap_or(&Value::Null);
        Self {
            missing_url_filtered_count: value_usize(stats, "missing_url_filtered_count"),
            untraceable_domain_filtered_count: value_usize(
                stats,
                "untraceable_domain_filtered_count",
            ),
            duplicate_filtered_count: value_usize(stats, "duplicate_filtered_count"),
            stale_filtered_count: value_usize(stats, "stale_filtered_count"),
            total_filtered_count: value_usize(stats, "total_filtered_count"),
        }
    }

    fn component_total(&self) -> usize {
        self.missing_url_filtered_count
            + self.untraceable_domain_filtered_count
            + self.duplicate_filtered_count
            + self.stale_filtered_count
    }
}

fn matrix_state(adapter_status: &str, source_state: &str, stats: &SmokeFilterStats) -> String {
    match adapter_status {
        "live" if stats.duplicate_filtered_count > 0 => "live_duplicate_filtered".to_string(),
        "live" if stats.total_filtered_count > 0 => "live_filtered".to_string(),
        "live" => "live".to_string(),
        "degraded" if stats.stale_filtered_count > 0 => "degraded_stale".to_string(),
        "degraded"
            if stats.missing_url_filtered_count > 0
                || stats.untraceable_domain_filtered_count > 0 =>
        {
            "degraded_untraceable".to_string()
        }
        "degraded" => "degraded_no_reliable_sources".to_string(),
        "adapter_required" => "adapter_required".to_string(),
        _ => format!("{adapter_status}_{source_state}"),
    }
}

fn evidence_packet_marker(packet: &Value) -> String {
    serde_json::to_string(packet)
        .map(|packet| format!("search_evidence_packet:{packet}"))
        .unwrap_or_else(|_| "search_evidence_packet:null".to_string())
}

fn evidence_trace_from_output(output: &Value, packet: &Value, source_count: usize) -> String {
    let stats = SmokeFilterStats::from_output(output, packet);
    let skill = value_string(packet, "skill").unwrap_or_else(|| DEFAULT_SMOKE_SKILL_ID.to_string());
    let status = value_string(output, "status").unwrap_or_else(|| "unknown".to_string());
    let adapter_status = value_string(output, "adapter_status")
        .or_else(|| value_string(packet, "adapter_status"))
        .unwrap_or_else(|| "unknown".to_string());
    let intent =
        value_string(packet, "intent").unwrap_or_else(|| "general_search_evidence".to_string());
    let strength = value_string(packet, "search_strength").unwrap_or_else(|| "unknown".to_string());
    let query = value_string(packet, "query")
        .or_else(|| value_string(output, "planned_query"))
        .unwrap_or_default();
    let usable_claims = array_len(packet, "usable_claims");
    let gaps = array_len(packet, "gaps");
    let confidence = packet
        .get("confidence")
        .and_then(Value::as_f64)
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "unknown".to_string());

    let mut trace = format!(
        "search_evidence_trace: skill={} status={} adapter_status={} intent={} strength={} \
         sources={} usable_claims={} gaps={} confidence={} filtered={} stale={} missing_url={} \
         untraceable={} duplicate={} query=\"{}\"",
        compact_token(&skill),
        compact_token(&status),
        compact_token(&adapter_status),
        compact_token(&intent),
        compact_token(&strength),
        source_count,
        usable_claims,
        gaps,
        confidence,
        stats.total_filtered_count,
        stats.stale_filtered_count,
        stats.missing_url_filtered_count,
        stats.untraceable_domain_filtered_count,
        stats.duplicate_filtered_count,
        compact_quoted(&query, 180)
    );

    let gap_preview = packet
        .get("gaps")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .take(2)
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .unwrap_or_default();
    if !gap_preview.is_empty() {
        trace.push_str(" gap_preview=\"");
        trace.push_str(&compact_quoted(&gap_preview, 220));
        trace.push('"');
    }

    trace
}

fn value_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn value_usize(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
}

fn array_len(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

fn compact_token(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join("_")
}

fn compact_quoted(value: &str, max_chars: usize) -> String {
    let value = value.replace('"', "'");
    if value.chars().count() <= max_chars {
        return value;
    }
    let mut text = value.chars().take(max_chars).collect::<String>();
    text.push_str("...");
    text
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{build_search_provider_smoke_report, live_search_output, normalize_sources};
    use crate::skills::business::search::{config::SearchAdapterConfig, output::SearchPlan};

    fn adapter() -> SearchAdapterConfig {
        SearchAdapterConfig {
            enabled: true,
            provider: Some("codex-web-search".to_string()),
            base_url: Some("http://search.test".to_string()),
            search_endpoint: Some("/api/external/search".to_string()),
            ..SearchAdapterConfig::default()
        }
    }

    #[test]
    fn smoke_report_contract_builds_replay_case_for_live_filtered_payload() {
        let plan = SearchPlan::new(
            "校园冷萃咖啡 价格",
            "校园冷萃咖啡 价格 竞品 2026",
            "general",
            60,
        )
        .with_max_results(Some(6));
        let payload = json!({
            "engine": "codex-web-search",
            "confidence": 0.68,
            "data": {
                "items": [
                    {
                        "title": "冷萃咖啡竞品价格参考",
                        "url": "https://example.com/cold-brew-price",
                        "snippet": "竞品价格集中在 12-19 元。",
                        "published_at": "2026-05-20"
                    },
                    {
                        "title": "重复来源",
                        "url": "https://example.com/cold-brew-price",
                        "snippet": "重复来源。",
                        "published_at": "2026-05-20"
                    },
                    {
                        "title": "本地调试来源",
                        "url": "http://localhost/search",
                        "snippet": "不可追溯来源。"
                    }
                ]
            }
        });
        let source_set = normalize_sources(&payload, plan.max_results, plan.days);
        let normalized_sources = json!(source_set.sources.clone());
        let filter_stats = source_set.filter_stats.as_json();
        let outcome = live_search_output(
            "search_competitor",
            "校园冷萃咖啡 价格".to_string(),
            plan,
            &adapter(),
            &payload,
        );

        let report =
            build_search_provider_smoke_report(payload, normalized_sources, filter_stats, outcome);

        assert!(report.contract.ok, "{:?}", report.contract.failures);
        assert_eq!(report.adapter_status, "live");
        assert_eq!(report.source_state, "has_reliable_sources");
        assert_eq!(report.contract.matrix_state, "live_duplicate_filtered");
        assert!(
            report
                .contract
                .evidence_packet_marker
                .starts_with("search_evidence_packet:")
        );
        assert!(
            report
                .contract
                .evidence_trace
                .contains("adapter_status=live")
        );
        assert!(report.contract.evidence_trace.contains("sources=1"));
        assert!(report.contract.evidence_trace.contains("filtered=2"));
        assert!(report.contract.evidence_trace.contains("untraceable=1"));
        assert!(report.contract.evidence_trace.contains("duplicate=1"));
    }

    #[test]
    fn smoke_report_contract_builds_replay_case_for_stale_degraded_payload() {
        let plan = SearchPlan::new("抖音平台政策", "抖音 平台政策", "news", 30);
        let payload = json!({
            "engine": "codex-web-search",
            "data": {
                "items": [
                    {
                        "title": "旧平台政策",
                        "url": "https://policy.example.com/old",
                        "snippet": "旧政策。",
                        "published_at": "2000-01-01"
                    }
                ]
            }
        });
        let source_set = normalize_sources(&payload, plan.max_results, plan.days);
        let normalized_sources = json!(source_set.sources.clone());
        let filter_stats = source_set.filter_stats.as_json();
        let outcome = live_search_output(
            "search_platform_policy",
            "抖音平台政策".to_string(),
            plan,
            &adapter(),
            &payload,
        );

        let report =
            build_search_provider_smoke_report(payload, normalized_sources, filter_stats, outcome);

        assert!(report.contract.ok, "{:?}", report.contract.failures);
        assert_eq!(report.adapter_status, "degraded");
        assert_eq!(report.source_state, "no_reliable_sources");
        assert_eq!(report.source_count, 0);
        assert_eq!(report.contract.matrix_state, "degraded_stale");
        assert!(
            report
                .contract
                .evidence_trace
                .contains("adapter_status=degraded")
        );
        assert!(report.contract.evidence_trace.contains("sources=0"));
        assert!(report.contract.evidence_trace.contains("stale=1"));
        assert!(
            report
                .contract
                .evidence_packet_marker
                .contains("\"source_state\":\"no_reliable_sources\"")
        );
    }
}
