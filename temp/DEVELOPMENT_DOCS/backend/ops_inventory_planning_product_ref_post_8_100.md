# Post 8-100 Ops Inventory Planning Product Ref

## 需求摘要

继续把旧产品运营岗位能力迁入新产品 Rust Agent 框架。本切片处理 `ops_inventory_planning`：

- 支持新产品 `product_ref=product:{uuid}` 契约。
- 用户显式选择商品后，库存 / 补货 / 备货 / 补货点等问法需要保留本轮商品引用。
- 用户没有选择商品时，不允许注入历史商品、默认商品或空 `product_ref`。
- 日均销量和供货周期优先读取本轮文本中的显式标签，例如 `日均销量120`、`供货周期7天`、`avg_daily_demand`、`lead_time_days`。
- 本切片不接入淘宝、抖音、千川、ERP、WMS、供应链或实时库存数据；库存规划仍基于用户输入和确定性库存模型。

## 后端设计

### 数据表

本切片不新增数据库表，不修改 migration。

原因：

- `ops_inventory_planning` 仍是 `agent` crate native skill。
- 成功结果已通过既有 workspace artifact 物化链路沉淀为 `ops_inventory_plan`。
- skill run 历史继续由 `agent_skill_runs` 统一记录。

### Skill 输入

扩展 `ops_inventory_planning` schema：

```json
{
  "avg_daily_demand": 120,
  "lead_time_days": 7,
  "product_ref": "product:11111111-1111-1111-1111-111111111111",
  "demand_std": 24,
  "service_level": 0.95
}
```

`product_ref` 是可选字段，只能来自当前 Agent task metadata 中的显式选品上下文。未选择商品时 worker 不写该字段。

### Skill 输出

成功输出新增：

```json
{
  "商品引用": "product:11111111-1111-1111-1111-111111111111"
}
```

未提供 `product_ref` 时输出 `null`。安全库存、补货点、建议订货量和周转天数仍来自用户输入与确定性库存模型，不伪造平台库存、WMS、ERP 或供应链数据。

### Worker 自动补偿

`ops` worker 在模型没有显式 `skill_requests` 时，可根据以下意图自动生成 `ops_inventory_planning` 请求：

- inventory
- stock
- 库存
- 补货
- 备货

参数解析规则：

- 优先读取 `日均销量120`、`日均需求120`、`日销量120`、`avg_daily_demand 120`。
- 优先读取 `供货周期7`、`补货周期7`、`交货周期7`、`lead_time_days 7`。
- 如果没有标签，则沿用原有数字顺序：第一个数字作为日均销量，第二个数字作为供货周期。
- 如果本轮显式选择商品，则带入 `product_ref` 和选中商品平台；未选商品时不写空引用。

## 鉴权和用户隔离

本切片不新增 HTTP API，不直接访问用户数据表。用户隔离继续依赖现有 SSE / Agent session 链路：

- 只有本轮显式 `context.selected_summary` 才能形成 `product_ref`。
- 未选商品时 worker 不写空 `product_ref`。
- 不从历史对话、预设商品或旧测试数据中推断商品。

## API / SSE

不新增接口。同步更新 `API_DOCS/sse.md`：

- 库存规划支持 `product_ref=product:{uuid}`。
- 库存 / 补货 / 备货自然语言意图可触发 `ops_inventory_planning`。
- 日均销量和供货周期来自本轮显式输入；不会读取或伪造 ERP、WMS、平台库存或实时供应链数据。

## 验证计划

```text
cargo fmt --all
cargo test -p agent ops_inventory_planning -- --nocapture
cargo test -p agent selected_product_context_feeds_inventory_planning_ref -- --nocapture
cargo test -p agent inventory_planning_without_selection_has_no_empty_product_ref -- --nocapture
cargo test -p agent inventory_optimizer_without_selection_has_no_empty_product_ref -- --nocapture
cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture
cargo fmt --all --check
cargo check -p agent
cargo clippy -p agent
git diff --check
```
