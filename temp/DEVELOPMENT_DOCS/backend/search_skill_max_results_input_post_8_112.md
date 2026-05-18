# Post 8-112 Search Skill Max Results Input

## Requirement Summary

Post 8-111 made App external search and Agent `search_*` normalization enforce the configured result limit after provider response normalization. This slice makes result count a first-class Agent skill input too, matching the old product's search strategy controls and the new product's backend-first capability contract.

## Scope

- Backend Agent skill contract only.
- No database change.
- No new HTTP route.
- No frontend change.
- No fake provider, fake source, or fake search result.

## Existing Architecture

- Search skill definitions live in `backend/crates/agent/src/skills/search.rs`.
- The worker or model can invoke six search skills:
  - `search_trends`
  - `search_competitor`
  - `search_market_info`
  - `search_product_reviews`
  - `search_platform_policy`
  - `search_industry_benchmarks`
- `SearchAdapterConfig.default_max_results` currently controls the default result count.
- Provider request already sends `max_results`.
- Post 8-111 already enforces `SearchPlan.max_results` during provider source normalization.

## Design

### Data Tables

No table is added or changed.

### Skill Input Contract

Each `search_*` skill declares an optional common input:

```json
{
  "name": "max_results",
  "required": false,
  "schema": {
    "type": "integer",
    "minimum": 1,
    "maximum": 12
  }
}
```

Runtime parsing accepts both:

- `max_results`
- `maxResults`

Resolution order:

1. If input provides `max_results`, use it.
2. Else if input provides `maxResults`, use it.
3. Else use `[external_search].default_max_results`.
4. Always clamp to `1..=12`.

### API / SSE Contract

No new event or route is added.

Existing `skill_result.output.query.max_results` now reflects either the explicit skill input or the adapter default. Provider request, live sources, `data.items`, `provenance`, `evidence`, and any search evidence artifact remain aligned with this value.

### Authentication

Unchanged. Search skills still run only inside the existing Agent runtime/session boundary.

### Error and Degraded Handling

Unchanged.

- Invalid non-integer `max_results/maxResults` returns the existing invalid parameter error.
- Out-of-range integer values are clamped to `1..=12`.
- Provider failure or missing trusted sources still returns degraded/deferred output.

## Implementation Steps

1. Add a common `max_results` optional input to every `search_*` skill spec.
2. Parse `max_results` / `maxResults` in search skill runtime.
3. Apply explicit input after adapter defaults so user/request-level strategy wins.
4. Add tests covering snake_case, camelCase, and clamping behavior.
5. Update API/SSE docs and migration logs.

## Validation

Targeted validation:

- `cargo fmt --all`
- `cargo test -p agent search`
- `cargo check -p agent -p web-server`
- `cargo clippy -p agent -p web-server`
- `git diff --check`
