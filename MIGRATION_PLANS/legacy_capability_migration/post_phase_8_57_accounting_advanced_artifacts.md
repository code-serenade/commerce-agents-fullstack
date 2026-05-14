# Post 8-57: Advanced Accounting Skill Contract and Artifacts

## Status

Done.

## Purpose

Move advanced accounting capabilities from "native calculation exists" to "new-product contract is complete and successful outputs can become workspace artifacts".

## Backend Changes

- `accounting_break_even_calc`
- `accounting_compliance_check`
- `accounting_pl_statement`
- `accounting_budget_vs_actual`
- `accounting_gmv_waterfall`
- `accounting_scenario_analysis`
- `accounting_cash_flow_forecast`
- `accounting_financial_narrative`

These now return unified Agent skill fields where applicable:

- `status`
- `has_data`
- `data_readiness`
- `adapter_status`
- `summary`

Required-input gaps now return `status=deferred` instead of untyped JSON hints or error-only payloads.

## Artifact Changes

`WorkspaceApi::materialize_agent_content_artifact` now supports:

- `accounting_break_even_report`
- `accounting_compliance_report`
- `accounting_pl_statement`
- `accounting_budget_vs_actual_report`
- `accounting_gmv_waterfall_report`
- `accounting_scenario_report`
- `accounting_cash_flow_forecast_report`
- `accounting_financial_narrative_report`

Only `status=success` outputs create artifacts. Deferred or missing-data outputs do not create fake accounting reports.

## User-Visible Meaning

After frontend connects workspace artifacts, users can see advanced finance deliverables as durable reports in the workspace. The system will distinguish reports based on user-supplied data from reports using disclosed default assumptions.

## Validation

- [x] `cargo test -p agent accounting_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_accounting_skill_output -- --nocapture`
- [x] `cargo check -p agent -p service`
- [x] `cargo clippy -p agent -p service`
- [x] `git diff --check`

## Remaining Gaps

- Real Taobao/Douyin/Qianchuan/ERP/order/accounting adapters are still not connected.
- Frontend still needs to surface these reports in workspace artifacts and process panels.
- Real platform write actions still require approval and adapter work in later slices.
