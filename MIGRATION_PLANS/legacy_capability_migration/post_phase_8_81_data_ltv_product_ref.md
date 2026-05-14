# Post 8-81 LTV 客户价值 product_ref 契约

状态：done

## 背景

`data_ltv_calculator` 是旧产品里 LTV、复购和获客成本回收分析的重要能力。新产品已能用 Rust native skill 计算 LTV、LTV/CAC 和回收周期，但还没有把用户手动选择的商品引用带入结果，也缺少直接触达 LTV skill 的自然问法。

## 本次改动

- `data_ltv_calculator` schema 新增 `product_ref`。
- 成功输出新增 `商品引用`。
- data worker 在本轮显式选品时给 LTV input 自动带入 `product_ref`。
- data worker 对 `LTV` / `客户价值` / `生命周期价值` / `获客成本` / `回收周期` 问法自动触发 `data_ltv_calculator`。
- data worker 可解析本轮显式 `CAC`、客单价、月购买频次、毛利率、月留存率。
- 未选商品时不写空 `product_ref`。

## 涉及能力

- `data_ltv_calculator`

## 用户可感知变化

用户手动选择商品后，再问“分析这个商品 LTV / 客户价值 / 获客成本回收周期”时，后端会把本轮选中商品的 `product:<uuid>` 带进 LTV 结果。用户没有选择商品时，系统仍按普通 LTV 问题运行，不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取会员系统、淘宝、抖音、千川或其它平台实时数据。
- 本阶段不执行平台写操作。
- 缺 LTV 参数时仍使用 deterministic 默认估算，并以 `data_readiness=calculated_from_defaults` 明示。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_ltv_calculator -- --nocapture`
- [x] `cargo test -p agent autosuggests_ltv_calculator_with_explicit_value_metrics -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_ltv_calculator_ref -- --nocapture`
- [x] `cargo test -p agent ltv_calculator_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
