# Post 8-29: Data Dashboard Native Skill

## 1. Background

`data_dashboard` was still marked as `pending_review` / deferred in the migration matrix. This left a visible gap in the data worker path: users could ask for a store dashboard, but the backend only returned a deferred plan.

The new product already has the Agent runtime, skill registry, SSE skill events, and workspace persistence. This slice upgrades the existing skill inside the current framework instead of adding a separate API or frontend-only dashboard.

## 2. Goal

Turn `data_dashboard` into a native deterministic data skill.

It should:

- accept manually supplied commerce metrics
- calculate dashboard cards, growth/efficiency/risk sections, health score, alerts, and next actions
- clearly mark output as `manual_input` when metrics are supplied
- clearly mark output as `store_metrics_required` when no real metrics are supplied
- avoid pretending that platform data was queried

## 3. Non-Goals

This slice does not:

- create a `store_metrics` table
- add a new HTTP API
- connect a live platform data adapter
- create frontend-only dashboard state
- call LLM

## 4. Backend Design

Changed area:

- `backend/crates/agent/src/skills/data.rs`
- `backend/crates/agent/src/agent/worker/runner.rs`
- `backend/crates/agent/src/skills/runtime.rs`

The skill remains in the existing `DataSkill` executor. The `data_dashboard` spec is now tagged `deterministic`, so runtime metadata reports it as native.

Output contract:

- `status=success`
- `has_data=true` when user metrics are present
- `has_data=false` when no metric input exists
- `adapter_status=manual_input` or `store_metrics_required`
- `核心指标`
- `经营健康度`
- `看板分区`
- `预警`
- `下一步动作`

## 5. User-Visible Behavior

When the user asks for store dashboard / business overview / core metrics, the data worker can now return a structured dashboard instead of a deferred placeholder.

If no real metric input exists, the output explicitly says that store metrics adapter is not connected and the values are only sample baselines. Frontend should render it as missing-data/sample state, not as live platform data.

## 6. Validation Plan

- `cargo fmt`
- `cargo test -p agent data_dashboard_builds_from_user_metrics -- --nocapture`
- `cargo test -p agent data_dashboard_marks_sample_when_metrics_missing -- --nocapture`
- `cargo test -p agent autosuggests_more_specific_data_skills_by_intent -- --nocapture`
- `cargo test -p agent executor_runs_representative_native_categories -- --nocapture`
- `cargo check -p agent`
- `cargo clippy -p agent`
