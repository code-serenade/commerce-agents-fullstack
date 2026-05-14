# Post 8-73 客服支持类 Skill 商品引用契约补齐

## 业务理解

客服工单、FAQ、升级处理和退换货处理都可能涉及某个商品。新产品的正确链路是用户手动勾选商品后，后端把 `product_ref=product:{uuid}` 作为商品引用传入 skill；未勾选商品时，客服 skill 仍按普通客服问题处理，不默认使用历史商品或预设商品。

## 数据库设计

本切片不新增表、不修改 migration。

## 接口设计

本切片不新增 HTTP API。

Agent skill 输入补充：

- `service_ticket_handler.product_ref?: string`
- `service_faq_playbook.product_ref?: string`
- `service_escalation_flow.product_ref?: string`
- `service_return_handler.product_ref?: string`

旧 `product_id` 字段继续保留，仅作为旧整数参数兼容。

## 鉴权与用户隔离

商品归属和 active 状态校验仍由 web-server / service 在构建 `context.selected_summary` 前完成。Agent skill 只接收 worker 从已校验 metadata 中解析出的 `product_ref`，不直连数据库，不扩大用户可见商品范围。

## 行为规则

1. 只有本轮存在 `context.selected_summary` 时，worker 才给客服支持类 skill 自动补偿 `product_ref`。
2. 未选商品时，autosuggest input 不留下空 `product_ref`。
3. skill 输出保留 `product_ref` 作为关联字段，便于前端过程、artifact 和后续客服记录追踪。
4. 旧 `product_id` 输入继续兼容，但新产品 UUID 不写入旧整数字段。

## 验收

- 客服工单、FAQ、升级、退换货 schema 均支持 `product_ref`。
- 对应 skill 输出回显商品引用。
- 显式选品后 worker 自动补偿 `product_ref`。
- 未选品时不注入空商品引用。
- 原有客服 skill 单测继续通过。
