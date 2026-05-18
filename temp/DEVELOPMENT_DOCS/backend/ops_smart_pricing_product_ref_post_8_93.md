# Post 8-93: ops smart pricing product_ref contract

## Requirement Summary

Continue old-product capability migration under the new product architecture, backend first. This slice focuses on `ops_smart_pricing`.

The business requirement is:

- make smart pricing reachable from natural language such as 智能定价 / 动态定价 / 调价建议 / 价格优化
- preserve explicit product selection by passing `product_ref=product:{uuid}` only when the user selected a product in this run
- parse current-turn manual pricing inputs into the skill input
- do not bind default products, historical products, or demo products
- do not pretend to read live Taobao, Douyin, Qianchuan, WMS, ERP, CRM, DMP, or warehouse data

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
ops_smart_pricing
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
- Missing `product_ref` means a general pricing analysis.
- Manual cost, current price, monthly sales, competitor prices, elasticity, stock days, and platform are current-turn inputs only.
- Missing cost or current price keeps the existing validation behavior.
- Missing competitor prices continues to use deterministic fallback and reports `calculated_from_defaults`.

Output addition:

```json
{
  "商品引用": "product:{uuid}"
}
```

When no product is selected, the output value is null.

## Worker Autosuggest

The ops worker now routes smart-pricing intent to `ops_smart_pricing` when text contains terms such as:

```text
智能定价, 动态定价, 价格优化, 利润定价, 最优定价, 调价建议, 涨价建议, 降价建议, 清仓定价, 库存压力价, smart pricing, dynamic pricing
```

The worker parses:

- `cost`
- `current_price`
- `current_monthly_sales`
- `competitor_prices`
- `stock_days`
- `elasticity`
- `platform`
- `product_ref`

Positive natural-language elasticity values are normalized to negative elasticity for demand-curve calculation.

## Authorization

No new authorization rule is introduced in this slice. It runs inside the existing authenticated Agent SSE session and workspace context.

## Frontend Impact

No frontend file is changed in this slice.

Frontend can consume this through existing `skill_requested`, `skill_result`, and generated content artifact events.

User-visible behavior after frontend renders existing events:

- selected-product smart pricing can show the selected product reference
- unspecified smart pricing stays general and does not show a hidden default product
- manual pricing metrics entered by the user in the question can drive the pricing calculation
- missing external/platform data remains explicit instead of fake platform results

## Non-Goals

This slice does not:

- read live platform data
- create external platform adapters
- add frontend buttons
- create database tables
- change authentication
- use historical/demo/default product context
