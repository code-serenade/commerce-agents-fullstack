# Post 8-80 渠道归因 product_ref 契约

状态：done

## 背景

`data_attribution_analysis` 是旧产品里投放复盘和渠道贡献分析的重要能力。新产品已能基于本轮显式渠道 GMV、UV、转化率和广告费计算多模型归因，但还没有把用户手动选择的商品引用带入结果，也没有把“渠道归因 / 多触点归因”从普通渠道 ROI 问法里区分出来。

## 本次改动

- `data_attribution_analysis` schema 新增 `product_ref`。
- 成功输出新增 `商品引用`。
- data worker 在本轮显式选品时给渠道归因 input 自动带入 `product_ref`。
- data worker 对 `渠道归因` / `多触点归因` / `归因模型` 问法自动触发 `data_attribution_analysis`。
- `渠道ROI` / `渠道投产` 问法仍触发 `data_channel_roi`。
- 未选商品时不写空 `product_ref`。

## 涉及能力

- `data_attribution_analysis`
- `data_channel_roi` 触达边界

## 用户可感知变化

用户手动选择商品后，再问“对这个商品做渠道归因 / 多触点归因 / 归因模型分析”时，后端会把本轮选中商品的 `product:<uuid>` 带进渠道归因结果。用户没有选择商品时，系统仍按普通渠道归因问题运行，不会暗中绑定历史商品或默认商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不读取平台实时归因数据。
- 本阶段不执行平台写操作。
- 缺多渠道数据时仍返回 deferred，不创建假报告。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data_attribution_analysis -- --nocapture`
- [x] `cargo test -p agent autosuggests_attribution_analysis_with_explicit_channel_metrics -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_attribution_analysis_ref -- --nocapture`
- [x] `cargo test -p agent attribution_analysis_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
