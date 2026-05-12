# Post 8-30: Data Dashboard Artifact Materialization

## Status

Done.

## Goal

Persist successful `data_dashboard` skill output as a real workspace artifact.

Post 8-29 made the data dashboard a native backend skill. This slice connects that native output to the new product's workspace deliverable system.

## Backend Change

`WorkspaceApi::materialize_agent_content_artifact` now supports:

```text
data_dashboard -> artifactType=data_dashboard
```

The created artifact is:

- `format=markdown`
- `storageKind=inline`
- `source=agent_session`
- `metadata.kind=agent_generated_content`
- `metadata.contentCategory=data`

The markdown body preserves the structured JSON from the backend skill result, including `has_data`, `adapter_status`, metric sections, alerts, health status, and next actions.

## API / SSE Contract

No new HTTP route was added.

The existing `workspace_artifact_created` SSE event is reused. `API_DOCS/app.md` and `API_DOCS/sse.md` now document `data_dashboard` as a supported generated content artifact.

## User-Visible Change

When users ask for a store dashboard / data overview and the data skill succeeds, the result can now be saved as a workspace deliverable. It can later be queried, previewed, exported, or reused through the existing artifact flow.

If no real store metrics were supplied, the artifact still carries the backend's `has_data=false` / `adapter_status=store_metrics_required` signal. The frontend must not present that as live platform data.

## Validation

- `cargo fmt`
- `cargo test -p service content_artifact_builds_from_data_dashboard_skill_output -- --nocapture`
- `cargo test -p service content_artifact_builds_from_design_skill_output -- --nocapture`
- `cargo test -p service content_artifact_builds_from_creative_skill_output -- --nocapture`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p service -p web-server`
