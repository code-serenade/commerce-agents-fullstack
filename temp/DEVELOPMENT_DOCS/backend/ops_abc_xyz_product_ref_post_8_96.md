# Post 8-96 Ops ABC-XYZ Product Ref

## 需求摘要

继续把旧产品库存 / 运营分析能力迁入新产品 Rust Agent 框架。本切片处理 `ops_abc_xyz_classifier`：

- 支持新产品 `product_ref=product:{uuid}` 契约。
- 用户显式选择商品后，ABC-XYZ / ABC分类 / XYZ分类 / 九宫格库存分类等问法需要保留本轮商品引用。
- 用户没有选择商品时，不允许注入历史商品、默认商品或空 `product_ref`。
- 用户本轮提供 SKU、GMV、历史销量时，worker 应解析成真实 skill input。
- 没有 SKU 输入时仍沿用原有确定性演示 SKU，并通过 `data_readiness=calculated_from_defaults` 明示，不伪造平台库存或销量。

## 后端设计

### 数据表

本切片不新增数据库表，不修改 migration。

原因：

- `ops_abc_xyz_classifier` 仍是 `agent` crate native skill。
- 成功结果已通过既有 workspace artifact 物化链路沉淀为 `ops_abc_xyz_report`。
- skill run 历史继续由 `agent_skill_runs` 统一记录。

### Skill 输入

扩展 `ops_abc_xyz_classifier` schema：

```json
{
  "product_ref": "product:11111111-1111-1111-1111-111111111111",
  "platform": "抖音",
  "top_n": 5,
  "skus": [
    {
      "sku_id": "SKU冷萃A",
      "gmv": 120000,
      "sales_history": [100, 102, 98, 101]
    }
  ]
}
```

`product_ref` 是可选字段，只能来自当前 Agent task metadata 中的显式选品上下文。

### Skill 输出

成功输出新增：

```json
{
  "商品引用": "product:11111111-1111-1111-1111-111111111111"
}
```

未提供 `product_ref` 时输出 `null`。缺少 `skus` 时继续使用原有确定性演示 SKU，并通过 `data_readiness=calculated_from_defaults` 明示。

### Worker 自动补偿

`ops` worker 在模型没有显式 `skill_requests` 时，可根据以下意图自动生成 `ops_abc_xyz_classifier` 请求：

- ABC-XYZ
- ABC分类
- XYZ分类
- ABC库存
- XYZ库存
- 九宫格库存
- 9格矩阵
- 库存分类矩阵
- SKU分层
- 高价值SKU

可从本轮文本解析：

- `SKU冷萃A GMV12万 历史销量100 102 98 101`
- `SKU拿铁B GMV6万 历史销量60 88 44 96`
- `展示前2` / `top_n`

## 鉴权和用户隔离

本切片不新增 HTTP API，不直接访问用户数据表。用户隔离继续依赖现有 SSE / Agent session 链路：

- 只有本轮显式 `context.selected_summary` 才能形成 `product_ref`。
- 未选商品时 worker 不写空 `product_ref`。
- 不从历史对话、预设商品或旧测试数据中推断商品。

## API / SSE

不新增接口。同步更新 `API_DOCS/sse.md` 中 `skill_result` 和 ops artifact 说明：

- ABC-XYZ 分类支持 `product_ref=product:{uuid}`。
- 自然语言库存分类意图可触发 `ops_abc_xyz_classifier`。
- 使用演示 SKU 时仍明示 `calculated_from_defaults`，不伪造 WMS/ERP/平台实时库存。

## 验证计划

```text
cargo fmt --all
cargo test -p agent ops_abc_xyz_classifier -- --nocapture
cargo test -p agent autosuggests_abc_xyz_classifier_with_explicit_sku_rows -- --nocapture
cargo test -p agent selected_product_context_feeds_abc_xyz_classifier_ref -- --nocapture
cargo test -p agent abc_xyz_classifier_without_selection_has_no_empty_product_ref -- --nocapture
cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture
cargo fmt --all --check
cargo check -p agent
cargo clippy -p agent
git diff --check
```
