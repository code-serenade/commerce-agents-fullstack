# Post 8-30: Data Dashboard Artifact Materialization

## 1. Background

Post 8-29 migrated `data_dashboard` from a deferred placeholder into a native Rust Agent data skill. The skill can now return structured dashboard sections, health status, alerts, and next actions.

The remaining backend gap is persistence: the dashboard result still behaves mainly as a transient skill result unless it is manually copied from the current Agent turn.

## 2. Goal

When `data_dashboard` returns `status=success`, persist the structured output as a real workspace artifact:

```text
data_dashboard skill_result -> workspace_artifact
```

This continues the same backend-first materialization path already used by creative and design deliverables.

## 3. Scope

This slice covers only:

- `data_dashboard` -> `artifactType=data_dashboard`
- `contentCategory=data`
- markdown inline artifact content
- existing SSE `workspace_artifact_created` event

No new database table, route, frontend state, or fake live platform metric source is introduced.

## 4. Service Design

Reuse:

- `WorkspaceApi::materialize_agent_content_artifact`
- `workspace_artifacts`
- `agent_skill_runs.metadata.contentArtifactId`
- SSE `workspace_artifact_created`

The existing service method remains responsible for:

1. verifying user/workspace/run ownership
2. checking that the skill result is `success`
3. checking idempotency through `contentArtifactId`
4. writing the artifact in the same transaction as the skill run metadata update
5. returning the created artifact to the web-server SSE layer

## 5. Artifact Contract

Common fields:

- `artifactType=data_dashboard`
- `storageKind=inline`
- `format=markdown`
- `mimeType=text/markdown`
- `source=agent_session`
- `metadata.kind=agent_generated_content`

Data-specific metadata:

- `contentCategory=data`
- `skillId=data_dashboard`
- `requestId`
- `role`
- `summary`

The markdown body wraps the original structured JSON so the artifact keeps the same fields the backend skill produced, including `has_data` and `adapter_status`.

## 6. User-Visible Behavior

After this slice, a successful data dashboard run can create a workspace deliverable. Users can later find the data dashboard through the existing artifact list/detail/export path instead of losing it inside a single chat turn.

If the dashboard was produced without real store metrics, the artifact still contains the backend's `has_data=false` / `store_metrics_required` signal. The product must not present that artifact as live platform data.

## 7. Non-Goals

This slice does not:

- connect live store metrics
- create dashboard history tables
- add chart rendering
- add frontend-only dashboard cards
- claim platform real-time data when adapters are missing

## 8. Validation

Required checks:

- `cargo fmt`
- targeted service test for `data_dashboard` artifact builder
- existing creative/design artifact builder tests
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p service -p web-server`
