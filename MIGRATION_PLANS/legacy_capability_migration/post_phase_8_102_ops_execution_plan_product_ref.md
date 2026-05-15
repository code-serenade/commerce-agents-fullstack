# Post 8-102 Ops Execution Plan Product Ref

## 状态

done

## 本次目标

- 把运营执行计划 `ops_execution_plan` 补齐到新产品 `product_ref=product:{uuid}` 契约。
- 让用户手动选择商品后，执行计划 / 冷启动计划 / 运营计划等问法能保留本轮商品引用。
- 继续保持未选商品时不绑定历史商品或预设商品，也不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链或实时平台数据。

## 本次修改

- 后端 agent：`ops_execution_plan` 成功输出新增 `商品引用`。
- 后端 worker：运营岗位兜底执行计划自动补偿改为结构化 input。
- 后端 worker：本轮显式选品后，执行计划 input 带入 `product_ref`。
- 后端 worker：未选商品时不写空 `product_ref`。
- API/SSE：同步 `API_DOCS/sse.md`。
- 迁移文档：同步阶段验收、总计划和 skill 矩阵。

## 用户可感知变化

- 用户手动勾选商品后，再问“给这个商品做运营执行计划 / 新品冷启动计划”时，后端会把本轮选中商品的 `product:<uuid>` 带进执行计划结果。
- 用户没有手动选商品时，执行计划仍按普通运营目标生成，但不会暗中绑定历史商品、预设商品或测试商品。
- 前端展示 skill 过程或交付物详情时，可以用 `skill_result.output.商品引用` 区分“指定商品计划”和“未指定商品计划”。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent ops_execution_plan -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_execution_plan_ref -- --nocapture`
- [x] `cargo test -p agent execution_plan_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo test -p agent autosuggests_ops_execution_plan_without_llm_tool_json -- --nocapture`
- [x] `cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`

## 后续缺口

- 真实淘宝、抖音、千川、ERP、WMS、供应链和实时平台数据仍需要后续 service/adapter/search 注入，不能由本地 native skill 伪造或默认读取。
- 前端后续应把执行计划 `skill_result.output.商品引用`、`30天执行计划`、`AARRR指标` 和 `风险预案` 展示在真实过程面板与交付物详情中。
