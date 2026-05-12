# Post 8-38 Backend Design: Data Advanced Artifact Materialization

## Requirement Summary

Continue the legacy capability migration with backend-first development. The new product must use the existing teammate framework instead of copying old product code or adding a parallel execution path.

This slice materializes advanced data worker outputs as workspace artifacts:

- `data_multi_period_trend`
- `data_ltv_calculator`
- `data_cohort_analysis`
- `data_seasonal_decompose`
- `data_price_elasticity`
- `data_demand_forecast`

## Existing Framework Entry Points

- Agent skill execution: `backend/crates/agent/src/skills/data.rs`
- Workspace artifact materialization: `backend/crates/service/src/api/workspace.rs`
- SSE artifact event contract: `API_DOCS/sse.md`
- App artifact query/export contract: `API_DOCS/app.md`

No new executor, table, HTTP route, or frontend-only fake flow is introduced.

## Database Design

No database schema change.

The existing `workspace_artifacts` table is reused. Advanced data reports are stored as inline markdown artifacts generated from successful skill output.

## API Design

No new HTTP API.

Existing artifact APIs continue to be used:

- `GET /api/app/workspaces/{workspaceId}/artifacts`
- `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}/export`

This slice only expands supported `artifactType` values.

## Authentication

No authentication change.

Artifact creation still happens inside the existing Agent SSE workspace binding flow and uses the current App user ownership checks.

## Behavior

When a supported data skill returns `status=success`, `WorkspaceApi::materialize_agent_content_artifact` creates a markdown artifact with:

- `metadata.kind=agent_generated_content`
- `metadata.contentCategory=data`
- `metadata.skillId`
- `metadata.artifactType`
- `metadata.summary`

When required user data is missing, the skill returns `deferred` or validation failure and no artifact is created.

Default deterministic estimates are allowed only when the skill already documents that behavior through `data_readiness=calculated_from_defaults`.

## Artifact Types

- `data_multi_period_trend_report`
- `data_ltv_report`
- `data_cohort_report`
- `data_seasonal_decompose_report`
- `data_price_elasticity_report`
- `data_demand_forecast_report`

## Validation Plan

- `cargo fmt`
- `cargo test -p agent data_ -- --nocapture`
- `cargo test -p service content_artifact_builds_from_data_ -- --nocapture`
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
