# Post 8-77 Data Funnel Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_funnel_analysis` skill so product-bound funnel analysis can carry the selected-product reference:

```text
product_ref = product:{uuid}
```

The skill still only uses explicit funnel metrics from the current turn. It must not pretend to fetch live platform funnel data.

## Scope

- Agent data skill: `data_funnel_analysis`
- Data worker autosuggest for funnel / conversion analysis
- SSE/API documentation
- Migration logs and matrix

## Out Of Scope

- No database migration.
- No HTTP API route change.
- No frontend code change.
- No official platform adapter integration.
- No fake live funnel metrics.

## Data Contract

`data_funnel_analysis` accepts:

- `product_ref?: string`
- `impressions?: integer`
- `clicks?: integer`
- `add_to_cart?: integer`
- `orders?: integer`
- `payments?: integer`
- `platform?: string`
- `days?: integer`

`product_ref` is an identity field only. Funnel calculations still depend on explicit funnel metrics or future service/adapter injected metrics.

## Behavior

- Successful output includes `商品引用`.
- Deferred output already echoes `received_params`; when a `product_ref` is supplied it remains visible there.
- Worker autosuggest passes `product_ref` only from explicit `context.selected_summary`.
- Without selected product context, autosuggest does not add an empty `product_ref`.

## API / Auth

No new API route. Existing authenticated Agent SSE flow is unchanged:

```text
POST /api/sse/agent/session/stream
```

The field appears inside existing `skill_requested.input` and `skill_result.output`.

## Validation

- `cargo fmt --all`
- `cargo test -p agent data_funnel_analysis -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_funnel_analysis_ref -- --nocapture`
- `cargo test -p agent funnel_analysis_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
