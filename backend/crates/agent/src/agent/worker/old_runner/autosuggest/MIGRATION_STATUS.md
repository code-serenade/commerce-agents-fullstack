# Autosuggest Migration Status

This directory is frozen reference code for the old runner.

Files should be deleted only after their behavior is represented by:

1. `backend/config/json/agent/worker-autosuggest-policy.json`
2. `backend/crates/agent/src/agent/worker/runner/autosuggest.rs`
3. later `runner/inference.rs` when the old behavior was input inference rather than routing

## Removed

`chat.rs`

The chat memory/status routing is now handled by the new config-driven runner autosuggest path:

- `chat.save_memory_keywords`
- `chat.status_query_keywords`
- `coordination_save_memory`
- `coordination_status_query`

## Migrated In Place

`search.rs`

The external search routing part has been moved to `worker-autosuggest-policy.json` rules:

- `search.platform_policy`
- `search.product_reviews`
- `search.industry_benchmarks`
- `search.competitor`
- `search.trends`
- `search.market_info`

The file is not deleted yet because it still contains metric query autosuggest helpers:

- `autosuggest_query_store_metrics`
- `autosuggest_product_or_campaign_metrics`

Those remaining helpers depend on metrics input inference and should move with the inference phase.

## Partially Migrated

The following files still contain old behavior that has only been partially moved:

- `service.rs`: first-level service skill routing is configured; benchmark/policy/review search chains still need migration.
- `web.rs`: first-level web skill routing is configured; SEO/search chain details still need migration.
- `design.rs`: first-level design skill routing is configured; trend/material policy search chains still need migration.
- `engineering.rs`: first-level engineering skill routing is configured; external knowledge search chain still needs migration.
- `douyin_ads.rs`: first-level ads skill routing is configured; search chain details still need migration.
- `creative.rs`: first-level creative skill routing is configured; trend/calendar search chains still need migration.
- `accounting.rs`: first-level accounting skill routing is configured; finance search chain and numeric inference still need migration.
- `data.rs`: first-level dashboard/diagnosis/channel ROI routing is configured; most metrics and numeric inference still need migration.
- `ops.rs`: not yet migrated as a complete file; contains many routing and search-chain references.
- `search.rs`: only metric query routing remains as reference material.

## Still Shared Reference

- `common.rs`
- `policy.rs`

These files remain as old-runner reference only. They are not compiled by the new runner.
