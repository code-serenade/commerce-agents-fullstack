# Post 8-68：渠道 ROI / 退款拆解 native 契约修正

## 背景

旧产品里 `DataChannelROI` 和 `DataRefundDecomposition` 是数据岗位的核心分析能力。新产品已经有 Rust 实现和 artifact 沉淀链路，但 runtime spec 仍带 `deferred` / `deterministic_fallback` 标签，导致前端或审计侧容易把它们理解成 adapter 型能力。

本轮把这两个 skill 的契约修正为：有本轮显式用户数据时就是 native 执行；没有数据时仍 truthful deferred。

## 本次修改

- `data_channel_roi` 的 skill spec 改为 `deterministic`，运行时 metadata 为 `execution_mode=native`、`risk_level=low`。
- `data_refund_decomposition` 的 skill spec 改为 `deterministic`，运行时 metadata 为 `execution_mode=native`、`risk_level=low`。
- worker autosuggest 在渠道 ROI 意图中解析本轮显式渠道指标，生成 `channels[]`。
- worker autosuggest 在退款帕累托 / 退款拆解意图中解析本轮显式退款项，生成 `items[]`。
- 保留原有缺数据边界：没有 `channels[]` 或 `items[]` 时仍返回 `deferred`，不伪造平台数据。
- 沿用既有 workspace artifact 沉淀：成功的 `data_channel_roi` 可生成 `data_channel_roi_report`，成功的 `data_refund_decomposition` 可生成 `data_refund_report`。

## 用户可感知变化

- 用户在问题里直接写出渠道指标，例如“抖音 GMV12 万，广告费 3 万，订单 600，UV2 万；淘宝 GMV8 万，广告费 2.6 万”，后端可以直接进入渠道 ROI 计算，而不是只提示缺数据。
- 用户在问题里直接写出退款原因和金额，例如“质量问题退款 2.4 万，物流破损退款 9000，色差退款 4000”，后端可以直接生成退款帕累托拆解。
- 如果用户没有提供这些明细，系统仍明确返回缺数据 / 待接入，不会假装已经读取淘宝、抖音、千川或其它平台数据。

## 边界

- 本轮不新增 HTTP API。
- 本轮不新增数据库表。
- 本轮不接官方平台 adapter。
- 本轮不改前端；前端后续应消费真实 `skill_result.execution_mode`、`risk_level`、`status` 和 artifact 事件。

## 验证

- `cargo test -p agent data_channel_roi_marks_success_for_user_channels -- --nocapture`
- `cargo test -p agent data_refund_decomposition_marks_success_for_user_items -- --nocapture`
- `cargo test -p agent autosuggests_channel_roi_with_explicit_channel_metrics -- --nocapture`
- `cargo test -p agent autosuggests_refund_decomposition_with_explicit_refund_items -- --nocapture`
- `cargo test -p agent executor_runs_native_data_channel_and_refund_skills -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
