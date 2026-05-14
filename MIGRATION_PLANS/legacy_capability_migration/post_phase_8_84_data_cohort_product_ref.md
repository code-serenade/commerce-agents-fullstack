# Post 8-84 队列留存 product_ref 契约

状态：done

## 背景

`data_cohort_analysis` 是旧产品里留存、复购和流失节点分析的重要能力。新产品已能用 Rust native skill 生成 deterministic 队列留存矩阵，但还没有把用户手动选择的商品引用带入结果，也缺少从“队列留存 / 同期群 / 复购留存”等自然问法直接触达该 skill 的路由。

## 本次改动

- `data_cohort_analysis` schema 新增 `product_ref`。
- 成功输出新增 `商品引用`。
- data worker 在本轮显式选品时给队列留存 input 自动带入 `product_ref`。
- data worker 对 `队列留存` / `同期群` / `留存分析` / `复购留存` / `新客留存` / `流失节点` 问法自动触发 `data_cohort_analysis`。
- data worker 可解析本轮显式 `首期新客` / `新客数` / `新增客户` 为 `new_customers`。
- data worker 可根据“月 / month / monthly”选择 `cohort_unit=month`，否则默认为 `week`。
- 未选商品时不写空 `product_ref`。

## 涉及能力

- `data_cohort_analysis`

## 用户可感知变化

用户手动选择商品后，再问“分析这个商品的新客留存 / 复购留存 / 队列留存 / 流失节点”时，后端会把本轮选中商品的 `product:<uuid>` 带进队列留存结果。用户没有选择商品时，系统仍按普通队列留存问题运行，不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取淘宝、抖音、千川、会员系统、订单系统、CRM 或平台实时留存数据。
- 本阶段不推断历史对话商品，不使用预设商品或测试商品。
- 缺少 `new_customers` 时仍使用 deterministic 默认估算，并以 `data_readiness=calculated_from_defaults` 明示。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_cohort_analysis -- --nocapture`
- [x] `cargo test -p agent autosuggests_cohort_analysis_with_explicit_retention_terms -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_cohort_analysis_ref -- --nocapture`
- [x] `cargo test -p agent cohort_analysis_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
