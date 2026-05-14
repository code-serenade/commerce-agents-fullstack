# Post 8-101 Ops Pricing Strategy Product Ref

## 需求摘要

继续把旧产品运营岗位能力迁入新产品 Rust Agent 框架。本切片处理 `ops_pricing_strategy`：

- 支持新产品 `product_ref=product:{uuid}` 契约。
- 用户显式选择商品后，定价 / 价格 / 售价 / 成本等问法需要保留本轮商品引用。
- 用户没有选择商品时，不允许注入历史商品、默认商品或空 `product_ref`。
- 成本优先读取本轮文本中的显式标签，例如 `成本42`、`单位成本42`、`cost 42`。
- 竞品价格优先读取本轮显式 `竞品价` / `竞品价格` / `对手价格` 等片段。
- 本切片不接入淘宝、抖音、千川、ERP、WMS、供应链或实时竞品价格数据；定价策略仍基于用户输入和确定性价格模型。

## 后端设计

### 数据表

本切片不新增数据库表，不修改 migration。

原因：

- `ops_pricing_strategy` 仍是 `agent` crate native skill。
- 成功结果已通过既有 workspace artifact 物化链路沉淀为 `ops_pricing_strategy`。
- skill run 历史继续由 `agent_skill_runs` 统一记录。

### Skill 输入

扩展 `ops_pricing_strategy` schema：

```json
{
  "cost": 42,
  "competitor_prices": [89, 99, 109],
  "platform": "抖音",
  "product_ref": "product:11111111-1111-1111-1111-111111111111"
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

未提供 `product_ref` 时输出 `null`。建议零售价、平台佣金、利润率和竞品分析仍来自用户输入与确定性定价模型；缺少竞品价格时继续以 `data_readiness=calculated_from_defaults` 明示默认估算，不伪造平台实时竞品价。

### Worker 自动补偿

`ops` worker 在模型没有显式 `skill_requests` 时，可根据以下意图自动生成 `ops_pricing_strategy` 请求：

- price
- pricing
- cost
- 定价
- 价格
- 售价
- 成本

参数解析规则：

- 优先读取 `单位成本42`、`单品成本42`、`成本42`、`cost 42`。
- 优先从 `竞品价89`、`竞品价格99`、`对手价格109` 等片段提取竞品价格。
- 如果没有明确竞品价片段，则沿用旧逻辑，把本轮数字中非成本值作为竞品价格候选。
- 如果本轮显式选择商品，则带入 `product_ref` 和选中商品平台；未选商品时不写空引用。

## 鉴权和用户隔离

本切片不新增 HTTP API，不直接访问用户数据表。用户隔离继续依赖现有 SSE / Agent session 链路：

- 只有本轮显式 `context.selected_summary` 才能形成 `product_ref`。
- 未选商品时 worker 不写空 `product_ref`。
- 不从历史对话、预设商品或旧测试数据中推断商品。

## API / SSE

不新增接口。同步更新 `API_DOCS/sse.md`：

- 定价策略支持 `product_ref=product:{uuid}`。
- 定价 / 价格 / 售价 / 成本自然语言意图可触发 `ops_pricing_strategy`。
- 成本和竞品价格来自本轮显式输入；不会读取或伪造平台实时价格、竞品价、ERP、WMS 或供应链数据。

## 验证计划

```text
cargo fmt --all
cargo test -p agent ops_pricing_strategy -- --nocapture
cargo test -p agent selected_product_context_feeds_pricing_strategy_ref -- --nocapture
cargo test -p agent pricing_strategy_without_selection_has_no_empty_product_ref -- --nocapture
cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture
cargo fmt --all --check
cargo check -p agent
cargo clippy -p agent
git diff --check
```
