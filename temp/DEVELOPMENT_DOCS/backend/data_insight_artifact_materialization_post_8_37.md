# Post 8-37: Data Insight Artifact Materialization

## Requirement Summary

Continue migrating old product data-analysis capabilities into the new product framework.

This slice focuses on already native data insight skills:

- `data_anomaly_diagnosis`
- `data_trend_forecast`
- `data_customer_segmentation`
- `data_competitor_analysis`
- `data_ab_test_analyzer`

The goal is to make successful results reusable as workspace artifacts while keeping missing-input states honest.

## Existing Framework Boundary

- Agent execution remains in `backend/crates/agent/src/skills/data.rs`.
- Service materialization remains in `WorkspaceApi::materialize_agent_content_artifact`.
- No new execution framework is introduced.
- No new database table is needed.
- No new HTTP route is needed.
- Frontend should keep using existing workspace artifact APIs and SSE events.

## Data Model

No database migration is required.

The slice reuses existing workspace artifact storage:

- `workspace_artifacts.artifact_type`
- `workspace_artifacts.title`
- `workspace_artifacts.preview`
- `workspace_artifacts.content`
- `workspace_artifacts.metadata`

## API / SSE Contract

No new API route is added.

Existing behavior is extended:

- `skill_result.status=success` plus a supported data insight skill can create `workspace_artifact_created`.
- `status=deferred` does not create a content artifact.
- Artifacts remain markdown and inline.
- `metadata.kind=agent_generated_content`.
- `metadata.contentCategory=data`.

New artifact types:

- `data_anomaly_report`
- `data_trend_forecast_report`
- `data_customer_segmentation_report`
- `data_competitor_report`
- `data_ab_test_report`

## Capability Rules

`data_anomaly_diagnosis`:

- Requires at least three numeric series points.
- Missing or insufficient series returns `deferred`.

`data_trend_forecast`:

- Requires at least two numeric series points.
- Missing or insufficient series returns `deferred`.

`data_customer_segmentation`:

- Can run on deterministic defaults.
- Default-only usage must mark `data_readiness=calculated_from_defaults`.

`data_competitor_analysis`:

- Requires `our_product`.
- Missing competitors can use deterministic comparison competitors and must mark `data_readiness=calculated_from_defaults`.

`data_ab_test_analyzer`:

- Requires control/test visitors and conversions.
- Invalid sample or conversion counts return validation errors from the existing skill contract.

## Validation Plan

- `cargo fmt`
- Targeted `agent` tests for success, defaults, and deferred contracts.
- Targeted `service` test for data insight artifact materialization.
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
