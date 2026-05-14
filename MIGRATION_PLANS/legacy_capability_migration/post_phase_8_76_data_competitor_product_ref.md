# Post 8-76 竞品分析 product_ref 契约

状态：done

## 背景

`data_competitor_analysis` 是旧产品里高频的商品 / 竞品能力。新产品已能做本地 deterministic 竞品矩阵，但还没有把用户手动选择的商品引用带入该 skill，导致后续前端过程面板和 artifact 难以明确知道“这是针对哪个商品做的竞品分析”。

## 本次改动

- `data_competitor_analysis` schema 新增 `product_ref`。
- skill 输出新增 `商品引用`。
- data worker 在本轮显式选品时给竞品分析 input 自动带入 `product_ref` 和选中商品名。
- 未选商品时不写空 `product_ref`，仍按普通竞品分析处理。

## 涉及能力

- `data_competitor_analysis`

## 用户可感知变化

用户手动选择商品后，再问“对这个商品做竞品分析 / 对标竞品 / 价格带对比”时，后端会把本轮选中商品的 `product:<uuid>` 带进竞品分析结果。用户没有选择商品时，系统不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不联网搜索竞品，不调用淘宝、抖音、千川等官方接口。
- 本阶段不执行平台写操作。
- 真实竞品公开资料仍由 `search_competitor` 或后续 adapter 负责。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_competitor_analysis -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_competitor_analysis_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
