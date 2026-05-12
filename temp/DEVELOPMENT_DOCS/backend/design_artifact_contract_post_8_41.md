# Post 8-41 Backend Design: Design Artifact Contract Completion

## Requirement Summary

Continue the old-product capability migration inside the teammate-provided new product framework.

This slice is backend-first. It fixes the existing design skill artifact path rather than adding a new frontend-only surface.

## Scope

1. Keep using Rust Agent skills in `backend/crates/agent/src/skills/design.rs`.
2. Keep using `WorkspaceApi::materialize_agent_content_artifact`.
3. Normalize design native skill outputs so existing design artifact mappings persist reliably during SSE.
4. Preserve honest boundaries:
   - design skills generate structured briefs/specs
   - they do not generate image files in this slice
   - missing required parameters remain validation errors

## Database Design

No database schema change.

Existing `workspace_artifacts` stores generated design markdown artifacts.

## API Design

No new HTTP API.

Existing artifact APIs continue to serve the outputs:

- `GET /api/app/workspaces/{workspaceId}/artifacts`
- `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}/export`

## Behavior

Design native outputs now include:

- `status=success`
- `has_data=true`
- `adapter_status=manual_input`
- `data_readiness=user_provided` or `calculated_from_defaults`
- `summary`

Supported design artifacts:

- `design_main_image_brief`
- `design_detail_page_brief`
- `design_color_scheme`
- `design_material_spec`
- `design_campaign_poster_brief`

## Authentication

No authentication change.

Artifact creation still happens through the existing authenticated Agent SSE workspace binding path and current user ownership checks.

## Frontend Impact

No frontend implementation in this slice.

The frontend should continue consuming real `workspace_artifact_created` SSE events and workspace artifact APIs. It should not show generated image files because this slice only creates structured markdown briefs/specs.

## Validation Plan

- `cargo fmt`
- `cargo test -p agent design_ -- --nocapture`
- `cargo test -p service content_artifact_builds_from_design_ -- --nocapture`
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
