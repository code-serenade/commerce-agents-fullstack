# Post 8-36: Ops Advanced Artifact Materialization

## Requirement Summary

Continue migrating old product operations capabilities into the new product framework.

This slice focuses on already native advanced operations skills:

- `ops_smart_pricing`
- `ops_inventory_optimizer`
- `ops_ad_fatigue_detector`
- `ops_abc_xyz_classifier`

The goal is to make successful results reusable as workspace artifacts while keeping missing-input states honest.

## Existing Framework Boundary

- Agent execution remains in `backend/crates/agent/src/skills/ops.rs`.
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

- `skill_result.status=success` plus a supported advanced ops skill can create `workspace_artifact_created`.
- `status=deferred` does not create a content artifact.
- Artifacts remain markdown and inline.
- `metadata.kind=agent_generated_content`.
- `metadata.contentCategory=ops`.

New artifact types:

- `ops_smart_pricing_report`
- `ops_inventory_optimization_report`
- `ops_ad_fatigue_report`
- `ops_abc_xyz_report`

## Capability Rules

`ops_smart_pricing`:

- Requires valid `cost` and `current_price`.
- Missing `competitor_prices` is allowed but must mark `data_readiness=calculated_from_defaults`.

`ops_inventory_optimizer`:

- Requires `skus`.
- Missing `skus` returns `deferred`.

`ops_ad_fatigue_detector`:

- Requires `channels`.
- Missing `channels` returns `deferred`.

`ops_abc_xyz_classifier`:

- Can use deterministic demo SKUs.
- Demo usage must mark `data_readiness=calculated_from_defaults`.

## Validation Plan

- `cargo fmt`
- Targeted `agent` tests for success, defaults, and deferred contracts.
- Targeted `service` test for advanced ops artifact materialization.
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
