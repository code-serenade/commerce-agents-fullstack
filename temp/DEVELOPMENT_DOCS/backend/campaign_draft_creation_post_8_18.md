# Post 8-18: Campaign draft creation API

## Requirement Summary

Continue migrating the legacy commerce agent capability into the new product architecture. This slice implements a backend-first campaign draft creation capability so Agent output, workspace deliverables, or user-selected context can be persisted as a real marketing campaign draft.

## Scope

- Add a dedicated campaign draft creation API.
- Reuse the existing `campaigns` table.
- Reuse existing workspace and artifact tables for ownership validation only.
- Do not call LLM.
- Do not execute or publish to any commerce platform.
- Do not add a database migration.

## Existing Tables

### campaigns

Existing table is reused.

Important fields:

- `id`: campaign UUID.
- `user_id`: owner user UUID from JWT.
- `product_id`: optional related product UUID.
- `name`: campaign draft name.
- `campaign_type`: optional campaign category such as `launch`, `promotion`, `ad_review`.
- `description`: draft summary.
- `budget`: planned budget.
- `start_date`: optional text date.
- `end_date`: optional text date.
- `status`: always `draft` for this API.
- `metadata`: draft source and planning context.
- `output`: structured draft output.
- `action_metadata`: future execution hints and approval metadata.

### workspaces

Read-only validation:

- If `workspaceId` is provided, it must belong to the current user and must not be archived.

### workspace_artifacts

Read-only validation:

- If `sourceArtifactId` is provided, `workspaceId` is required.
- The artifact must belong to the current user and workspace and must not be archived.

## HTTP API

### POST /api/app/campaigns/draft

Auth:

- Requires App JWT.
- `user_id` is read from JWT and treated as UUID string.
- Current app user must exist and be enabled.

Request:

```json
{
  "productId": "product-uuid",
  "workspaceId": "workspace-uuid",
  "sourceArtifactId": "artifact-uuid",
  "name": "五一新品冷启动草稿",
  "campaignType": "launch",
  "description": "围绕新品首发做冷启动投放",
  "budget": 3000,
  "startDate": "2026-05-01",
  "endDate": "2026-05-07",
  "goal": "验证首批人群和素材方向",
  "targetAudience": "18-30 岁咖啡兴趣人群",
  "channels": ["douyin", "xiaohongshu"],
  "strategy": "先小预算测素材，再放大高转化计划",
  "sourceType": "agent_output",
  "sourceSummary": "Agent 生成的冷启动方案",
  "draft": {
    "tasks": []
  },
  "metadata": {},
  "actionMetadata": {}
}
```

Response:

- `CampaignDraftResponse`
- Contains the persisted `campaign`, source refs, warnings, and `agentContext`.

## Service Behavior

1. Validate user and app user status.
2. Validate optional product ownership and active status.
3. Validate optional workspace ownership and active status.
4. Validate optional source artifact ownership and active status.
5. Normalize required `name`.
6. Create a `campaigns` record with `status = draft`.
7. Store source context in:
   - `metadata.source`
   - `metadata.planning`
   - `metadata.refs`
8. Store draft content in `output`.
9. Store execution hints in `action_metadata`.

## Error Handling

- Invalid UUID or empty name uses `CAMPAIGN_REQUEST_INVALID`.
- Missing campaign source workspace for artifact uses `CAMPAIGN_REQUEST_INVALID`.
- Product ownership uses existing product errors.
- Workspace ownership uses existing workspace errors.
- Artifact ownership uses existing workspace artifact errors.

## Frontend Contract

Only add API adapter and types in `apps/web/src/api/commerce.ts`.

No new UI in this slice.

## Validation Plan

- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p service campaign_draft`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo check`
- `cargo clippy`
- `pnpm -C apps/web rules`
- `pnpm -C apps/web lint`
- `pnpm -C apps/web test`
- `pnpm -C apps/web build`
- `pnpm rules`
- `git diff --check`
