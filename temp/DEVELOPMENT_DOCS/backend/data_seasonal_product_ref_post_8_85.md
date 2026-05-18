# Post 8-85 Data Seasonal Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_seasonal_decompose` skill so product-bound seasonality, periodic trend, and intra-week fluctuation analysis can carry the selected-product reference:

```text
product_ref=product:{uuid}
```

The reference must only come from explicit selected product context in the current request. The skill must not bind historical products, preset products, sample products, or default test products.

## Database

No database schema change.

## Backend Scope

- Agent data skill: `data_seasonal_decompose`
- Worker autosuggest: data role seasonality / periodic fluctuation intent
- SSE/API documentation only; no new HTTP route

## Skill Input Contract

`data_seasonal_decompose` accepts:

```json
{
  "product_ref": "product:{uuid}",
  "metric": "orders",
  "days": 56,
  "period": 7,
  "values": [120, 132, 141, 150, 168, 181, 190, 128, 139, 149, 158, 176, 188, 201]
}
```

`product_ref` is optional. When absent, the result is an unspecified-product seasonal decomposition.

## Business Flow

1. User asks a data role question containing `季节性`, `周期性`, `周期分解`, `季节分解`, `周内波动`, `周期波动`, or equivalent English phrasing.
2. Worker autosuggest creates a `data_seasonal_decompose` request before the generic trend forecast branch.
3. If `context.selected_summary` contains an explicitly selected product, worker adds `product_ref`.
4. Worker parses explicit historical numeric series from the current text into `values`.
5. Skill requires at least `period * 2` data points; otherwise it returns `deferred` through the existing data normalization path.
6. Successful output echoes `商品引用` only when `product_ref` was provided.

## Auth

No new auth behavior. The existing agent session auth boundary applies.

## API / SSE

No new endpoint. Existing `skill_requested` and `skill_result` events are used.

`skill_result.output` may include:

```json
{
  "status": "success",
  "商品引用": "product:{uuid}",
  "指标": "订单数",
  "季节周期": "7天",
  "季节性分析": {},
  "data_readiness": "user_provided"
}
```

When no product is selected, `商品引用` is null in the skill output and worker input contains no empty `product_ref`.

## Boundary

- Do not read live Taobao, Douyin, Qianchuan, platform history, campaign calendar, or industry seasonality data in this slice.
- Do not infer product identity from chat history.
- Do not create platform write actions or campaign schedule changes.
- Missing or insufficient `values` still returns deferred and does not create a fake seasonal decomposition artifact.

## Validation

- `cargo fmt --all`
- `cargo test -p agent data_seasonal_decompose -- --nocapture`
- `cargo test -p agent autosuggests_seasonal_decompose_with_explicit_series -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_seasonal_decompose_ref -- --nocapture`
- `cargo test -p agent seasonal_decompose_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
