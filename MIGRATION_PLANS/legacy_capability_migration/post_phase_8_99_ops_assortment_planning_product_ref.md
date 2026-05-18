# Post 8-99 Ops Assortment Planning Product Ref

## 状态

done

## 本次目标

- 把选品规划 `ops_assortment_planning` 接入新产品 `product_ref=product:{uuid}` 契约。
- 让用户手动选择商品后，货盘 / SKU 组合 / 选品组合 / 组合规划等问法能保留本轮商品引用。
- 让选品预算解析优先读取 `选品预算` / `货盘预算` / `采购预算` / `预算` / `budget` 标签，避免把 `618`、`双11` 等活动名数字误当预算。
- 继续保持未选商品时不绑定历史商品或预设商品，也不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链或平台热卖数据。

## 本次修改

- 后端 agent：`ops_assortment_planning` schema 新增 `product_ref`。
- 后端 agent：成功输出新增 `商品引用`。
- 后端 worker：货盘 / SKU 组合 / 选品组合 / 组合规划自然语言自动补偿改为结构化 input。
- 后端 worker：本轮显式选品后，选品规划 input 带入 `product_ref`。
- 后端 worker：未选商品时不写空 `product_ref`。
- 后端 worker：预算解析优先使用带标签的预算值，避免活动名数字污染预算。
- API/SSE：同步 `API_DOCS/sse.md`。
- 迁移文档：同步阶段验收、总计划和 skill 矩阵。

## 用户可感知变化

- 用户手动勾选商品后，再问“给这个商品做货盘 / SKU 组合 / 选品组合规划”时，后端会把本轮选中商品的 `product:<uuid>` 带进选品规划结果。
- 用户没有手动选商品时，选品规划仍按普通货盘问题运行，但不会暗中绑定历史商品或默认商品。
- 用户写“618 货盘规划，选品预算8万”时，后端会把预算识别为 `80000`，不会把 `618` 当成选品预算。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent ops_assortment_planning -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_assortment_planning_ref -- --nocapture`
- [x] `cargo test -p agent assortment_planning_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture`

## 后续缺口

- 真实淘宝、抖音、千川、ERP、WMS、供应链、平台热卖、供应商报价和实时库存数据仍需要后续 service/adapter/search 注入，不能由本地 native skill 伪造或默认读取。
- 前端后续应把选品规划 `skill_result.output.商品引用`、`商品组合`、`测品规则` 和预算解析结果展示在真实过程面板与交付物详情中。
