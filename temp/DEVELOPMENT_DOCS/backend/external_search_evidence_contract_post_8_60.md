# Post 8-60: External Search Evidence Contract Hardening

## 1. Requirement Summary

The old product has six search skills:

- `search_trends`
- `search_competitor`
- `search_market_info`
- `search_product_reviews`
- `search_platform_policy`
- `search_industry_benchmarks`

The new product already has Rust Agent search skills and an `[external_search]` adapter contract. This slice continues the backend-first migration by making search outputs more reusable for Agent answers, workspace artifacts, and later Kimi-style source panels.

This is not a Taobao/Douyin official platform integration. It only hardens the generic external search adapter contract. If no provider is configured or no credible source is returned, the backend must still degrade explicitly and must not fabricate live sources.

## 2. Business Understanding

Users need real-time market, competitor, review, platform policy, and benchmark information. The backend must separate three states:

1. Adapter is not configured.
2. Adapter is configured but did not return acceptable sources.
3. Adapter returned live sources that can be reused as evidence.

The UI later needs a clean source panel, but the source panel should consume real backend evidence fields instead of parsing raw provider payloads.

## 3. Database Design

No new tables or migrations are needed.

Search evidence that comes from a successful Agent `search_*` skill already persists through existing `workspace_artifacts` as:

- `artifactType = search_evidence`
- `storageKind = inline`
- `format = json`

This slice only enriches the JSON contract stored in that existing artifact path.

## 4. API / Contract Design

### Existing App API

`GET /api/app/external-search/status`

No response change.

`GET /api/app/external-search/search`

Response adds:

- `maxResults`: normalized result limit actually used by the backend.
- `bypassCache`: whether this request asked the provider to bypass cache.
- `evidence`: normalized evidence object for frontend and Agent reuse.

`evidence` shape:

```json
{
  "summary": "共 2 条实时来源，覆盖 2 个域名；回答应优先引用这些来源，并注明时效性和置信度。",
  "sourceTitles": ["咖啡消费趋势", "咖啡行业报告"],
  "sourceDomains": ["example.com", "report.example.cn"],
  "llmContext": "（共2条实时来源...）"
}
```

When no live sources exist, `evidence` is still present but explicitly empty and says the result is degraded.

### Agent Skill Output

All six Agent `search_*` skills now include:

- `query.bypass_cache`
- `evidence.summary`
- `evidence.source_titles`
- `evidence.source_domains`
- `evidence.llm_context`

Agent search provider requests now forward `bypass_cache=true` when the input contains:

- `bypass_cache`
- `bypassCache`
- `_needs_fresh_data`

This preserves the old product's "needs fresh data" intent without importing the old Python runtime.

## 5. Auth Design

No auth change.

The App API still requires a valid App user JWT and an enabled `app_users` record. Agent SSE still runs through the existing authenticated session flow.

## 6. Error / Degradation Design

The backend still returns successful HTTP responses with `status=degraded` when:

- `[external_search]` is not configured.
- provider HTTP request fails.
- provider response cannot be decoded.
- provider returns no normalized source.

No fake source, fake engine, or fake evidence artifact is created in those cases.

Provider result deduplication is now part of normalization. Duplicate sources are removed by normalized URL first, and by title when URL is absent.

## 7. Frontend Impact

No frontend code is changed in this slice.

Later frontend work can use:

- `evidence.summary` for compact source-panel status.
- `evidence.sourceTitles` and `evidence.sourceDomains` for collapsed source cards.
- `evidence.llmContext` only as backend-provided structured evidence text, not as a user-facing long paragraph by default.

## 8. Validation Plan

Run from `backend/`:

```bash
cargo fmt --all
cargo test -p agent search -- --nocapture
cargo test -p service external_search -- --nocapture
cargo check -p agent -p service
cargo clippy -p agent -p service
git diff --check
```
