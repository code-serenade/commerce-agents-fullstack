# Post 8-58 Backend Design: Data Attribution Native Skill

## Context

Old product `DataAttributionAnalysis` computes multi-touch channel attribution with last-touch, linear, time-decay, and position models. The new product still had `data_attribution_analysis` as a deferred placeholder, so frontend and workspace artifacts could not consume a real backend result.

This slice migrates the capability into the new Rust Agent skill framework. It does not connect official Taobao, Douyin, Qianchuan, ERP, or warehouse APIs.

## Goals

1. Make `data_attribution_analysis` a native deterministic skill.
2. Require explicit channel data from user input or future adapters.
3. Return honest `deferred` output when channel data is missing.
4. Materialize successful output as a workspace artifact.

## Input

`data_attribution_analysis` accepts:

- `days`: integer, optional, default `30`
- `attribution_model`: string, optional, one of `last_touch`, `linear`, `time_decay`, `position`, `all`
- `channels`: array of objects, required for success

Each channel can include:

- `name`
- `gmv`
- `uv`
- `conversion_rate`
- `ad_spend`

## Output Contract

On success:

- `status=success`
- `has_data=true`
- `data_readiness=user_provided`
- `adapter_status=manual_input`
- `summary`
- `各渠道归因明细`
- `关键洞察`
- `预算再分配建议`

On missing or insufficient data:

- `status=deferred`
- `skill=data_attribution_analysis`
- `deferred_reason`
- `received_params`
- `summary=渠道归因分析报告已生成`
- no workspace artifact is created

## Artifact

Successful `data_attribution_analysis` output maps to:

- `contentCategory=data`
- `artifactType=data_attribution_report`
- title `Data attribution report`

## Validation

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p agent --lib data_attribution_analysis -- --nocapture`
- `cargo test -p service content_artifact_builds_from_data_advanced_skill_output -- --nocapture`
- `cargo check -p agent -p service`
- `cargo clippy -p agent -p service`
