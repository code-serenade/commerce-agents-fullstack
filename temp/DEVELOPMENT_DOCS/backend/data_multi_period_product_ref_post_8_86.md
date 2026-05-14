# Post 8-86 Backend Design：data_multi_period_trend product_ref

## 业务理解

旧产品的数据分析能力里，多周期趋势用于判断 GMV、订单、UV、转化率等指标在多个周期上的环比变化、移动平均和趋势动能。新产品需要把这类能力继续落在 Rust agent skill 中，并和前端“用户手动选择商品”的商品上下文契约对齐。

本阶段重点不是接入真实平台历史数据，而是先保证：

- 用户选中商品时，skill 输入和输出能稳定保留 `product_ref=product:{uuid}`。
- 用户未选商品时，不注入默认商品。
- 自然语言中的多周期趋势意图能触达 `data_multi_period_trend`。
- 明确数字序列只来自本轮用户输入。

## 数据表设计

本阶段不新增数据库表，不新增 migration。

原因：

- `data_multi_period_trend` 是 agent native skill。
- 当前输入来自 SSE agent session 的 task/context 和用户本轮文本。
- 成功输出仍沿用既有 workspace artifact 沉淀链路。

## 字段设计

`data_multi_period_trend` 新增可选输入：

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
  "metric": "gmv",
  "periods": [7, 14, 30],
  "ma_window": 7,
  "values": [100, 108, 115, 126]
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

新增或调整的 worker autosuggest：

- `多周期趋势`
- `多周期环比`
- `多周期对比`
- `环比趋势`
- `同比环比`
- `移动平均`
- `趋势动能`
- `multi period`
- `multi-period trend`

触发后生成：

```json
{
  "skill_id": "data_multi_period_trend",
  "input": {
    "product_ref": "product:{uuid}",
    "metric": "gmv",
    "periods": [7, 14, 30],
    "values": [100, 108, 115, 126]
  }
}
```

`product_ref` 只有在本轮显式选品时出现。

## 错误和降级

- `values` 少于 4 个数据点时继续返回 deferred。
- 不创建假多周期趋势报告。
- 不伪造平台历史销售、行业周期或活动影响数据。
- 不执行任何外部平台写操作。

## API 文档

同步更新：

- `API_DOCS/sse.md`

## 验证

本阶段需要覆盖：

- skill 成功执行并回显 `商品引用`。
- 自然语言触发 `data_multi_period_trend`。
- 显式选品时 input 带 `product_ref`。
- 未选品时 input 不带空 `product_ref`。
- cargo fmt / test / check / clippy 通过。

