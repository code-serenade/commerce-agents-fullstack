# Post 8-74 营销活动草稿 product_ref 契约

## 业务理解

`coordination_create_campaign` 会把 Agent 结果沉淀为营销活动草稿。活动草稿经常需要关联本轮用户手动选择的商品。当前 skill 只有旧 `product_id` 字段，容易把新产品 UUID 商品引用和旧字段语义混在一起。此切片新增 `product_ref=product:{uuid}`，同时保留旧 `product_id` 兼容。

## 数据库设计

本切片不新增表、不修改 migration。

## 接口设计

本切片不新增 HTTP API。

Agent skill `coordination_create_campaign` 输入补充：

- `product_ref?: string`，新产品商品引用，如 `product:{uuid}`
- `product_id?: string`，旧兼容字段

输出补充：

- `campaign_creation.product_ref`
- `normalized_inputs.product_ref`

## 鉴权与用户隔离

商品归属仍由 web-server / service 在构建 Agent metadata 前校验。Agent crate 只消费已校验的 `context.refs/context.product_ids`，不直连数据库。

## 行为规则

1. 优先使用显式传入的 `product_ref`。
2. 未传 `product_ref` 时，从 `context.refs` 中第一个 `product:{uuid}` 推断。
3. 旧 `product_id` 继续保留；未传时可从 `context.product_ids` 或 `product_ref` 推断出 UUID 字符串，以兼容现有草稿持久化链路。
4. 本阶段不触碰平台写操作，不创建审批，不执行平台动作。

## 验收

- schema 支持 `product_ref`。
- 输出 `campaign_creation.product_ref`。
- metadata 只有 `context.refs` 时可推断 `product_ref`。
- 旧 `campaign_creation.product_id` 继续兼容。
