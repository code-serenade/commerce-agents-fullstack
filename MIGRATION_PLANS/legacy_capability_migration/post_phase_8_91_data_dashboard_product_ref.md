# Post 8-91: 数据看板 product_ref 契约

## 状态

done

## 本轮目标

把旧产品里的经营看板 / 店铺大盘能力按新产品 Agent native skill 体系继续补齐：

- `data_dashboard` 支持 `product_ref=product:{uuid}`
- data worker 能从 dashboard、看板、经营概览、经营指标、店铺大盘等自然语言触发该 skill
- 用户显式选择商品时才绑定商品引用
- 用户在本轮问题里提供 GMV、订单、UV、广告费、退款、毛利、目标 GMV 等指标时，后端解析成真实 skill input
- 未选商品时不注入默认商品、历史商品或演示商品

## 修改范围

- `backend/crates/agent/src/skills/data.rs`
- `backend/crates/agent/src/agent/worker/runner.rs`
- `API_DOCS/sse.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `temp/DEVELOPMENT_DOCS/backend/data_dashboard_product_ref_post_8_91.md`

## 产品内核变化

经营看板从“能生成结构化看板”补强为“可绑定本轮显式选中商品、可消费本轮显式经营指标的 Agent skill”。它仍然运行在新产品 Agent runtime、SSE、skill result 和 artifact 沉淀链路里，不绕开同事给的框架。

## 后端变化

- `data_dashboard` schema 新增可选 `product_ref`。
- 成功输出新增 `商品引用`。
- worker dashboard 自动补偿改为结构化 input 推断。
- worker 可解析本轮显式 GMV、订单、UV、点击、广告费、退款、毛利、目标 GMV 等指标。
- worker 本轮显式选品后，dashboard input 带入 `product_ref`。
- worker 未选商品时不写空 `product_ref`。

## 前端变化

本轮不改前端，不新增假入口。

前端后续可通过已有 `skill_requested`、`skill_result` 和内容 artifact 展示：

- 经营健康度
- 核心指标卡
- 增长 / 效率 / 风险分区
- 预警和下一步动作
- 本轮显式选中的商品引用

## 用户可感知变化

用户手动勾选商品后，提问：

```text
打开这个商品经营看板：GMV18万，订单1800，UV3.6万，广告费3.6万，目标GMV20万
```

后端会把本轮选中商品的 `product:<uuid>` 带进经营看板，并用用户本轮提供的指标计算看板。

用户没有手动选商品时，同样的问题仍可以生成普通经营看板，但不会暗中绑定历史商品或默认商品。

如果用户没有提供经营指标，后端继续明确返回 `store_metrics_required` / 样例态，不会伪造淘宝、抖音、千川或其它平台实时数据。

## 边界

本轮不读取淘宝、抖音、千川、CRM、ERP、WMS 或实时店铺数据；真实经营数据接入仍属于后续 adapter/service 工作。

