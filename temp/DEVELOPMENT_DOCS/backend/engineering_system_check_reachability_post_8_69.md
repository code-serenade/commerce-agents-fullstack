# Post 8-69 Backend Design: Engineering System Check Reachability

## Requirement Summary

Continue backend-first migration of old product skills into the new product architecture. The old product has `EngineeringSystemCheck`, which checks configured platform API connections. The new product already has `engineering_system_check`, but it was not naturally reachable from engineering user prompts.

This slice makes the skill reachable from explicit platform health / API connectivity language, while preserving the existing adapter and approval boundary.

## Scope

- No database migration.
- No new HTTP API.
- No real Taobao, Tmall, Douyin, Qianchuan, JD, PDD, or other official platform call.
- No frontend-only fake status.
- Use existing Agent worker autosuggest, role skill catalog, skill runtime, and SSE `skill_requested` / `skill_result`.

## Backend Contract

`engineering_system_check` accepts:

- `platform`: optional target platform.
- `test_live`: optional boolean.

Behavior:

- If `test_live=false`, the skill returns `status=deferred` because Rust Agent runtime does not read persisted platform connection config directly.
- If `test_live=true`, the skill returns `status=pending_approval`, `approval_required=true`, and does not call platform APIs directly.
- Runtime metadata should classify the skill as an adapter / platform boundary with high risk and platform adapter requirement.

Autosuggest should trigger this skill when the engineering worker sees explicit platform health wording, such as:

- `检查一下淘宝平台API连通性和接口健康情况`
- `抖音平台连接状态是否正常`
- `测试一下千川授权和同步状态`

Generic `系统健康 / 当前状态 / 任务进度` should still go to `coordination_status_query`, not platform health check.

## API / SSE

No new SSE event type is added.

Existing events remain:

- `skill_requested`
- `skill_result`
- `approval_required` if downstream web-server turns high-risk skill result into an approval record

`API_DOCS/sse.md` should document that `engineering_system_check` can now be auto-suggested from platform health language, but still returns `deferred` or `pending_approval`.

## Validation Plan

- `cargo test -p agent autosuggests_engineering_system_check_with_platform_boundary -- --nocapture`
- `cargo test -p agent autosuggests_engineering_skill_by_intent -- --nocapture`
- `cargo test -p agent executor_marks_engineering_system_check_as_platform_boundary -- --nocapture`
- `cargo test -p agent engineering_system_check_stays_pending_approval_for_live_test -- --nocapture`
- `cargo test -p agent keeps_roles_from_requesting_unowned_skills -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
