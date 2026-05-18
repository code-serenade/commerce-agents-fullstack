# Post 8-39 Backend Design: Web Growth Artifact Materialization

## Requirement Summary

Continue migrating legacy product capabilities into the new product framework with backend-first delivery. This slice keeps using the existing teammate framework:

- Rust Agent skills in `backend/crates/agent/src/skills/web.rs`
- Workspace artifact materialization in `backend/crates/service/src/api/workspace.rs`
- SSE workspace persistence flow
- Root API docs in `API_DOCS/`

No old product code is copied, and no frontend-only fake flow is added.

## Scope

Materialize successful Web growth skill outputs as real workspace artifacts:

- `web_seo_optimize`
- `web_title_generator`
- `web_page_conversion`
- `web_store_design`
- `web_keyword_research`
- `web_product_description`
- `web_title_seo_scorer`

## Database Design

No database schema change.

The existing `workspace_artifacts` table is reused. Web growth outputs are stored as inline markdown artifacts generated from successful skill output.

## API Design

No new HTTP API.

Existing artifact APIs continue to be used:

- `GET /api/app/workspaces/{workspaceId}/artifacts`
- `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}/export`

This slice expands supported `artifactType` values and SSE documentation.

## Authentication

No authentication change.

Artifact creation still happens through the existing authenticated Agent SSE workspace binding path and current user ownership checks.

## Behavior

Web growth skills currently return deterministic native outputs, but not all outputs include `status=success`. Since workspace artifact materialization only persists successful outputs, this slice normalizes Web skill outputs to include:

- `status=success`
- `has_data=true`
- `adapter_status=manual_input`
- `data_readiness=user_provided` or `calculated_from_defaults`
- `summary`

Skills with missing required parameters keep returning validation errors through the existing skill error path and do not create fake artifacts.

## Artifact Types

- `web_seo_plan`
- `web_title_candidates`
- `web_conversion_plan`
- `web_store_design_brief`
- `web_keyword_matrix`
- `web_product_description`
- `web_title_seo_score`

## Validation Plan

- `cargo fmt`
- `cargo test -p agent web_ -- --nocapture`
- `cargo test -p service content_artifact_builds_from_web_ -- --nocapture`
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
