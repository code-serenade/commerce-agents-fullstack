# Post 8-76 Data Competitor Product Ref

## Requirement Summary

Continue migrating old product skill capability into the new product framework. This slice extends the native `data_competitor_analysis` skill with the new selected-product reference contract:

```text
product_ref = product:{uuid}
```

The goal is to keep local deterministic competitor analysis linked to the user-selected product without pretending to fetch live competitor data.

## Scope

- Agent data skill: `data_competitor_analysis`
- Data worker autosuggest for competitor analysis
- SSE/API documentation
- Migration logs and matrix

## Out Of Scope

- No database migration.
- No HTTP API route change.
- No frontend code change.
- No external search provider change.
- No official Taobao/Douyin competitor data integration.

## Data Contract

`data_competitor_analysis` accepts:

- `product_ref?: string`
- `our_product: string`
- `our_price?: number`
- `competitors?: array<object>`

`product_ref` is only an identity/reference field. Competitor facts still must come from explicit input, existing deterministic defaults, or future adapter/search outputs.

## Behavior

- Worker autosuggest passes `product_ref` only when `context.selected_summary` exists.
- Worker uses selected product name as `our_product` when a product is selected.
- Skill output echoes `商品引用` with `product_ref` or null.
- Existing default competitor disclosure remains unchanged through `data_readiness=calculated_from_defaults`.
- This slice does not call `search_competitor` automatically.

## API / Auth

No new API route. Existing authenticated Agent SSE flow is unchanged:

```text
POST /api/sse/agent/session/stream
```

The field appears inside existing `skill_requested.input` and `skill_result.output`.

## Validation

- `cargo fmt --all`
- `cargo test -p agent data_competitor_analysis -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_competitor_analysis_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
