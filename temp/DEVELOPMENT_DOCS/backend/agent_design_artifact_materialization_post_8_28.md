# Post 8-28: Design Deliverable Artifact Materialization

## 1. Background

The new product already has native Rust design skills:

- `design_main_image`
- `design_detail_page`
- `design_color_scheme`
- `design_material_spec`
- `design_campaign_poster`

These skills produce structured design briefs and specs, but the result currently stays mainly in the current Agent turn unless it is included in the final report. Old-product design capabilities are more useful when their outputs become reusable deliverables.

Post 8-27 added the first backend path for skill-result content materialization into `workspace_artifacts`. This slice extends that same path to design outputs.

## 2. Goal

When a supported design skill returns `status=success`, persist its structured output as a real workspace artifact:

```text
design skill_result -> workspace_artifact
```

No new database table, route, or frontend-only state is introduced.

## 3. Supported Skills

This slice covers:

- `design_main_image` -> `artifactType=design_main_image_brief`
- `design_detail_page` -> `artifactType=design_detail_page_brief`
- `design_color_scheme` -> `artifactType=design_color_scheme`
- `design_material_spec` -> `artifactType=design_material_spec`
- `design_campaign_poster` -> `artifactType=design_campaign_poster_brief`

## 4. Service Design

Reuse:

- `WorkspaceApi::materialize_agent_content_artifact`
- `workspace_artifacts`
- `agent_skill_runs.metadata.contentArtifactId`
- SSE `workspace_artifact_created`

The existing method remains responsible for:

1. verifying user/workspace/run ownership
2. checking that the skill result is `success`
3. building a stable markdown artifact from structured output
4. writing the artifact
5. updating `agent_skill_runs.metadata` for idempotency

The only change is expanding the supported skill map from creative-only to content/design deliverables.

## 5. Artifact Contract

Common fields:

- `storageKind=inline`
- `format=markdown`
- `source=agent_session`
- `metadata.kind=agent_generated_content`

Design-specific metadata:

- `contentCategory=design`
- `artifactType`
- `skillId`
- `requestId`
- `role`
- `summary`

## 6. User-Visible Behavior

After this slice, when users ask for:

- main image design plan
- detail page structure
- color system
- platform material spec
- campaign poster brief

the backend will persist the successful skill output as a workspace artifact. Frontend can later show these deliverables through the existing artifact list and detail views.

## 7. Non-Goals

This slice does not:

- generate real image files
- create product assets automatically
- upload objects to storage
- add frontend buttons
- fake image/OCR/material generation

The design deliverable is a structured markdown artifact, not a generated bitmap asset.

## 8. Validation

Required checks:

- `cargo fmt`
- `cargo check -p service`
- `cargo check -p web-server`
- targeted service tests for design artifact builder

