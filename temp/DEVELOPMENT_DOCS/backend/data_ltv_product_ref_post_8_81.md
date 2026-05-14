# Post 8-81 Data LTV Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_ltv_calculator` skill so product-bound LTV / customer value analysis can carry the selected-product reference:

```text
product_ref=product:{uuid}
```

The reference must only come from explicit selected product context in the current request. The skill must not bind historical products, preset products, sample products, or default test products.

## Database

No database schema change.

## Backend Scope

- Agent data skill: `data_ltv_calculator`
- Worker autosuggest: data role LTV / customer value intent
- SSE/API documentation only; no new HTTP route

## Skill Input Contract

`data_ltv_calculator` accepts:

```json
{
  "product_ref": "product:{uuid}",
  "days": 90,
  "churn_period_days": 90,
  "cac": 35,
  "avg_order_value": 128,
  "purchase_frequency": 1.8,
  "gross_margin": 42,
  "monthly_retention": 38
}
```

`product_ref` is optional. When absent, the result is an unspecified-product LTV analysis.

## Business Flow

1. User asks a data role question containing `LTV`, `客户价值`, `生命周期价值`, `获客成本`, or `回收周期`.
2. Worker autosuggest creates a `data_ltv_calculator` request.
3. If `context.selected_summary` contains an explicitly selected product, worker adds `product_ref`.
4. Worker parses explicit `CAC`, 客单价, 月购买频次, 毛利率, 月留存率 from the current text when present.
5. Skill computes LTV, LTV/CAC, monthly contribution margin, and payback period.
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
  "LTV": 312.19,
  "LTV/CAC": 8.92,
  "data_readiness": "user_provided"
}
```

When no product is selected, `商品引用` is null in the skill output and worker input contains no empty `product_ref`.

## Boundary

- Do not read live member, Taobao, Douyin, Qianchuan, or CRM data in this slice.
- Do not infer product identity from chat history.
- Do not create platform write actions.
- Missing LTV inputs still use deterministic defaults and disclose `data_readiness=calculated_from_defaults`.

## Validation

- `cargo fmt --all`
- `cargo test -p agent data_ltv_calculator -- --nocapture`
- `cargo test -p agent autosuggests_ltv_calculator_with_explicit_value_metrics -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_ltv_calculator_ref -- --nocapture`
- `cargo test -p agent ltv_calculator_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
