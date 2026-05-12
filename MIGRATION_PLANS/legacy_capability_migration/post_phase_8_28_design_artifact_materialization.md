# Post 8-28: Design Artifact Materialization

## Status

Done.

## Goal

Make native design skill outputs reusable in the new product by persisting successful results as real `workspace_artifacts`.

This continues the backend-first migration rule:

```text
design skill_result -> service materialization -> workspace_artifact -> SSE workspace_artifact_created
```

## Backend Change

The existing `WorkspaceApi::materialize_agent_content_artifact` path now supports design deliverables in addition to creative deliverables.

Supported mappings:

- `design_main_image` -> `design_main_image_brief`
- `design_detail_page` -> `design_detail_page_brief`
- `design_color_scheme` -> `design_color_scheme`
- `design_material_spec` -> `design_material_spec`
- `design_campaign_poster` -> `design_campaign_poster_brief`

The artifact is stored as:

- `storageKind=inline`
- `format=markdown`
- `source=agent_session`
- `metadata.kind=agent_generated_content`
- `metadata.contentCategory=design`

The existing `agent_skill_runs.metadata.contentArtifactId` remains the idempotency marker.

## Frontend Impact

No frontend fake button or fake state was added.

Frontend can consume the result through the existing artifact list/detail/export contracts and the existing `workspace_artifact_created` SSE event.

## User-Visible Change

When users ask design workers for main image planning, detail page structure, color scheme, platform material spec, or campaign poster brief, successful backend skill output is no longer only part of the current answer. It becomes a workspace deliverable that can be listed, opened, reused, and exported through the existing artifact workflow.

## Non-Goals

This slice does not:

- generate bitmap images
- create product asset records
- upload files to object storage
- perform OCR or image analysis
- pretend a generated image exists when only a design brief exists

Those require a later real asset generation / object storage pipeline.
