# Post 8-77 漏斗分析 product_ref 契约

状态：done

## 背景

`data_funnel_analysis` 是电商经营分析的核心能力之一。新产品已经能基于本轮显式曝光、点击、加购、下单和付款数据做 native 漏斗诊断，但还没有把用户手动选择的商品引用带入结果。

## 本次改动

- `data_funnel_analysis` schema 新增 `product_ref`。
- 成功输出新增 `商品引用`。
- data worker 在本轮显式选品时给漏斗分析 input 自动带入 `product_ref`。
- 未选商品时不写空 `product_ref`。

## 涉及能力

- `data_funnel_analysis`

## 用户可感知变化

用户手动选择商品后，再问“分析这个商品的转化漏斗 / 曝光点击加购下单付款”时，后端会把本轮选中商品的 `product:<uuid>` 带进漏斗分析结果。用户没有选择商品时，系统仍按普通漏斗指标问题运行，不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取平台实时漏斗数据。
- 本阶段不执行平台写操作。
- 缺曝光/点击等漏斗数据时仍返回 deferred，不创建假报告。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_funnel_analysis -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_funnel_analysis_ref -- --nocapture`
- [x] `cargo test -p agent funnel_analysis_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
