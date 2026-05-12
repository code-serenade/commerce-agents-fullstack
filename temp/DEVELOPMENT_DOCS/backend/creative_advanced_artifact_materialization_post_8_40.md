# Post 8-40 Backend Design: Creative Advanced Artifact Materialization

## Requirement Summary

Continue the legacy capability migration with backend-first implementation in the existing teammate framework.

This slice fixes and extends creative skill artifact materialization:

- Keep using Rust Agent skills in `backend/crates/agent/src/skills/creative.rs`.
- Keep using `WorkspaceApi::materialize_agent_content_artifact`.
- Keep using the existing SSE workspace persistence flow.
- Do not add a parallel UI or fake frontend state.

## Scope

1. Normalize creative native skill outputs so existing creative artifact mappings can actually persist during SSE.
2. Add artifact support for advanced creative skills:
   - `creative_trend_catch`
   - `creative_title_ctr_scorer`

## Database Design

No database schema change.

The existing `workspace_artifacts` table is reused. Creative outputs are stored as inline markdown artifacts generated from successful skill output.

## API Design

No new HTTP API.

Existing artifact APIs continue to be used:

- `GET /api/app/workspaces/{workspaceId}/artifacts`
- `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}/export`

This slice expands supported `artifactType` values and corrects the creative output contract.

## Authentication

No authentication change.

Artifact creation still happens through the existing authenticated Agent SSE workspace binding path and current user ownership checks.

## Behavior

Creative native outputs are normalized to include:

- `status=success`
- `has_data=true`
- `adapter_status=manual_input`
- `data_readiness=user_provided` or `calculated_from_defaults`
- `summary`

Skills with missing required parameters keep returning validation errors through the existing `SkillError` path and do not create fake artifacts.

## Artifact Types

Existing creative artifact mappings become effective because output now includes `status=success`:

- `video_script`
- `seeding_copy`
- `ip_branding`
- `content_calendar`
- `live_script`
- `product_article`

New advanced creative artifact types:

- `creative_trend_report`
- `creative_title_ctr_score`

## Validation Plan

- `cargo fmt`
- `cargo test -p agent creative_ -- --nocapture`
- `cargo test -p service content_artifact_builds_from_creative_ -- --nocapture`
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
