# Post 8-68 Backend Design: Data Channel ROI and Refund Native Contract

## Requirement Summary

Continue migrating old-product skill capability into the new product framework. This slice focuses on two old data skills that already have Rust implementations but still carried a deferred-style runtime contract:

- `data_channel_roi`
- `data_refund_decomposition`

The goal is to make them behave as real native Agent skills when the user explicitly provides channel or refund data in the current request, while preserving truthful deferred behavior when data is missing.

## Scope

- No database migration.
- No new HTTP API.
- No new official Taobao, Douyin, Qianchuan, JD, PDD, or other platform adapter.
- No frontend-only fake capability.
- Use the existing Agent skill runtime, worker autosuggest, SSE `skill_result`, and workspace artifact materialization path.

## Backend Contract

### `data_channel_roi`

Input:

- `days`
- `include_organic`
- `channels[]`

Each channel can include:

- `name`
- `gmv`
- `ad_spend`
- `orders`
- `uv`
- `conversion_rate`

Behavior:

- If `channels[]` is provided, execute native calculation and return `status=success`.
- If `channels[]` is missing, return `status=deferred`.
- Runtime metadata should classify the skill as native, not adapter/deferred.
- Autosuggest should parse explicit same-turn channel metrics where possible, such as:
  - `抖音GMV12万，广告费3万，订单600，UV2万`
  - `淘宝GMV8万，广告费2.6万，订单420，UV1.6万`

### `data_refund_decomposition`

Input:

- `days`
- `top_n`
- `items[]`

Each refund item can include:

- `name`
- `refund_amount`
- `refund_rate`
- `gmv`

Behavior:

- If `items[]` is provided, execute native Pareto decomposition and return `status=success`.
- If `items[]` is missing, return `status=deferred`.
- Runtime metadata should classify the skill as native, not adapter/deferred.
- Autosuggest should parse explicit same-turn refund items where possible, such as:
  - `质量问题退款2.4万`
  - `物流破损退款9000`
  - `色差退款4000`

## Authentication and User Isolation

This slice does not read persisted user data. It only consumes explicit same-turn user input passed through the Agent skill request. Therefore it does not introduce new authorization paths.

Future persisted metrics or platform adapter reads must go through service/repo/web-server boundaries with current `user_id` isolation.

## API / SSE

No new SSE event type is added.

Existing events remain:

- `skill_requested`
- `skill_result`
- `workspace_artifact_created` when successful output materializes

`API_DOCS/sse.md` must document that autosuggest can now pass parsed `channels[]` and `items[]` into these skills from explicit user text.

## Artifact Behavior

Existing service materialization remains:

- `data_channel_roi` -> `data_channel_roi_report`
- `data_refund_decomposition` -> `data_refund_report`

Only successful outputs create artifacts. Missing data returns `deferred` and creates no fake report.

## Validation Plan

- `cargo fmt --all`
- `cargo test -p agent data_channel_roi_marks_success_for_user_channels -- --nocapture`
- `cargo test -p agent data_refund_decomposition_marks_success_for_user_items -- --nocapture`
- `cargo test -p agent autosuggests_channel_roi_with_explicit_channel_metrics -- --nocapture`
- `cargo test -p agent autosuggests_refund_decomposition_with_explicit_refund_items -- --nocapture`
- `cargo test -p agent executor_runs_native_data_channel_and_refund_skills -- --nocapture`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
