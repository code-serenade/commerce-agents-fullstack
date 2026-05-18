# Post 8-79 Data Refund Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_refund_decomposition` skill so product-bound refund decomposition can carry the selected-product reference:

```text
product_ref = product:{uuid}
```

The skill still only uses explicit refund items from the current turn. It must not pretend to fetch live Taobao, Douyin, Qianchuan, or other platform refund data.

## Scope

- Agent data skill: `data_refund_decomposition`
- Data worker autosuggest for refund / return / Pareto analysis intent
- SSE/API documentation
- Migration logs and matrix

## Out Of Scope

- No database migration.
- No HTTP API route change.
- No frontend code change.
- No official platform adapter integration.
- No fake live refund metrics.

## Data Contract

`data_refund_decomposition` accepts:

- `product_ref?: string`
- `days?: integer`
- `top_n?: integer`
- `items?: array<object>`

Each item remains:

```json
{"name":"质量问题","refund_amount":24000}
```

or:

```json
{"name":"色差","gmv":100000,"refund_rate":0.04}
```

`product_ref` is an identity field only. Refund decomposition still depends on explicit refund items or future service/adapter injected metrics.

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
- `cargo test -p agent data_refund_decomposition -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_refund_decomposition_ref -- --nocapture`
- `cargo test -p agent refund_decomposition_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
