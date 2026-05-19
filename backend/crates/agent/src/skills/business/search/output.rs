use serde_json::{Value, json};

use super::{
    config::{
        DEFAULT_MAX_RESULTS, MAX_DAYS, MAX_RESULTS, SEARCH_ADAPTER_REQUIRED_CONFIG,
        SEARCH_DEFERRED_REASON, SEARCH_DEGRADED_REASON, SEARCH_PROVIDER_EMPTY_REASON,
        SEARCH_PROVIDER_STALE_REASON, SearchAdapterConfig,
    },
    provider::{
        build_provenance, build_search_evidence, estimate_confidence, normalize_sources,
        normalize_topic, utc_now_rfc3339,
    },
};
use crate::skills::SkillOutcome;

#[derive(Debug, Clone)]
pub(super) struct SearchPlan {
    pub(super) raw: String,
    pub(super) planned: String,
    pub(super) topic: String,
    pub(super) days: i64,
    pub(super) max_results: i64,
    pub(super) bypass_cache: bool,
    pub(super) product_ref: Option<String>,
    pub(super) product_name: Option<String>,
    pub(super) product_category: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct SearchSourceSet {
    pub(super) sources: Vec<Value>,
    pub(super) stale_filtered_count: usize,
}

impl SearchPlan {
    pub(super) fn new(
        raw: impl Into<String>,
        planned: impl Into<String>,
        topic: impl Into<String>,
        days: i64,
    ) -> Self {
        Self {
            raw: raw.into(),
            planned: planned.into(),
            topic: topic.into(),
            days: days.max(1),
            max_results: DEFAULT_MAX_RESULTS,
            bypass_cache: false,
            product_ref: None,
            product_name: None,
            product_category: None,
        }
    }

    pub(super) fn with_bypass_cache(mut self, bypass_cache: bool) -> Self {
        self.bypass_cache = bypass_cache;
        self
    }

    pub(super) fn with_product_context(
        mut self,
        product_ref: Option<String>,
        product_name: Option<String>,
        product_category: Option<String>,
    ) -> Self {
        self.product_ref = product_ref;
        self.product_name = product_name;
        self.product_category = product_category;
        self
    }

    pub(super) fn with_adapter_defaults(
        mut self,
        adapter: &SearchAdapterConfig,
    ) -> Result<Self, crate::skills::SkillError> {
        self.topic = normalize_topic(&self.topic, adapter.default_topic.as_deref())?;
        self.days = self.days.clamp(1, MAX_DAYS);
        self.max_results = adapter
            .default_max_results
            .unwrap_or(DEFAULT_MAX_RESULTS as u64)
            .clamp(1, MAX_RESULTS as u64) as i64;
        Ok(self)
    }

    pub(super) fn with_max_results(mut self, max_results: Option<i64>) -> Self {
        if let Some(value) = max_results {
            self.max_results = value.clamp(1, MAX_RESULTS);
        }
        self
    }
}

pub(super) fn deferred_search_output(
    skill_id: &str,
    required_input: String,
    plan: SearchPlan,
    adapter: &SearchAdapterConfig,
) -> SkillOutcome {
    let readiness = adapter.readiness();
    SkillOutcome::new(json!({
        "status": "deferred",
        "adapter_status": "adapter_required",
        "adapter": {
            "kind": "search",
            "provider": null,
            "configured": false,
            "required_config": SEARCH_ADAPTER_REQUIRED_CONFIG,
            "readiness": readiness
        },
        "deferred_reason": SEARCH_DEFERRED_REASON,
        "skill": skill_id,
        "required_input": required_input,
        "planned_query": plan.planned,
        "query": {
            "raw": plan.raw,
            "planned": plan.planned,
            "topic": plan.topic,
            "days": plan.days,
            "max_results": plan.max_results,
            "bypass_cache": plan.bypass_cache,
            "product_ref": plan.product_ref,
            "product_name": plan.product_name,
            "product_category": plan.product_category
        },
        "data": {
            "items": []
        },
        "sources": [],
        "provenance": {
            "source_count": 0,
            "domain_count": 0,
            "domains": [],
            "published_count": 0,
            "latest_published_at": null,
            "stale_filtered_count": 0
        },
        "confidence": 0.0,
        "freshness": {
            "requested_days": plan.days,
            "requested_at": null,
            "stale_filtered_count": 0
        },
        "evidence": build_search_evidence(&[], 0),
        "degraded": true,
        "degraded_reason": SEARCH_DEGRADED_REASON,
        "expected_outputs": ["分析摘要", "结构化要点", "sources", "engine"]
    }))
    .with_summary("外部搜索 adapter 未接入，已返回结构化降级契约")
    .with_metadata("adapter_status", json!("adapter_required"))
    .with_metadata("adapter_kind", json!("search"))
    .with_metadata("adapter_readiness", readiness)
}

pub(super) fn degraded_search_output(
    skill_id: &str,
    required_input: String,
    plan: SearchPlan,
    adapter: &SearchAdapterConfig,
    reason: impl Into<String>,
    stale_filtered_count: usize,
) -> SkillOutcome {
    let reason = reason.into();
    let readiness = adapter.readiness();
    SkillOutcome::new(json!({
        "status": "deferred",
        "adapter_status": "degraded",
        "adapter": {
            "kind": "search",
            "provider": adapter.provider_name(),
            "configured": adapter.is_configured(),
            "required_config": if adapter.is_configured() { Value::Null } else { json!(SEARCH_ADAPTER_REQUIRED_CONFIG) },
            "readiness": readiness
        },
        "deferred_reason": "外部搜索 adapter 已配置但本次未拿到可采信来源，不能伪造实时来源",
        "skill": skill_id,
        "required_input": required_input,
        "planned_query": plan.planned,
        "query": {
            "raw": plan.raw,
            "planned": plan.planned,
            "topic": plan.topic,
            "days": plan.days,
            "max_results": plan.max_results,
            "bypass_cache": plan.bypass_cache,
            "product_ref": plan.product_ref,
            "product_name": plan.product_name,
            "product_category": plan.product_category
        },
        "engine": adapter.provider_name(),
        "data": {
            "items": []
        },
        "sources": [],
        "provenance": {
            "source_count": 0,
            "domain_count": 0,
            "domains": [],
            "published_count": 0,
            "latest_published_at": null,
            "stale_filtered_count": stale_filtered_count
        },
        "confidence": 0.0,
        "freshness": {
            "requested_days": plan.days,
            "requested_at": utc_now_rfc3339(),
            "stale_filtered_count": stale_filtered_count
        },
        "evidence": build_search_evidence(&[], stale_filtered_count),
        "notice": "未获取到可靠公开来源，当前不能伪造实时搜索结果。",
        "degraded": true,
        "degraded_reason": reason,
        "expected_outputs": ["分析摘要", "结构化要点", "sources", "engine"]
    }))
    .with_summary("外部搜索 adapter 未返回可采信来源，已按降级结果处理")
    .with_metadata("adapter_status", json!("degraded"))
    .with_metadata("adapter_kind", json!("search"))
    .with_metadata("provider", json!(adapter.provider_name()))
    .with_metadata("stale_filtered_count", json!(stale_filtered_count))
    .with_metadata("adapter_readiness", readiness)
}

pub(super) fn live_search_output(
    skill_id: &str,
    required_input: String,
    plan: SearchPlan,
    adapter: &SearchAdapterConfig,
    payload: &Value,
) -> SkillOutcome {
    let source_set = normalize_sources(payload, plan.max_results, plan.days);
    let sources = source_set.sources;
    if sources.is_empty() {
        let reason = if source_set.stale_filtered_count > 0 {
            SEARCH_PROVIDER_STALE_REASON
        } else {
            SEARCH_PROVIDER_EMPTY_REASON
        };
        return degraded_search_output(
            skill_id,
            required_input,
            plan,
            adapter,
            reason,
            source_set.stale_filtered_count,
        );
    }

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
        .unwrap_or_else(|| adapter.provider_name());
    let requested_at = payload
        .get("requested_at")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(utc_now_rfc3339);
    let source_count = provenance
        .get("source_count")
        .and_then(Value::as_u64)
        .unwrap_or(sources.len() as u64);
    let readiness = adapter.readiness();

    SkillOutcome::new(json!({
        "status": "live",
        "adapter_status": "live",
        "adapter": {
            "kind": "search",
            "provider": adapter.provider_name(),
            "configured": true,
            "required_config": null,
            "readiness": readiness
        },
        "skill": skill_id,
        "required_input": required_input,
        "planned_query": plan.planned,
        "query": {
            "raw": plan.raw,
            "planned": plan.planned,
            "topic": plan.topic,
            "days": plan.days,
            "max_results": plan.max_results,
            "bypass_cache": plan.bypass_cache,
            "product_ref": plan.product_ref,
            "product_name": plan.product_name,
            "product_category": plan.product_category
        },
        "key": payload.get("key").and_then(Value::as_str).unwrap_or("search"),
        "engine": engine,
        "data": {
            "items": sources
        },
        "sources": sources,
        "provenance": provenance,
        "confidence": confidence,
        "freshness": {
            "requested_days": plan.days,
            "requested_at": requested_at,
            "stale_filtered_count": source_set.stale_filtered_count
        },
        "evidence": evidence,
        "degraded": false,
        "degraded_reason": null,
        "expected_outputs": ["分析摘要", "结构化要点", "sources", "engine"]
    }))
    .with_summary(format!("外部搜索已返回 {source_count} 个可采信来源"))
    .with_metadata("adapter_status", json!("live"))
    .with_metadata("adapter_kind", json!("search"))
    .with_metadata("provider", json!(adapter.provider_name()))
    .with_metadata("source_count", json!(source_count))
    .with_metadata("adapter_readiness", readiness)
    .with_metadata(
        "stale_filtered_count",
        json!(source_set.stale_filtered_count),
    )
}
