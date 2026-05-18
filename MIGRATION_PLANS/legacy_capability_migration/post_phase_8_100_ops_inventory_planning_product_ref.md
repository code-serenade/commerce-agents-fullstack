# Post 8-100 Ops Inventory Planning Product Ref

## 状态

done

## 本次目标

- 把库存规划 `ops_inventory_planning` 接入新产品 `product_ref=product:{uuid}` 契约。
- 让用户手动选择商品后，库存 / 补货 / 备货 / 补货点等问法能保留本轮商品引用。
- 让日均销量和供货周期解析优先读取本轮文本中的显式标签。
- 继续保持未选商品时不绑定历史商品或预设商品，也不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链或实时库存数据。

## 本次修改

- 后端 agent：`ops_inventory_planning` schema 新增 `product_ref`。
- 后端 agent：成功输出新增 `商品引用`。
- 后端 worker：库存 / 补货 / 备货自然语言自动补偿改为结构化 input。
- 后端 worker：本轮显式选品后，库存规划 input 带入 `product_ref`。
- 后端 worker：未选商品时不写空 `product_ref`。
- 后端 worker：日均销量和供货周期优先使用带标签的数值。
- API/SSE：同步 `API_DOCS/sse.md`。
- 迁移文档：同步阶段验收、总计划和 skill 矩阵。

## 用户可感知变化

- 用户手动勾选商品后，再问“给这个商品做库存 / 补货 / 备货 / 补货点规划”时，后端会把本轮选中商品的 `product:<uuid>` 带进库存规划结果。
- 用户没有手动选商品时，库存规划仍按普通库存问题运行，但不会暗中绑定历史商品或默认商品。
- 用户写“日均销量120，供货周期7天，帮我算补货点”时，后端会稳定识别日均销量和供货周期，并继续生成安全库存、补货点和建议订货量。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent ops_inventory_planning -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_inventory_planning_ref -- --nocapture`
- [x] `cargo test -p agent inventory_planning_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo test -p agent inventory_optimizer_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`

## 后续缺口

- 真实淘宝、抖音、千川、ERP、WMS、供应链和实时库存数据仍需要后续 service/adapter/search 注入，不能由本地 native skill 伪造或默认读取。
- 前端后续应把库存规划 `skill_result.output.商品引用`、`安全库存`、`补货点`、`建议订货量` 和周转提示展示在真实过程面板与交付物详情中。
