# Post 8-82 Data Price Elasticity Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_price_elasticity` skill so product-bound pricing elasticity analysis can carry the selected-product reference:

```text
product_ref=product:{uuid}
```

The reference must only come from explicit selected product context in the current request. The skill must not bind historical products, preset products, sample products, or default test products.

## Database

No database schema change.

## Backend Scope

- Agent data skill: `data_price_elasticity`
- Worker autosuggest: data role price elasticity / price sensitivity intent
- SSE/API documentation only; no new HTTP route

## Skill Input Contract

`data_price_elasticity` accepts:

```json
{
  "product_ref": "product:{uuid}",
  "price_points": [
    {"price": 90, "quantity": 560},
    {"price": 110, "quantity": 440}
  ],
  "cost": 45,
  "current_price": 99,
  "days": 60
}
```

`product_ref` is optional. When absent, the result is an unspecified-product price elasticity analysis.

## Business Flow

1. User asks a data role question containing `价格弹性`, `价格敏感`, `调价测试`, `涨价影响`, `降价影响`, or equivalent English phrasing.
2. Worker autosuggest creates a `data_price_elasticity` request.
3. If `context.selected_summary` contains an explicitly selected product, worker adds `product_ref`.
4. Worker parses explicit price/quantity pairs such as `价格90销量560，价格110销量440`.
5. Worker parses explicit cost and current price when present.
6. Skill computes arc elasticity, sensitivity tier, current price, and optimal price reference.
7. Successful output echoes `商品引用` only when `product_ref` was provided.

## Auth

No new auth behavior. The existing agent session auth boundary applies.

## API / SSE

No new endpoint. Existing `skill_requested` and `skill_result` events are used.

`skill_result.output` may include:

```json
{
  "status": "success",
  "商品引用": "product:{uuid}",
  "价格弹性系数": -1.2,
  "敏感度分类": "中弹性",
  "data_readiness": "user_provided"
}
```

When no product is selected, `商品引用` is null in the skill output and worker input contains no empty `product_ref`.

## Boundary

- Do not read live Taobao, Douyin, Qianchuan, CRM, or pricing platform data in this slice.
- Do not infer product identity from chat history.
- Do not create platform write actions or pricing changes.
- Missing `price_points` still uses deterministic defaults and discloses `data_readiness=calculated_from_defaults`.

## Validation

- `cargo fmt --all`
- `cargo test -p agent data_price_elasticity -- --nocapture`
- `cargo test -p agent autosuggests_price_elasticity_with_explicit_price_points -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_price_elasticity_ref -- --nocapture`
- `cargo test -p agent price_elasticity_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
