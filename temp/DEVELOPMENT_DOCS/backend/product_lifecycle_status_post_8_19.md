# Post 8-19：产品生命周期状态 API 后端设计

## 目标

把旧产品 `draft -> listing -> active -> archived` 的商品生命周期，迁成新产品后端的真实 API 能力。前端后续只能消费后端返回的状态、可流转动作和 Agent 上下文，不再自行硬编码商品阶段。

## 范围

- 复用现有 `products.status` 字段，不新增数据库表。
- 扩展商品状态为 `draft`、`listing`、`active`、`archived`。
- 新增生命周期选项、单商品生命周期查询、生命周期流转接口。
- 更新 `API_DOCS/app.md` 和前端 API adapter 类型。

## 状态机

```text
draft -> listing
listing -> active
listing -> draft
active -> archived
archived -> terminal
```

说明：

- `archived` 是终态，不能再流转。
- 现有创建商品默认仍保持 `active`，避免当前前端商品列表和历史数据行为突变。
- 原 `DELETE /api/app/products/{productId}` 继续作为软归档入口，生命周期 API 提供结构化流转入口。

## API

### `GET /api/app/products/lifecycle/options`

返回生命周期状态目录、允许的状态流转和 Agent 上下文。

### `GET /api/app/products/{productId}/lifecycle`

校验当前用户拥有商品后，返回商品、当前状态、下一步可选状态、当前可用流转和 Agent 上下文。已归档商品可查询，但没有下一步动作。

### `PATCH /api/app/products/{productId}/lifecycle`

请求体：

```json
{
  "targetStatus": "listing",
  "reason": "完成选品校验",
  "metadata": {}
}
```

行为：

- 校验当前用户、商品归属和目标状态。
- 校验当前状态到目标状态是否符合状态机。
- 更新 `products.status`。
- 在 `products.metadata.lifecycle` 写入 `currentStatus`、`lastTransition` 和最近流转 `history`。
- 返回更新后的生命周期摘要。

## 错误边界

- 商品不存在：`PRODUCT_NOT_FOUND`
- 商品不属于当前用户：`PRODUCT_FORBIDDEN`
- 目标状态非法或状态流转非法：`PRODUCT_REQUEST_INVALID`

## 验收

- Service 层状态机单元测试覆盖合法/非法流转和 metadata 记录。
- `cargo check -p service`、`cargo check -p web-server` 通过。
- 前端 `commerce.ts` 拥有可消费的 lifecycle API adapter。
- `API_DOCS/app.md` 与实现同步。
