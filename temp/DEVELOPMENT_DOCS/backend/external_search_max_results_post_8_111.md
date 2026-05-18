# Post 8-111 External Search Max Results Enforcement

## Requirement Summary

The old product search skills expose result-count control as part of search strategy. In the new product, App external search and Agent `search_*` skills already send `maxResults` / `max_results` to the configured provider, but backend normalization still capped only by the global hard limit. This slice makes the requested limit part of the trusted backend contract.

## Scope

- Backend only.
- No database change.
- No new HTTP route.
- No frontend change.
- No fake search provider or fake live source.

## Existing Architecture

- App side search uses `ExternalSearchApi` in `backend/crates/service/src/api/external_search.rs`.
- Agent search skills use `SearchAdapterConfig` and `SearchPlan` in `backend/crates/agent/src/skills/search.rs`.
- Both paths request the external provider with `max_results`.
- Both paths normalize provider payload into trusted sources, provenance, confidence, evidence, and degraded state.

## Design

### Data Tables

No table is added or changed.

### API Contract

No route shape changes.

`GET /api/app/external-search/search` keeps the existing `maxResults` query parameter:

- allowed range: `1..=12`
- default: config `default_max_results` or `6`
- response `maxResults`: backend-normalized value used for this request
- response `data.items`, `provenance.sourceCount`, and `evidence` now reflect no more than that value

Agent `search_*` skill output keeps `output.query.max_results`:

- derived from `[external_search].default_max_results`
- clamped to `1..=12`
- `output.sources`, `output.data.items`, `output.provenance.source_count`, and `output.evidence` now reflect no more than that value

### Authentication

Unchanged.

- App search API still requires enabled App user.
- Agent SSE still runs through existing authenticated session path.

### Error and Degraded Handling

Unchanged.

- Provider failure or no trusted sources still returns degraded/deferred output.
- Sources without traceable URL are still filtered.
- If filtering leaves zero trusted sources, no fake live evidence or `search_evidence` artifact is created.

### Implementation Steps

1. Pass the request-level `ExternalSearchSpec.max_results` into App-side source normalization.
2. Pass `SearchPlan.max_results` into Agent-side source normalization.
3. Clamp the normalization limit to `1..=MAX_RESULTS` defensively.
4. Add tests proving provider payloads with extra sources are truncated before provenance/evidence are built.
5. Update `API_DOCS/` and migration logs.

## Validation

Targeted validation:

- `cargo fmt --all`
- `cargo test -p service external_search`
- `cargo test -p agent search`
- `cargo check -p service -p agent -p web-server`
- `cargo clippy -p service -p agent -p web-server`
- `git diff --check`
