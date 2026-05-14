# Post 8-90: data A/B test product_ref contract

## Requirement Summary

Continue the old-product capability migration into the new product architecture, backend first. This slice focuses on `data_ab_test_analyzer`.

The business requirement is:

- make A/B testing a real Agent native data skill path, not a frontend-only button
- preserve explicit product selection by passing `product_ref=product:{uuid}` only when the user selected a product in this run
- do not bind default products, historical products, demo coffee products, or platform data when the user did not explicitly select a product
- parse common A/B natural-language inputs such as control/test visitors and conversions

## Backend Design

No database table is added.

No HTTP route is added.

The existing Agent SSE route remains the integration surface:

```text
POST /api/sse/agent/session/stream
```

The skill result continues to flow through:

```text
skill_requested -> skill_result -> optional workspace artifact
```

## Agent Skill Contract

Skill:

```text
data_ab_test_analyzer
```

Input additions:

```json
{
  "product_ref": "product:{uuid}"
}
```

Rules:

- `product_ref` is optional.
- `product_ref` means only the current run's explicitly selected product.
- Missing `product_ref` means unspecified product context.
- The native skill still requires `control_visitors`, `control_conversions`, `test_visitors`, and `test_conversions`.
- Missing required A/B metrics returns existing validation behavior; the backend does not invent sample data.

Output additions:

```json
{
  "商品引用": "product:{uuid}"
}
```

When no product is selected, the output value is null or absent after compacting, depending on caller context.

## Worker Autosuggest

The data worker now routes A/B intent to `data_ab_test_analyzer` when text contains terms such as:

```text
A/B, AB测试, 对照组, 控制组, 实验组, 测试组, 转化对比, 推全
```

The worker parses:

- control visitors
- control conversions
- test visitors
- test conversions
- metric name
- test name
- selected product reference

If group labels cannot be parsed but four numeric values are present, the worker uses the current-text order:

```text
control_visitors, control_conversions, test_visitors, test_conversions
```

## Authorization

No new authorization rule is introduced in this slice. It runs inside the existing authenticated Agent SSE session and workspace context.

## Frontend Impact

No frontend file is changed in this slice.

Frontend can consume this through existing `skill_requested`, `skill_result`, and generated content artifact events.

User-visible behavior after frontend renders existing events:

- selected-product A/B reports can show the exact product reference
- unspecified A/B reports remain general and do not show a hidden default product
- missing sample inputs should be shown as a real backend validation/deferred state, not as a fake analysis result

## Non-Goals

This slice does not:

- read Taobao, Douyin, Qianchuan, CRM, DMP, or experimentation platform data
- create external adapter integrations
- execute platform rollout or push an experiment live
- add frontend buttons
- use historical/demo/default product context

## Validation Plan

Run from `backend/`:

```text
cargo fmt --all
cargo test -p agent data_ab_test_analyzer -- --nocapture
cargo test -p agent autosuggests_ab_test_analyzer_with_explicit_experiment_terms -- --nocapture
cargo test -p agent selected_product_context_feeds_ab_test_analyzer_ref -- --nocapture
cargo test -p agent ab_test_analyzer_without_selection_has_no_empty_product_ref -- --nocapture
cargo fmt --all --check
cargo check -p agent
cargo clippy -p agent
```

