# Post 8-113 Search Autosuggest Max Results Parsing

## Requirement Summary

Post 8-112 added `max_results` / `maxResults` to the Agent search skill contract. This slice connects that backend contract to natural-language search intent so the worker autosuggest layer can preserve user requests such as "查 3 条来源", "最多 5 条结果", or "top 4 sources".

## Scope

- Backend Agent worker autosuggest only.
- No database change.
- No new HTTP route.
- No frontend change.
- No fake provider or fake source.

## Existing Architecture

- Search intent routing lives in `backend/crates/agent/src/agent/worker/runner.rs`.
- `autosuggest_search` selects one of six `search_*` skills.
- Helper builders such as `search_trends_input`, `search_competitor_input`, and `search_platform_policy_input` construct structured skill input.
- Search skill runtime now accepts optional `max_results` / `maxResults`.

## Design

### Data Tables

No table is added or changed.

### Skill Input Behavior

When a natural-language prompt contains a result-count expression near search/source/result wording, autosuggest adds:

```json
{
  "max_results": 3
}
```

Supported examples:

- `查3条来源`
- `查 3 个来源`
- `最多5条结果`
- `返回4条公开资料`
- `top 4 sources`

The parser should avoid using unrelated numbers such as:

- `最近30天`
- `2026`
- `618`
- price, budget, or metric values

Resolution:

- If no result-count expression exists, do not add `max_results`.
- If a count is found, clamp to `1..=12`.
- The downstream search skill runtime still performs its own defensive clamp.

### API / SSE Contract

No new event is added.

Existing `skill_requested.input.max_results` may now appear when autosuggest infers it from the user's wording. Existing `skill_result.output.query.max_results` remains the normalized source limit.

### Authentication

Unchanged. This logic runs only inside the existing Agent session.

### Error and Degraded Handling

Unchanged. A parsed result count does not imply provider availability. Missing provider, provider failure, and no trusted sources still return degraded/deferred output.

## Implementation Steps

1. Add a helper to infer search `max_results` from natural language.
2. Add a helper to insert inferred `max_results` into every search input map.
3. Apply it to all six `search_*` autosuggest input builders.
4. Add worker autosuggest tests covering Chinese and English count expressions and non-result numbers.
5. Update API/SSE docs and migration logs.

## Validation

- `cargo fmt --all`
- `cargo test -p agent search`
- `cargo check -p agent -p web-server`
- `cargo clippy -p agent -p web-server`
- `git diff --check`
