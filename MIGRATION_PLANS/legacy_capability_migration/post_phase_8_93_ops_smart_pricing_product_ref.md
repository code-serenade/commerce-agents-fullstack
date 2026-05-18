# Post 8-93: 智能定价 product_ref 契约

## 状态

done

## 本轮目标

把旧产品里的智能定价 / 动态调价能力按新产品 Agent native skill 体系继续补齐：

- `ops_smart_pricing` 支持 `product_ref=product:{uuid}`
- ops worker 能从智能定价、动态定价、调价建议、价格优化等自然语言触发该 skill
- 用户显式选择商品时才绑定商品引用
- 用户在本轮问题里提供成本、当前售价、月销量、竞品价、库存天数、价格弹性等指标时，后端解析成真实 skill input
- 未选商品时不注入默认商品、历史商品或演示商品

## 修改范围

- `backend/crates/agent/src/skills/ops.rs`
- `backend/crates/agent/src/agent/worker/runner.rs`
- `API_DOCS/sse.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `temp/DEVELOPMENT_DOCS/backend/ops_smart_pricing_product_ref_post_8_93.md`

## 产品内核变化

智能定价从“已有 native skill，但主要依赖显式 skill 调用”补强为“运营岗位自然语言可触发、可绑定本轮显式选中商品、可消费本轮显式价格指标的运营分析能力”。它仍然运行在新产品 Agent runtime、SSE、skill result 和 artifact 沉淀链路里，不绕开同事给的框架。

## 后端变化

- `ops_smart_pricing` schema 新增可选 `product_ref`。
- 成功输出新增 `商品引用`。
- worker 新增智能定价自然语言自动补偿。
- worker 可解析本轮显式成本、当前售价、月销量、竞品价、库存天数和价格弹性。
- worker 会把用户正数表达的价格弹性归一为负弹性，匹配需求曲线计算。
- worker 本轮显式选品后，智能定价 input 带入 `product_ref`。
- worker 未选商品时不写空 `product_ref`。

## 前端变化

本轮不改前端，不新增假入口。

前端后续可通过已有 `skill_requested`、`skill_result` 和内容 artifact 展示：

- 当前经营状态
- 价格弹性
- 定价方案对比
- 推荐定价
- 竞品价格区间
- 库存压力告警
- 本轮显式选中的商品引用

## 用户可感知变化

用户手动勾选商品后，提问：

```text
给这个商品做智能定价：成本38，当前售价99，月销量820，竞品价89，竞品价109，库存天数45
```

后端会把本轮选中商品的 `product:<uuid>` 带进智能定价，并用用户本轮提供的价格指标计算推荐定价。

用户没有手动选商品时，同样的问题仍可以生成普通智能定价方案，但不会暗中绑定历史商品或默认商品。

如果用户没有提供成本或当前售价，后端继续返回原有 validation error；如果缺少竞品价格，后端会明确 `calculated_from_defaults`，不会伪造淘宝、抖音、千川或平台实时竞品数据。

## 边界

本轮不读取淘宝、抖音、千川、CRM、ERP、WMS、DMP、会员系统或实时数仓数据；真实外部数据接入仍属于后续 adapter/service/search 工作。
