# Post 8-75 商品指标查询 product_ref 契约

状态：done

## 背景

`query_product_metrics` 已经可以基于本轮显式商品指标生成商品 / SKU 指标分析，但它仍主要使用旧 `product_id/product_name/sku_id` 筛选。为了和新产品“用户手动勾选商品”链路一致，需要让该 skill 支持 `product_ref=product:{uuid}`。

## 本次改动

- `query_product_metrics` schema 新增 `product_ref`。
- 成功和 deferred 输出的 `query` 都回显 `product_ref`。
- worker 在用户有本轮显式选品时，商品指标查询自动补偿 `product_ref` 和商品名。
- 未选商品时不写空 `product_ref`，不从历史摘要、默认商品或测试商品推断。

## 涉及能力

- `query_product_metrics`

## 用户可感知变化

用户手动选择商品后，再问“查询这个商品的指标 / 单品数据 / SKU 表现”时，后端 skill input 和结果会带着本轮选中的 `product:<uuid>`。如果用户没有选择商品，系统仍按普通商品指标查询处理，并明确要求用户提供指标或后续 adapter 注入数据，不会自动套用某个预设商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取平台实时商品数据。
- 本阶段不执行平台写操作。
- 旧 `product_id` 继续保留为兼容字段。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent query_product_metrics -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_product_metrics_ref -- --nocapture`
- [x] `cargo test -p agent product_metrics_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
