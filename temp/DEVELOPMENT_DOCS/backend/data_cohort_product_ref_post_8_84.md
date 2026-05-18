# Post 8-84 Data Cohort Product Ref

## Requirement Summary

Continue backend-first migration of old product capabilities. This slice extends the native `data_cohort_analysis` skill so product-bound cohort, retention, repurchase-retention, and churn-node analysis can carry the selected-product reference:

```text
product_ref=product:{uuid}
```

The reference must only come from explicit selected product context in the current request. The skill must not bind historical products, preset products, sample products, or default test products.

## Database

No database schema change.

## Backend Scope

- Agent data skill: `data_cohort_analysis`
- Worker autosuggest: data role cohort / retention / repurchase-retention intent
- SSE/API documentation only; no new HTTP route

## Skill Input Contract

`data_cohort_analysis` accepts:

```json
{
  "product_ref": "product:{uuid}",
  "days": 90,
  "cohort_unit": "week",
  "new_customers": 180
}
```

`product_ref` is optional. When absent, the result is an unspecified-product cohort retention analysis.

## Business Flow

1. User asks a data role question containing `队列留存`, `同期群`, `留存分析`, `复购留存`, `新客留存`, `流失节点`, or equivalent English phrasing.
2. Worker autosuggest creates a `data_cohort_analysis` request before broader customer segmentation logic.
3. If `context.selected_summary` contains an explicitly selected product, worker adds `product_ref`.
4. Worker parses explicit `首期新客` / `新客数` / `新增客户` values from the current text into `new_customers`.
5. Skill runs with user-provided or deterministic default new-customer baseline.
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
  "队列留存矩阵": [],
  "关键流失节点": [],
  "data_readiness": "user_provided"
}
```

When no product is selected, `商品引用` is null in the skill output and worker input contains no empty `product_ref`.

## Boundary

- Do not read live Taobao, Douyin, Qianchuan, membership, order, CRM, or platform retention data in this slice.
- Do not infer product identity from chat history.
- Do not create platform write actions or customer operations.
- Missing `new_customers` still uses deterministic defaults and is marked through `data_readiness=calculated_from_defaults`.

## Validation

- `cargo fmt --all`
- `cargo test -p agent data_cohort_analysis -- --nocapture`
- `cargo test -p agent autosuggests_cohort_analysis_with_explicit_retention_terms -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_cohort_analysis_ref -- --nocapture`
- `cargo test -p agent cohort_analysis_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
