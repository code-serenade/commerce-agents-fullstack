# Post 8-92: data comprehensive diagnosis product_ref contract

## Requirement Summary

Continue old-product capability migration under the new product architecture, backend first. This slice focuses on `data_comprehensive_diagnosis`.

The business requirement is:

- make comprehensive business diagnosis reachable from natural language such as 综合诊断 / 业务诊断 / 经营体检
- preserve explicit product selection by passing `product_ref=product:{uuid}` only when the user selected a product in this run
- parse current-turn manual business metrics into the orchestrator input
- do not bind default products, historical products, or demo products
- do not pretend to read live Taobao, Douyin, Qianchuan, CRM, ERP, WMS, or warehouse data

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
data_comprehensive_diagnosis
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
- Missing `product_ref` means a general business diagnosis.
- Manual metrics are current-turn inputs only.
- The orchestrator keeps existing missing-data/deferred behavior when inputs are insufficient.

Output addition:

```json
{
  "商品引用": "product:{uuid}"
}
```

When no product is selected, the output value is null.

## Worker Autosuggest

The data worker now routes comprehensive diagnosis intent to `data_comprehensive_diagnosis` when text contains terms such as:

```text
综合诊断, 全面诊断, 业务诊断, 经营诊断, 经营体检, 业务体检, 一体化诊断, comprehensive diagnosis
```

The worker input reuses existing inference helpers:

- `infer_dashboard_input`
- `infer_refund_items`
- `infer_channel_roi_rows`
- `numbers_in_text`
- `selected_product_ref`

It parses:

- dashboard/store metrics
- target GMV
- funnel numeric values when enough sequential values are present
- time series values for anomaly/trend/forecast subparts
- channel rows
- refund item rows
- focus
- selected product reference

## Authorization

No new authorization rule is introduced in this slice. It runs inside the existing authenticated Agent SSE session and workspace context.

## Frontend Impact

No frontend file is changed in this slice.

Frontend can consume this through existing `skill_requested`, `skill_result`, and generated content artifact events.

User-visible behavior after frontend renders existing events:

- selected-product comprehensive diagnosis can show the selected product reference
- unspecified diagnosis stays general and does not show a hidden default product
- manual metrics entered by the user in the question can drive the comprehensive diagnosis
- missing subparts remain clearly deferred / missing-data instead of fake platform results

## Non-Goals

This slice does not:

- read live platform data
- create external platform adapters
- add frontend buttons
- create database tables
- change authentication
- use historical/demo/default product context

## Validation Plan

Run from `backend/`:

```text
cargo fmt --all
cargo test -p agent data_comprehensive_diagnosis -- --nocapture
cargo test -p agent autosuggests_comprehensive_diagnosis_with_explicit_metrics -- --nocapture
cargo test -p agent selected_product_context_feeds_comprehensive_diagnosis_ref -- --nocapture
cargo test -p agent comprehensive_diagnosis_without_selection_has_no_empty_product_ref -- --nocapture
cargo fmt --all --check
cargo check -p agent
cargo clippy -p agent
```

