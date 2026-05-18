# Post 8-94: 库存优化 product_ref 契约

## 状态

done

## 本轮目标

把旧产品里的库存优化 / EOQ / 安全库存 / 再订货点能力按新产品 Agent native skill 体系继续补齐：

- `ops_inventory_optimizer` 支持 `product_ref=product:{uuid}`
- ops worker 能从库存优化、库存健康、安全库存、再订货点、EOQ、ROP 等自然语言触发该 skill
- 用户显式选择商品时才绑定商品引用
- 用户在本轮问题里提供 SKU 月销量、成本、库存数量、供货周期、服务水平等指标时，后端解析成真实 skill input
- 未选商品时不注入默认商品、历史商品或演示商品

## 修改范围

- `backend/crates/agent/src/skills/ops.rs`
- `backend/crates/agent/src/agent/worker/runner.rs`
- `API_DOCS/sse.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `temp/DEVELOPMENT_DOCS/backend/ops_inventory_optimizer_product_ref_post_8_94.md`

## 产品内核变化

库存优化从“已有 native skill，但主要依赖显式 skill 调用”补强为“运营岗位自然语言可触发、可绑定本轮显式选中商品、可消费本轮显式 SKU 明细的运营分析能力”。它仍然运行在新产品 Agent runtime、SSE、skill result 和 artifact 沉淀链路里，不绕开同事给的框架。

## 后端变化

- `ops_inventory_optimizer` schema 新增可选 `product_ref`。
- 成功输出新增 `商品引用`。
- worker 新增库存优化自然语言自动补偿。
- worker 可解析本轮显式 SKU 月销量、成本、库存数量、供货周期和服务水平。
- worker 本轮显式选品后，库存优化 input 带入 `product_ref`。
- worker 未选商品时不写空 `product_ref`。
- worker 保持普通“日均销量 + 供货周期”补货问题继续走 `ops_inventory_planning`。

## 前端变化

本轮不改前端，不新增假入口。

前端后续可通过已有 `skill_requested`、`skill_result` 和内容 artifact 展示：

- SKU 库存优化明细
- EOQ 经济订货量
- 安全库存
- 再订货点 ROP
- 当前库存天数
- 库存健康
- ABC 分类汇总
- 本轮显式选中的商品引用

## 用户可感知变化

用户手动勾选商品后，提问：

```text
给这个商品做库存健康和再订货点优化：SKU冷萃A 月销量300 成本40 库存500
```

后端会把本轮选中商品的 `product:<uuid>` 带进库存优化，并用用户本轮提供的 SKU 明细计算 EOQ、安全库存、再订货点和库存健康。

用户没有手动选商品时，同样的问题仍可以生成普通库存优化方案，但不会暗中绑定历史商品或默认商品。

如果用户没有提供 SKU 明细，后端继续明确 `missing_user_input` / `deferred`，不会伪造淘宝、抖音、千川、WMS、ERP 或平台实时库存数据。

## 边界

本轮不读取淘宝、抖音、千川、CRM、ERP、WMS、DMP、会员系统或实时数仓数据；真实外部数据接入仍属于后续 adapter/service/search 工作。
