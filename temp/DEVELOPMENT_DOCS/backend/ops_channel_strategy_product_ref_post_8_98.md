# Post 8-98 Ops Channel Strategy Product Ref

## 需求摘要

继续把旧产品运营岗位能力迁入新产品 Rust Agent 框架。本切片处理 `ops_channel_strategy`：

- 支持新产品 `product_ref=product:{uuid}` 契约。
- 用户显式选择商品后，渠道分配 / 渠道策略 / 投放渠道等问法需要保留本轮商品引用。
- 用户没有选择商品时，不允许注入历史商品、默认商品或空 `product_ref`。
- 渠道预算解析优先读取 `渠道预算` / `月预算` / `预算` / `budget` 标签，避免把 `618`、`双11` 等活动名数字误当预算。
- 本切片不接入淘宝、抖音、千川或其它平台实时渠道数据；渠道策略仍基于用户输入、商品类目和确定性渠道规则。

## 后端设计

### 数据表

本切片不新增数据库表，不修改 migration。

原因：

- `ops_channel_strategy` 仍是 `agent` crate native skill。
- 成功结果已通过既有 workspace artifact 物化链路沉淀为 `ops_channel_strategy`。
- skill run 历史继续由 `agent_skill_runs` 统一记录。

### Skill 输入

扩展 `ops_channel_strategy` schema：

```json
{
  "product_type": "咖啡饮品",
  "product_ref": "product:11111111-1111-1111-1111-111111111111",
  "budget": 20000,
  "target_audience": "一二线城市通勤白领"
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

未提供 `product_ref` 时输出 `null`。渠道组合、内容节奏和预算拆分仍来自商品类目、目标人群和用户输入预算，不伪造平台渠道表现。

### Worker 自动补偿

`ops` worker 在模型没有显式 `skill_requests` 时，可根据以下意图自动生成 `ops_channel_strategy` 请求：

- channel
- 渠道
- 投放渠道
- 流量渠道
- 渠道分配
- 渠道策略

预算解析规则：

- 优先读取 `渠道预算20000`、`月预算2万`、`预算: 50000`、`budget 30000`。
- 若没有预算标签，再从本轮文本数字中选择非活动名数字。
- `618`、`双11` 这类活动名数字不会被当成预算。

## 鉴权和用户隔离

本切片不新增 HTTP API，不直接访问用户数据表。用户隔离继续依赖现有 SSE / Agent session 链路：

- 只有本轮显式 `context.selected_summary` 才能形成 `product_ref`。
- 未选商品时 worker 不写空 `product_ref`。
- 不从历史对话、预设商品或旧测试数据中推断商品。

## API / SSE

不新增接口。同步更新 `API_DOCS/sse.md`：

- 渠道策略支持 `product_ref=product:{uuid}`。
- 渠道 / 投放渠道 / 渠道分配 / 渠道策略自然语言意图可触发 `ops_channel_strategy`。
- budget 解析不会把活动名里的 `618` / `双11` 误当预算。

## 验证计划

```text
cargo fmt --all
cargo test -p agent ops_channel_strategy -- --nocapture
cargo test -p agent selected_product_context_feeds_channel_strategy_ref -- --nocapture
cargo test -p agent channel_strategy_without_selection_has_no_empty_product_ref -- --nocapture
cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture
cargo fmt --all --check
cargo check -p agent
cargo clippy -p agent
git diff --check
```
