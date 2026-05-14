# Post 8-91: data dashboard product_ref contract

## Requirement Summary

Continue old-product capability migration under the new product architecture, backend first. This slice focuses on `data_dashboard`.

The business requirement is:

- make dashboard / store overview questions consume the same explicit product-selection contract used by other data skills
- parse current-turn manual metrics such as GMV, orders, UV, clicks, ad spend, refunds, gross profit, and target GMV into the dashboard skill input
- do not bind default products, historical products, or demo products when the user did not explicitly select a product
- do not pretend to read live Taobao, Douyin, Qianchuan, CRM, ERP, or store-platform data

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
data_dashboard
```

Input addition:

```json
{
  "product_ref": "product:{uuid}"
}
```

Rules:

- `product_ref` is optional.
- `product_ref` means only the current run's explicitly selected product.
- Missing `product_ref` means a general store / business dashboard.
- Manual metrics are current-turn inputs only.
- Missing metrics keep the existing `store_metrics_required` / sample-only behavior.

Output addition:

```json
{
  "商品引用": "product:{uuid}"
}
```

When no product is selected, the output value is null.

## Worker Autosuggest

The data worker dashboard intent now uses structured dashboard input:

```text
dashboard, 看板, 数据概览, 经营概览, 经营指标, 核心指标, 店铺大盘, 大盘
```

It parses manual metrics with the same store metrics parser used by query-store-metrics:

- `gmv`
- `previous_gmv`
- `orders`
- `previous_orders`
- `visitors`
- `previous_visitors`
- `clicks`
- `ad_spend`
- `refund_amount`
- `gross_profit`
- `target_gmv`
- `platform`
- `days`

It adds `product_ref` only from explicit `context.selected_summary`.

## Authorization

No new authorization rule is introduced in this slice. It runs inside the existing authenticated Agent SSE session and workspace context.

## Frontend Impact

No frontend file is changed in this slice.

Frontend can consume this through existing `skill_requested`, `skill_result`, and generated content artifact events.

User-visible behavior after frontend renders existing events:

- selected-product dashboard reports can show the selected product reference
- unspecified dashboard reports stay general and do not show a hidden default product
- manual metrics entered by the user in the question can drive a real dashboard calculation
- missing metrics remain a clear sample / missing-data state

## Non-Goals

This slice does not:

- read live platform data
- create external store metrics adapters
- add frontend buttons
- create database tables
- change authentication
- use historical/demo/default product context

## Validation Plan

Run from `backend/`:

```text
cargo fmt --all
cargo test -p agent data_dashboard -- --nocapture
cargo test -p agent autosuggest_query_store_metrics_does_not_override_dashboard_intent -- --nocapture
cargo test -p agent selected_product_context_feeds_dashboard_ref -- --nocapture
cargo test -p agent dashboard_without_selection_has_no_empty_product_ref -- --nocapture
cargo fmt --all --check
cargo check -p agent
cargo clippy -p agent
```

