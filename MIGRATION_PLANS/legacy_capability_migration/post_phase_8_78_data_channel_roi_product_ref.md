# Post 8-78 渠道 ROI product_ref 契约

状态：done

## 背景

`data_channel_roi` 是旧产品里投放复盘和渠道效率分析的重要能力。新产品已能基于本轮显式渠道 GMV、广告费、订单和 UV 计算 ROI，但还没有把用户手动选择的商品引用带入结果。

## 本次改动

- `data_channel_roi` schema 新增 `product_ref`。
- 成功输出新增 `商品引用`。
- data worker 在本轮显式选品时给渠道 ROI input 自动带入 `product_ref`。
- 未选商品时不写空 `product_ref`。

## 涉及能力

- `data_channel_roi`

## 用户可感知变化

用户手动选择商品后，再问“复盘这个商品的渠道 ROI / 投放渠道效果 / 渠道投产”时，后端会把本轮选中商品的 `product:<uuid>` 带进渠道 ROI 分析结果。用户没有选择商品时，系统仍按普通渠道指标问题运行，不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取平台实时投放数据。
- 本阶段不执行平台写操作。
- 缺渠道数据时仍返回 deferred，不创建假报告。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_channel_roi -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_channel_roi_ref -- --nocapture`
- [x] `cargo test -p agent channel_roi_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
