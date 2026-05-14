# Post 8-85 季节性分解 product_ref 契约

状态：done

## 背景

`data_seasonal_decompose` 是旧产品里识别销量、订单、GMV 周期性波动和季节性节奏的重要能力。新产品已能用 Rust native skill 基于用户提供的时序数据做 STL-lite 分解，但还没有把用户手动选择的商品引用带入结果，也缺少从“季节性 / 周期性 / 周内波动”等自然问法直接触达该 skill 的路由。

## 本次改动

- `data_seasonal_decompose` schema 新增 `product_ref`。
- 成功输出新增 `商品引用`。
- data worker 在本轮显式选品时给季节性分解 input 自动带入 `product_ref`。
- data worker 对 `季节性` / `周期性` / `周期分解` / `季节分解` / `周内波动` / `周期波动` 问法自动触发 `data_seasonal_decompose`。
- data worker 可解析本轮显式历史数字序列为 `values`。
- 未选商品时不写空 `product_ref`。

## 涉及能力

- `data_seasonal_decompose`

## 用户可感知变化

用户手动选择商品后，再问“分析这个商品的季节性 / 周内波动 / 周期趋势”时，后端会把本轮选中商品的 `product:<uuid>` 带进季节性分解结果。用户没有选择商品时，系统仍按普通时序分解问题运行，不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取淘宝、抖音、千川、平台历史销售、活动日历或行业外部趋势数据。
- 本阶段不推断历史对话商品，不使用预设商品或测试商品。
- 少于 `period * 2` 个 `values` 时仍返回 deferred，不创建假季节性分解报告。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_seasonal_decompose -- --nocapture`
- [x] `cargo test -p agent autosuggests_seasonal_decompose_with_explicit_series -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_seasonal_decompose_ref -- --nocapture`
- [x] `cargo test -p agent seasonal_decompose_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
