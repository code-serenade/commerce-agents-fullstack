# Post 8-89 Backend Design：data_customer_segmentation product_ref

## 业务理解

客户分层是电商虚拟员工做用户运营、复购、召回和内容投放时的基础能力。用户选择商品后再问“这个商品的人群分层 / RFM”，新产品必须区分“指定商品客户分层”和“未指定商品客户分层”。本阶段把 `data_customer_segmentation` 接入 `product_ref=product:{uuid}` 契约，避免客户分层报告失去商品边界。

本阶段重点不是接入真实 CRM 或 DMP，而是先保证：

- 用户选中商品时，skill 输入和输出能稳定保留 `product_ref=product:{uuid}`。
- 用户未选商品时，不注入默认商品。
- 自然语言中的客户 / 人群 / RFM / 分层意图能继续触达 `data_customer_segmentation`。
- 明确 RFM 聚合指标只来自本轮用户输入。

## 数据表设计

本阶段不新增数据库表，不新增 migration。

原因：

- `data_customer_segmentation` 是 agent native skill。
- 当前输入来自 SSE agent session 的 task/context 和用户本轮文本。
- 成功输出仍沿用既有 workspace artifact 沉淀链路。

## 字段设计

`data_customer_segmentation` 新增可选输入：

```json
{
  "product_ref": "product:{uuid}"
}
```

字段说明：

- `product_ref`：仅代表用户本轮显式选中的商品引用。
- 不允许从历史对话、测试数据、预设商品或商品名猜测生成。
- 未选商品时不发送该字段。

既有输入保持：

```json
{
  "total_customers": 1200,
  "avg_order_value": 168,
  "avg_orders_per_customer": 2.3,
  "days": 90,
  "recency_days": 21
}
```

成功输出新增：

```json
{
  "商品引用": "product:{uuid}"
}
```

未传 `product_ref` 时输出为 `null`。

## 权限和用户隔离

本阶段不新增 HTTP API，不直接读取用户数据库资源。

商品引用来源仍遵守现有上下文规则：

- 只读取 `context.selected_summary`。
- 只有其中明确存在 `commerce.selected_product_context` 来源时才视为用户手动选择商品。
- 不从普通历史摘要中推断商品。

## SSE / Agent 行为

客户分层 autosuggest 触发后生成：

```json
{
  "skill_id": "data_customer_segmentation",
  "input": {
    "product_ref": "product:{uuid}",
    "total_customers": 1200,
    "avg_order_value": 168,
    "avg_orders_per_customer": 2.3,
    "recency_days": 21,
    "days": 90
  }
}
```

`product_ref` 只有在本轮显式选品时出现。

## 错误和降级

- 缺少 RFM 聚合输入时继续使用 deterministic 默认估算，并以 `data_readiness=calculated_from_defaults` 明示。
- 不伪造会员系统、订单系统、CRM、DMP 或平台实时人群数据。
- 不执行任何外部平台写操作。

## API 文档

同步更新：

- `API_DOCS/sse.md`

## 验证

本阶段需要覆盖：

- skill 成功执行并回显 `商品引用`。
- 自然语言触发 `data_customer_segmentation`。
- 显式选品时 input 带 `product_ref`。
- 未选品时 input 不带空 `product_ref`。
- cargo fmt / test / check / clippy 通过。

