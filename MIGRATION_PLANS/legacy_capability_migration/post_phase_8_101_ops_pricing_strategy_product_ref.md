# Post 8-101 Ops Pricing Strategy Product Ref

## 状态

done

## 本次目标

- 把定价策略 `ops_pricing_strategy` 接入新产品 `product_ref=product:{uuid}` 契约。
- 让用户手动选择商品后，定价 / 价格 / 售价 / 成本等问法能保留本轮商品引用。
- 让成本和竞品价格解析优先读取本轮文本中的显式标签。
- 继续保持未选商品时不绑定历史商品或预设商品，也不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链或实时竞品价格数据。

## 本次修改

- 后端 agent：`ops_pricing_strategy` schema 新增 `product_ref`。
- 后端 agent：成功输出新增 `商品引用`。
- 后端 worker：定价 / 价格 / 售价 / 成本自然语言自动补偿改为结构化 input。
- 后端 worker：本轮显式选品后，定价策略 input 带入 `product_ref`。
- 后端 worker：未选商品时不写空 `product_ref`。
- 后端 worker：成本和竞品价格优先使用带标签的数值。
- API/SSE：同步 `API_DOCS/sse.md`。
- 迁移文档：同步阶段验收、总计划和 skill 矩阵。

## 用户可感知变化

- 用户手动勾选商品后，再问“给这个商品做定价策略 / 价格建议 / 售价分析”时，后端会把本轮选中商品的 `product:<uuid>` 带进定价策略结果。
- 用户没有手动选商品时，定价策略仍按普通定价问题运行，但不会暗中绑定历史商品或默认商品。
- 用户写“成本42，竞品价89、99、109”时，后端会稳定识别成本和竞品价格，并继续生成建议零售价、平台佣金、利润率和竞品分析。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent ops_pricing_strategy -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_pricing_strategy_ref -- --nocapture`
- [x] `cargo test -p agent pricing_strategy_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`

## 后续缺口

- 真实淘宝、抖音、千川、ERP、WMS、供应链和实时竞品价格数据仍需要后续 service/adapter/search 注入，不能由本地 native skill 伪造或默认读取。
- 前端后续应把定价策略 `skill_result.output.商品引用`、`建议零售价`、`平台佣金`、`利润率` 和 `竞品分析` 展示在真实过程面板与交付物详情中。
