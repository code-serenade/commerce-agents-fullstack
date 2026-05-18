# Post 8-92: 综合业务诊断 product_ref 契约

## 状态

done

## 本轮目标

把旧产品里的综合诊断 / 业务体检能力按新产品 Agent native orchestrator 体系继续补齐：

- `data_comprehensive_diagnosis` 支持 `product_ref=product:{uuid}`
- data worker 能从综合诊断、全面诊断、业务诊断、经营体检等自然语言触发该 skill
- 用户显式选择商品时才绑定商品引用
- 用户在本轮问题里提供 GMV、订单、UV、广告费、退款、毛利、目标 GMV、渠道、退款项等指标时，后端解析成真实 orchestrator input
- 未选商品时不注入默认商品、历史商品或演示商品

## 修改范围

- `backend/crates/agent/src/skills/data.rs`
- `backend/crates/agent/src/agent/worker/runner.rs`
- `API_DOCS/sse.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `temp/DEVELOPMENT_DOCS/backend/data_comprehensive_product_ref_post_8_92.md`

## 产品内核变化

综合诊断从“已有 native orchestrator，但只能显式调用”补强为“数据岗位自然语言可触发、可绑定本轮显式选中商品、可消费本轮显式经营指标的核心诊断入口”。它仍然运行在新产品 Agent runtime、SSE、skill result 和 artifact 沉淀链路里，不绕开同事给的框架。

## 后端变化

- `data_comprehensive_diagnosis` schema 新增可选 `product_ref`。
- 成功输出新增 `商品引用`。
- worker 新增综合诊断自然语言自动补偿。
- worker 可解析本轮显式经营指标、渠道指标、退款项和数字序列。
- worker 本轮显式选品后，综合诊断 input 带入 `product_ref`。
- worker 未选商品时不写空 `product_ref`。

## 前端变化

本轮不改前端，不新增假入口。

前端后续可通过已有 `skill_requested`、`skill_result` 和内容 artifact 展示：

- 综合健康评分
- 分项诊断
- 核心发现
- 风险预警
- 增长机会
- 7 天行动计划
- 缺失数据
- 本轮显式选中的商品引用

## 用户可感知变化

用户手动勾选商品后，提问：

```text
对这个商品做综合业务诊断：GMV26万，上期GMV23万，订单2100，UV6.2万，广告费7.2万，退款1.56万
```

后端会把本轮选中商品的 `product:<uuid>` 带进综合诊断，并用用户本轮提供的指标编排看板、漏斗、异常、客户、退款、预测、归因和 LTV 分项。

用户没有手动选商品时，同样的问题仍可以生成普通综合业务诊断，但不会暗中绑定历史商品或默认商品。

如果用户没有提供足够的业务指标，后端继续明确返回 `missing_user_input` / `deferred`，不会伪造淘宝、抖音、千川、CRM、ERP、WMS 或实时经营数据。

## 边界

本轮不读取淘宝、抖音、千川、CRM、ERP、WMS、DMP、会员系统或实时数仓数据；真实外部数据接入仍属于后续 adapter/service/search 工作。

