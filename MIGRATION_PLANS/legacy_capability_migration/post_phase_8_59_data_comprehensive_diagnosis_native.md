# Post 8-59: data_comprehensive_diagnosis Native Orchestrator

## Status

Done.

## Purpose

Migrate old product comprehensive business diagnosis into the new product Rust Agent skill framework.

## Backend Changes

- `data_comprehensive_diagnosis` is no longer a deferred placeholder.
- The skill now orchestrates existing Rust data skills:
  - `data_dashboard`
  - `data_funnel_analysis`
  - `data_anomaly_diagnosis`
  - `data_customer_segmentation`
  - `data_refund_decomposition`
  - `data_demand_forecast`
  - `data_attribution_analysis`
  - `data_ltv_calculator`
- Each sub-diagnosis keeps its own `status`, `adapter_status`, and `data_readiness`.
- Missing sub-diagnoses are surfaced as `缺失数据` instead of being hidden.
- The skill does not read old Python code, Taobao, Douyin, Qianchuan, or warehouse data directly.

## Artifact Changes

Successful output can now become:

- `artifactType=data_comprehensive_diagnosis_report`

Only `status=success` output creates a workspace artifact.

## User-Visible Meaning

After frontend connects this contract, the data analyst virtual employee can produce a single integrated business diagnosis rather than forcing users to inspect many isolated reports. Users will see health score, findings, risks, opportunities, action plan, and missing data in one report.

## Validation

- [x] `cargo fmt --all`
- [x] `cargo fmt --all --check`
- [x] `cargo test -p agent --lib data_comprehensive_diagnosis -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_data_comprehensive_skill_output -- --nocapture`
- [x] `cargo check -p agent -p service`
- [x] `cargo clippy -p agent -p service`

## Remaining Gaps

- Real store metrics, Taobao, Douyin, Qianchuan, and warehouse adapters remain deferred.
- The deterministic report can later be enhanced by Agent runtime LLM synthesis, but not by handler-local LLM calls.
- Frontend still needs to show comprehensive diagnosis artifacts and missing-data guidance in the workbench.
