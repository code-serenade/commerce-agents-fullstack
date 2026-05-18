# Post 8-90: A/B 测试 product_ref 契约

## 状态

done

## 本轮目标

把旧产品里的 A/B 测试分析能力按新产品 Agent native skill 体系继续补齐：

- `data_ab_test_analyzer` 支持 `product_ref=product:{uuid}`
- data worker 能从 A/B、AB测试、对照组、实验组、推全等自然语言触发该 skill
- 用户显式选择商品时才绑定商品引用
- 未选商品时不注入默认商品、历史商品或演示商品

## 修改范围

- `backend/crates/agent/src/skills/data.rs`
- `backend/crates/agent/src/agent/worker/runner.rs`
- `API_DOCS/sse.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `temp/DEVELOPMENT_DOCS/backend/data_ab_test_product_ref_post_8_90.md`

## 产品内核变化

A/B 测试从“只能显式调用 skill”推进到“数据岗位自然语言可触达的真实 Agent skill”。它仍然运行在新产品 Agent runtime、SSE、skill result 和 artifact 沉淀链路里，不绕开同事给的框架。

## 后端变化

- `data_ab_test_analyzer` schema 新增可选 `product_ref`。
- 成功输出新增 `商品引用`。
- worker 新增 A/B 测试自然语言路由。
- worker 可解析对照组/实验组访客和转化数。
- worker 只从本轮 `context.selected_summary` 注入 `product_ref`。
- 未选商品时不写空 `product_ref`。

## 前端变化

本轮不改前端，不新增假入口。

前端后续可通过已有 `skill_requested`、`skill_result` 和内容 artifact 展示：

- A/B 测试计算过程
- 对照组与实验组转化率
- 显著性/置信度
- 是否建议推全
- 本轮显式选中的商品引用

## 用户可感知变化

用户手动勾选商品后，提问：

```text
分析这个商品详情页首屏A/B测试：对照组访客10000转化320，实验组访客9800转化390
```

后端会把本轮选中商品的 `product:<uuid>` 带进 A/B 测试报告。

用户没有手动选商品时，同样的问题仍可以做普通 A/B 测试，但不会暗中绑定历史商品或默认商品。

如果用户没有提供对照组/实验组样本和转化数据，后端不会伪造实验结果，而是按真实参数缺失处理。

## 边界

本轮不读取实验平台、淘宝、抖音、千川、CRM、DMP 或实时平台数据；真实实验数据接入仍属于后续 adapter/service 工作。

