# Post 8-57 Backend Design: Advanced Accounting Skill Contract and Artifact Materialization

## Context

Old product accounting capability contains more than basic cost, profit, ROI, and budget tools. The new product already has Rust-native deterministic implementations for advanced accounting and operating finance skills in `backend/crates/agent/src/skills/accounting.rs`, but several of those outputs did not yet use the unified new-product contract required by the workspace persistence layer.

This slice keeps the colleague-provided framework:

- Agent crate owns skill execution output.
- Service/web-server materializes only supported successful skill outputs.
- Workspace artifact is the durable user-facing delivery record.
- No direct Taobao, Douyin, Qianchuan, ERP, order, or accounting platform adapter is introduced in this slice.

## Goals

1. Normalize advanced accounting skill outputs.
2. Preserve honest data readiness and no-fake-data behavior.
3. Enable workspace artifact creation for advanced accounting reports.
4. Update API and migration docs so frontend can later consume the contract.

## In Scope

Advanced accounting skills:

- `accounting_break_even_calc`
- `accounting_compliance_check`
- `accounting_pl_statement`
- `accounting_budget_vs_actual`
- `accounting_gmv_waterfall`
- `accounting_scenario_analysis`
- `accounting_cash_flow_forecast`
- `accounting_financial_narrative`

Artifact mappings:

- `accounting_break_even_calc` -> `accounting_break_even_report`
- `accounting_compliance_check` -> `accounting_compliance_report`
- `accounting_pl_statement` -> `accounting_pl_statement`
- `accounting_budget_vs_actual` -> `accounting_budget_vs_actual_report`
- `accounting_gmv_waterfall` -> `accounting_gmv_waterfall_report`
- `accounting_scenario_analysis` -> `accounting_scenario_report`
- `accounting_cash_flow_forecast` -> `accounting_cash_flow_forecast_report`
- `accounting_financial_narrative` -> `accounting_financial_narrative_report`

## Out of Scope

- Real platform accounting data adapters.
- Automatic ERP/order/ad-account reconciliation.
- Frontend UI changes.
- New database tables or API routes.
- LLM client changes.

## Output Contract

Successful advanced accounting outputs must include:

- `status=success`
- `has_data=true`
- `adapter_status=manual_input`
- `data_readiness=user_provided` or `calculated_from_defaults`
- `summary`

Missing required input must return:

- `status=deferred`
- `skill`
- `deferred_reason`
- `received_params`
- `side_effects=none`
- `expected_outputs`

The service layer continues to create content artifacts only when `output.status == "success"` and the skill is in the supported content-artifact map.

## User Impact

When frontend later displays workspace artifacts, users can find durable accounting deliverables for break-even, compliance, P&L, budget execution, GMV waterfall, scenario analysis, cash-flow forecast, and financial narrative. If they do not provide required data, the system explains what is missing instead of manufacturing reports.

## Validation

- `cargo test -p agent accounting_ -- --nocapture`
- `cargo test -p service content_artifact_builds_from_accounting_skill_output -- --nocapture`
- `cargo check -p agent -p service`
- `cargo clippy -p agent -p service`
- `git diff --check`
