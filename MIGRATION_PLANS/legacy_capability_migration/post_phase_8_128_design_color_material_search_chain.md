# Post 8-128: Design Color / Material Search Chain

## Status

done

## Goal

Move the old product's hidden design color-reference and platform material-spec lookup behavior into the new product's visible Agent skill-chain model.

## New Product Implementation

- `design_color_scheme` can now be followed by `search_trends` when the user asks for latest color trends, popular palettes, competitor color references, online search, or public sources.
- `design_material_spec` can now be followed by `search_platform_policy(policy_type=material)` when the user asks for latest platform material specs, ad-slot sizes, upload format, file size, safe area, or review requirements.
- Pure material rule lookup requests trigger only `search_platform_policy(policy_type=material)`, so the system does not create an unrelated material-spec artifact.
- Ordinary color-scheme and material-spec tasks remain native-only.
- `design` role permission now includes `search_platform_policy` through the existing role skill catalog.

## Product Context Boundary

- Selected product context is copied only from the current turn's explicit `context.selected_summary`.
- Without an explicit selected product, the worker does not write `product_ref/product_name/product_category`.
- This keeps selected-product and no-product conversation paths separate.

## User-Visible Change

- A user asking for a color scheme plus latest color trends will see the color-scheme step and a separate external trend-search step.
- A user asking for ad material specs plus latest platform size/review requirements will see the material-spec step and a separate platform-policy search step.
- A user only asking to search latest material size rules will see only the search step.
- If the external search adapter is not configured or produces no trusted source, the system degrades visibly instead of inventing platform material rules.

## Files

- `backend/crates/agent/src/agent/worker/runner.rs`
- `backend/crates/agent/src/roles/skill_catalog.rs`
- `backend/crates/agent/src/skills/search.rs`
- `API_DOCS/sse.md`
- `API_DOCS/app.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `temp/DEVELOPMENT_DOCS/backend/design_color_material_search_chain_post_8_128.md`

## Acceptance

- [x] Implemented with the new product Agent worker / role catalog / search skill framework.
- [x] No new frontend-only button or fake data path.
- [x] No direct Taobao / Douyin platform write integration.
- [x] API and migration docs updated.
- [x] Tests cover native-only, native + search, pure search, role permission, and selected product boundaries.
