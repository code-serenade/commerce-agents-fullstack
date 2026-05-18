# Post 8-72 客服商品查询 product_ref 契约

## 业务理解

客服和售后岗位需要能查询“本轮用户手动选择的商品资料”，但不能默认拿历史商品、测试商品或 RAG 里的任意商品。Post 8-71 已把新产品商品引用统一为 `product_ref=product:{uuid}`，本切片把客服商品查询能力也接入同一契约。

## 数据库设计

本切片不新增表、不修改 migration。

## 接口设计

本切片不新增 HTTP API。

Agent skill `service_query_product` 兼容输入：

- `product_ref: string`，新产品商品引用，格式如 `product:{uuid}`
- `product_id: string | integer`，旧兼容字段
- `product_name: string`，名称关键词

SSE metadata 仍由 web-server 通过 `metadata.context.productIds/contextRefs` 构建，agent skill 只消费已校验后的 `context.product_ids`、`context.refs`、`context.selected_summary` 和 `context.sources_json`。

## 鉴权与用户隔离

商品归属和 active 状态校验仍在 service/web-server 构建上下文时完成。`service_query_product` 只读取 Agent runtime metadata，不直连数据库，不扩大商品可见范围。

## 行为规则

1. 有 `product_ref` 时优先按 `context.refs`、`context.product_ids`、`sources_json.product_id` 和 `selected_summary` 匹配。
2. 没有 `product_ref` 时继续兼容旧 `product_id` 和 `product_name`。
3. 没有显式商品上下文时返回 `deferred/context_required`。
4. 有显式上下文但筛选不命中时返回 `deferred/context_filter_not_matched`。
5. 客服岗位自然语言问“查商品信息/查询商品资料/商品上下文”等时，可自动触发 `service_query_product`。

## 验收

- 客服岗位能自然触发 `service_query_product`。
- 显式选品后 autosuggest input 带入 `product_ref`。
- `service_query_product` 能用 `product_ref` 精确匹配显式商品上下文。
- 旧 `product_id` 兼容不破坏。
- 未显式选品仍不默认注入商品。
