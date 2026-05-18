# Post 8-31: Data Analysis Artifact Materialization

## 1. Background

The new product already has several native deterministic data skills in the existing Agent framework:

- `data_funnel_analysis`
- `data_channel_roi`
- `data_refund_decomposition`

These skills can produce useful structured analysis, but their successful outputs are not yet materialized as workspace deliverables. Post 8-30 proved the path for `data_dashboard`.

## 2. Goal

Persist high-value successful data analysis outputs as real workspace artifacts:

```text
data skill_result -> WorkspaceApi::materialize_agent_content_artifact -> workspace_artifact
```

This keeps the migration inside the teammate-provided framework:

- Rust Agent skill output contract
- service `WorkspaceApi`
- existing `workspace_artifacts`
- existing `workspace_artifact_created` SSE event
- existing API docs

## 3. Scope

This slice covers:

- `data_funnel_analysis` -> `artifactType=data_funnel_report`
- `data_channel_roi` -> `artifactType=data_channel_roi_report`
- `data_refund_decomposition` -> `artifactType=data_refund_report`

No new database table, HTTP route, frontend state, or second artifact system is introduced.

## 4. Agent Output Contract

For successful analysis with user-supplied data, the three skills should include:

- `status=success`
- `has_data=true`
- `data_readiness=user_provided`
- `adapter_status=manual_input`
- `summary`

For missing required data, the existing deferred path remains:

- `status=deferred`
- no artifact materialization

This preserves honest degradation. The product must not create a data report artifact when input data is missing.

## 5. Service Design

Extend `build_content_artifact_from_skill_output` in `WorkspaceApi` with data report mappings.

The existing method remains responsible for:

1. checking `status=success`
2. checking support list by `skillId`
3. building markdown from the structured JSON output
4. writing `workspace_artifacts`
5. updating `agent_skill_runs.metadata.contentArtifactId`
6. returning the artifact to web-server for `workspace_artifact_created`

## 6. Artifact Contract

Common fields:

- `storageKind=inline`
- `format=markdown`
- `mimeType=text/markdown`
- `source=agent_session`
- `metadata.kind=agent_generated_content`
- `metadata.contentCategory=data`

Supported artifact types:

- `data_funnel_report`
- `data_channel_roi_report`
- `data_refund_report`

## 7. User-Visible Behavior

After this slice, when users ask the data worker for funnel diagnosis, channel ROI ranking, or refund decomposition and provide enough data, the successful skill output can become a workspace deliverable.

The frontend can later show these reports through the existing artifact list/detail/export path. There is no frontend-only fake report state in this slice.

## 8. Non-Goals

This slice does not:

- connect live platform metrics
- create a data warehouse table
- add charts
- add frontend dashboard cards
- fake external data
- materialize deferred data skills

## 9. Validation

Required checks:

- `cargo fmt`
- targeted agent tests for the three data skill output contracts
- targeted service tests for data report artifact mapping
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
