# Post 8-75 Query Product Metrics Product Ref

## Requirement Summary

Continue the backend-first migration of old product capabilities into the new framework. The current slice extends `query_product_metrics` so product metric queries can carry the new explicit product reference contract:

```text
product_ref = product:{uuid}
```

This keeps selected-product and unspecified-product paths separate. The agent must not read a default product, historical product, or test product when the user has not explicitly selected or supplied product context.

## Scope

- Agent data skill: `query_product_metrics`
- Agent worker autosuggest for product metric query intent
- SSE/API documentation
- Migration logs and matrix

## Out Of Scope

- No database migration.
- No HTTP API route change.
- No frontend code change.
- No platform adapter or official Taobao/Douyin integration.
- No fake real-time product metrics.

## Data Contract

`query_product_metrics` accepts:

- `product_ref?: string`
- `product_id?: string`
- `product_name?: string`
- `sku_id?: string`
- existing metric fields such as `gmv`, `orders`, `visitors`, `ad_spend`, `inventory`

Priority:

1. `product_ref` is the new product identity for selected products.
2. `product_id` remains a legacy explicit filter.
3. `product_name` remains a readable explicit filter.

## Behavior

- When explicit metrics are present, the skill returns `status=success` and echoes `query.product_ref`.
- When explicit metrics are missing, the skill returns `status=deferred` and echoes `query.product_ref` if supplied.
- Worker autosuggest adds `product_ref` only from `context.selected_summary`.
- If no selected product exists, autosuggest does not add an empty `product_ref`.
- Existing `product_id` compatibility remains.

## API / Auth

No new API route is added. Existing authenticated Agent SSE flow is unchanged:

```text
POST /api/sse/agent/session/stream
```

The field appears inside existing `skill_requested.input` and `skill_result.output.query`.

## Validation

- `cargo fmt --all`
- `cargo test -p agent query_product_metrics -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_product_metrics_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
