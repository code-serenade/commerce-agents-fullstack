# Post 8-83 Data Demand Forecast Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_demand_forecast` skill so product-bound demand, sales, stocking, and replenishment forecast analysis can carry the selected-product reference:

```text
product_ref=product:{uuid}
```

The reference must only come from explicit selected product context in the current request. The skill must not bind historical products, preset products, sample products, or default test products.

## Database

No database schema change.

## Backend Scope

- Agent data skill: `data_demand_forecast`
- Worker autosuggest: data role demand forecast / replenishment forecast intent
- SSE/API documentation only; no new HTTP route

## Skill Input Contract

`data_demand_forecast` accepts:

```json
{
  "product_ref": "product:{uuid}",
  "metric": "orders",
  "days": 60,
  "forecast_days": [7, 14, 30],
  "values": [120, 128, 134, 145, 151, 162]
}
```

`product_ref` is optional. When absent, the result is an unspecified-product demand forecast.

## Business Flow

1. User asks a data role question containing `需求预测`, `销量预测`, `订单预测`, `备货预测`, `补货预测`, or equivalent English phrasing.
2. Worker autosuggest creates a `data_demand_forecast` request before the generic trend forecast branch.
3. If `context.selected_summary` contains an explicitly selected product, worker adds `product_ref`.
4. Worker parses explicit historical numeric series from the current text into `values`.
5. Skill requires at least 3 data points; otherwise it returns `deferred` through the existing data normalization path.
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
  "预测结果": [],
  "data_readiness": "user_provided"
}
```

When no product is selected, `商品引用` is null in the skill output and worker input contains no empty `product_ref`.

## Boundary

- Do not read live Taobao, Douyin, Qianchuan, WMS, ERP, or inventory platform data in this slice.
- Do not infer product identity from chat history.
- Do not create platform write actions or replenishment orders.
- Missing or insufficient `values` still returns deferred and does not create a fake demand forecast artifact.

## Validation

- `cargo fmt --all`
- `cargo test -p agent data_demand_forecast -- --nocapture`
- `cargo test -p agent autosuggests_demand_forecast_with_explicit_series -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_demand_forecast_ref -- --nocapture`
- `cargo test -p agent demand_forecast_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
