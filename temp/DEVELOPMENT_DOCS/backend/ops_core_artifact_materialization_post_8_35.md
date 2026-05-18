# Post 8-35: Ops Core Artifact Materialization

## 1. Background

The migration matrix marks the old product's core operations skills as native-ready in the new product:

- `ops_promo_planning`
- `ops_pricing_strategy`
- `ops_channel_strategy`
- `ops_inventory_planning`
- `ops_listing_copy`
- `ops_assortment_planning`
- `ops_execution_plan`

These skills already run through the existing `ops::execute(skill_name, input, context)` path, but successful outputs are not yet persisted as reusable workspace deliverables.

## 2. Goal

Persist successful core ops skill outputs as real workspace artifacts:

```text
ops skill_result -> WorkspaceApi::materialize_agent_content_artifact -> workspace_artifact
```

This keeps the feature inside the teammate-provided framework:

- Rust Agent skill executor
- existing `ops::execute` dispatch path
- service `WorkspaceApi`
- existing `workspace_artifacts`
- existing `workspace_artifact_created` SSE event
- existing `API_DOCS`

## 3. Scope

This slice covers:

- `ops_promo_planning` -> `artifactType=ops_promo_plan`
- `ops_pricing_strategy` -> `artifactType=ops_pricing_strategy`
- `ops_channel_strategy` -> `artifactType=ops_channel_strategy`
- `ops_inventory_planning` -> `artifactType=ops_inventory_plan`
- `ops_listing_copy` -> `artifactType=ops_listing_copy`
- `ops_assortment_planning` -> `artifactType=ops_assortment_plan`
- `ops_execution_plan` -> `artifactType=ops_execution_plan`

No new database table, HTTP route, frontend-only state, or separate operations reporting service is introduced.

## 4. Agent Output Contract

Successful outputs should include:

- `status=success`
- `has_data=true`
- `data_readiness=user_provided` or `calculated_from_defaults`
- `adapter_status=manual_input`
- `summary`

Outputs that cannot be responsibly generated without user input should return:

- `status=deferred`
- `has_data=false`
- `adapter_status=user_input_required`
- `summary`

This prevents the frontend from showing empty or fake operations deliverables.

## 5. Service Design

Extend `build_content_artifact_from_skill_output` in `WorkspaceApi` with core ops mappings.

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
- `metadata.contentCategory=ops`

Supported artifact types:

- `ops_promo_plan`
- `ops_pricing_strategy`
- `ops_channel_strategy`
- `ops_inventory_plan`
- `ops_listing_copy`
- `ops_assortment_plan`
- `ops_execution_plan`

## 7. User-Visible Behavior

Users can ask the operations worker for promotion planning, pricing strategy, channel strategy, inventory planning, listing copy, assortment planning, or execution planning. When the backend succeeds, the output can appear in the workspace artifact list/detail/export flow.

If an operation needs user input and no safe input is available, the backend returns `deferred` and does not create a fake artifact.

## 8. Non-Goals

This slice does not:

- execute platform write actions
- connect live ad, inventory, or channel APIs
- create campaign records
- create frontend-only operation cards
- migrate advanced ops analytics such as ad fatigue or ABC-XYZ classification

## 9. Validation

Required checks:

- `cargo fmt`
- targeted agent tests for core ops success/deferred output contracts
- targeted service test for core ops artifact mapping
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
