# Post 8-82 价格弹性 product_ref 契约

状态：done

## 背景

`data_price_elasticity` 是旧产品里定价、促销和调价影响分析的重要能力。新产品已能用 Rust native skill 计算价格弹性系数、敏感度和最优价格参考，但还没有把用户手动选择的商品引用带入结果，也缺少直接触达价格弹性 skill 的自然问法。

## 本次改动

- `data_price_elasticity` schema 新增 `product_ref`。
- 成功输出新增 `商品引用`。
- data worker 在本轮显式选品时给价格弹性 input 自动带入 `product_ref`。
- data worker 对 `价格弹性` / `价格敏感` / `需求弹性` / `调价测试` / `涨价影响` / `降价影响` 问法自动触发 `data_price_elasticity`。
- data worker 可解析本轮显式 `价格-销量` 点、成本和当前售价。
- 未选商品时不写空 `product_ref`。

## 涉及能力

- `data_price_elasticity`

## 用户可感知变化

用户手动选择商品后，再问“分析这个商品价格弹性 / 调价影响 / 涨价降价影响”时，后端会把本轮选中商品的 `product:<uuid>` 带进价格弹性结果。用户没有选择商品时，系统仍按普通价格弹性问题运行，不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取淘宝、抖音、千川、CRM 或调价系统实时数据。
- 本阶段不执行真实调价，不生成平台写操作。
- 缺 `price_points` 时仍使用 deterministic 默认估算，并以 `data_readiness=calculated_from_defaults` 明示。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_price_elasticity -- --nocapture`
- [x] `cargo test -p agent autosuggests_price_elasticity_with_explicit_price_points -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_price_elasticity_ref -- --nocapture`
- [x] `cargo test -p agent price_elasticity_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
