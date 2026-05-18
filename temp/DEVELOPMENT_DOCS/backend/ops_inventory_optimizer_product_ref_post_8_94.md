# Post 8-94: ops inventory optimizer product_ref contract

## Requirement Summary

Continue old-product capability migration under the new product architecture, backend first. This slice focuses on `ops_inventory_optimizer`.

The business requirement is:

- make inventory optimization reachable from natural language such as 库存优化 / EOQ / 安全库存 / 再订货点 / 库存健康
- preserve explicit product selection by passing `product_ref=product:{uuid}` only when the user selected a product in this run
- parse current-turn manual SKU rows into the skill input
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
ops_inventory_optimizer
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
- Missing `product_ref` means a general inventory optimization calculation.
- SKU rows are current-turn manual inputs only.
- Missing SKU rows keeps the existing deferred behavior.

SKU input shape remains:

```json
{
  "skus": [
    {
      "name": "主推款A",
      "monthly_sales": 300,
      "cost": 40,
      "stock_qty": 500
    }
  ]
}
```

Output addition:

```json
{
  "商品引用": "product:{uuid}"
}
```

When no product is selected, the output value is null.

## Worker Autosuggest

The ops worker now routes inventory-optimization intent to `ops_inventory_optimizer` when text contains terms such as:

```text
库存优化, 库存健康, 安全库存, 再订货点, EOQ, ROP, 经济订货量, 周转天数
```

The worker parses:

- `skus[]`
- `ordering_cost`
- `holding_rate`
- `lead_time_days`
- `service_level`
- `product_ref`

Natural language SKU rows can be written as separate comma/semicolon/newline segments, for example:

```text
SKU冷萃A 月销量300 成本40 库存500；SKU拿铁B 月销量180 成本35 库存160
```

## Authorization

No new authorization rule is introduced in this slice. It runs inside the existing authenticated Agent SSE session and workspace context.

## Frontend Impact

No frontend file is changed in this slice.

Frontend can consume this through existing `skill_requested`, `skill_result`, and generated content artifact events.

User-visible behavior after frontend renders existing events:

- selected-product inventory optimization can show the selected product reference
- unspecified inventory optimization stays general and does not show a hidden default product
- manual SKU rows entered by the user in the question can drive EOQ, safety stock, ROP, and inventory-health calculation
- missing SKU rows remains explicit `deferred` / `missing_user_input`

## Non-Goals

This slice does not:

- read live platform inventory data
- create external WMS/ERP/platform adapters
- add frontend buttons
- create database tables
- change authentication
- use historical/demo/default product context
