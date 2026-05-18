# Post 8-83 需求预测 product_ref 契约

状态：done

## 背景

`data_demand_forecast` 是旧产品里需求预测、备货建议和补货判断的重要能力。新产品已能用 Rust native skill 基于历史序列预测未来需求，但还没有把用户手动选择的商品引用带入结果，也缺少把“需求预测 / 备货预测”从普通趋势预测中区分出来的自然问法。

## 本次改动

- `data_demand_forecast` schema 新增 `product_ref`。
- 成功输出新增 `商品引用`。
- data worker 在本轮显式选品时给需求预测 input 自动带入 `product_ref`。
- data worker 对 `需求预测` / `销量预测` / `订单预测` / `备货预测` / `补货预测` / `库存预测` 问法自动触发 `data_demand_forecast`。
- data worker 可解析本轮显式历史数字序列为 `values`。
- 未选商品时不写空 `product_ref`。

## 涉及能力

- `data_demand_forecast`

## 用户可感知变化

用户手动选择商品后，再问“预测这个商品需求 / 销量 / 订单 / 备货量 / 补货量”时，后端会把本轮选中商品的 `product:<uuid>` 带进需求预测结果。用户没有选择商品时，系统仍按普通需求预测问题运行，不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取淘宝、抖音、千川、WMS、ERP 或库存系统实时数据。
- 本阶段不执行真实补货，不生成平台写操作。
- 少于 3 个 `values` 时仍返回 deferred，不创建假需求预测报告。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_demand_forecast -- --nocapture`
- [x] `cargo test -p agent autosuggests_demand_forecast_with_explicit_series -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_demand_forecast_ref -- --nocapture`
- [x] `cargo test -p agent demand_forecast_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
