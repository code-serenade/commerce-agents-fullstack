# Post 8-32: Accounting Artifact Materialization

## 1. Background

The migration matrix marks the old product's core accounting skills as native-ready in the new product:

- `accounting_cost_calc`
- `accounting_profit_analysis`
- `accounting_roi_calc`
- `accounting_budget_plan`

These skills already run inside the existing Rust Agent framework, but their successful outputs are not yet persisted as workspace deliverables.

## 2. Goal

Persist successful core accounting skill outputs as real workspace artifacts:

```text
accounting skill_result -> WorkspaceApi::materialize_agent_content_artifact -> workspace_artifact
```

This keeps the feature inside the teammate-provided framework:

- Rust Agent skill executor
- service `WorkspaceApi`
- existing `workspace_artifacts`
- existing `workspace_artifact_created` SSE event
- existing `API_DOCS`

## 3. Scope

This slice covers:

- `accounting_cost_calc` -> `artifactType=accounting_cost_report`
- `accounting_profit_analysis` -> `artifactType=accounting_profit_report`
- `accounting_roi_calc` -> `artifactType=accounting_roi_report`
- `accounting_budget_plan` -> `artifactType=accounting_budget_plan`

No new database table, HTTP route, frontend-only state, or separate reporting system is introduced.

## 4. Agent Output Contract

Successful outputs should include:

- `status=success`
- `has_data=true`
- `data_readiness=user_provided` or `calculated_from_defaults`
- `adapter_status=manual_input`
- `summary`

When a calculation requires user input and the input is missing, the skill should return:

- `status=deferred`
- no artifact materialization

For this slice:

- `accounting_profit_analysis` defers when `revenue` is missing.
- `accounting_roi_calc` defers when both `investment` and `revenue_generated` are missing.
- `accounting_cost_calc` and `accounting_budget_plan` can still produce deterministic outputs with defaults, but their `data_readiness` must disclose that defaults were used.

## 5. Service Design

Extend `build_content_artifact_from_skill_output` in `WorkspaceApi` with accounting mappings.

The existing service method continues to:

1. check `status=success`
2. check supported `skillId`
3. build markdown from structured JSON output
4. write `workspace_artifacts`
5. update `agent_skill_runs.metadata.contentArtifactId`
6. return the artifact to web-server for `workspace_artifact_created`

## 6. Artifact Contract

Common fields:

- `storageKind=inline`
- `format=markdown`
- `mimeType=text/markdown`
- `source=agent_session`
- `metadata.kind=agent_generated_content`
- `metadata.contentCategory=accounting`

Supported artifact types:

- `accounting_cost_report`
- `accounting_profit_report`
- `accounting_roi_report`
- `accounting_budget_plan`

## 7. User-Visible Behavior

Users can ask the accounting worker for cost calculation, profit analysis, ROI calculation, or budget planning. When the backend successfully computes the result, it can become a workspace deliverable and later appear in the existing artifact list/detail/export flow.

If required financial input is missing, the backend returns `deferred` and does not create a fake accounting report.

## 8. Non-Goals

This slice does not:

- connect live store finance data
- create finance history tables
- add frontend charts
- add a separate report service
- perform tax/legal advice beyond deterministic calculation output

## 9. Validation

Required checks:

- `cargo fmt`
- targeted agent tests for accounting success/deferred output contracts
- targeted service test for accounting artifact mapping
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
