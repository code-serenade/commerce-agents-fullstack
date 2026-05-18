# Post 8-63: Query Store Metrics Reachability

## 1. Requirement Summary

Continue backend-first migration under the existing strategy:

```text
旧产品能力 = 后端内核能力来源
Kimi Agent 集群 = 前端交互和视觉体验参考
新产品架构 = 唯一落地框架
```

Post 8-62 made `query_store_metrics` useful when explicit metrics are passed to the skill. This slice makes that backend capability easier to reach from normal user language. The user should not need to know the exact skill JSON shape.

## 2. Scope

In scope:

- Update Agent worker autosuggest logic in `backend/crates/agent/src/agent/worker/runner.rs`.
- Detect explicit store-metric statements such as GMV, orders, UV, clicks, ad spend, refund, gross profit.
- Build `query_store_metrics` input from natural text.
- Let data, ops, and accounting roles auto-request `query_store_metrics` when the intent is clearly metric loading/querying.
- Preserve existing more-specific data skills such as dashboard, channel ROI, refund decomposition, anomaly, forecast, and funnel.

Out of scope:

- No new database table.
- No new HTTP API.
- No frontend change.
- No platform data adapter.
- No direct service/database read from the `agent` crate.

## 3. Data / Database Design

No schema change.

The autosuggest layer only parses user text into a skill input object. `query_store_metrics` still owns truthful handling of success vs deferred state.

## 4. API / SSE Contract

No endpoint or event shape changes.

The same `skill_requested` and `skill_result` events are used. The practical behavior change is that backend may now emit:

```text
skill_requested.skill_id = query_store_metrics
```

for clear store metric loading/query intents.

## 5. Auth / User Isolation

No user-owned DB resources are read in this slice. If future store metrics are read from database, that must be injected by authenticated service-layer adapter logic.

## 6. Degradation Rules

- If the user provides numbers, they are treated as user-provided metrics.
- If the user only asks to query/load metrics and no metrics are available, `query_store_metrics` returns deferred.
- The autosuggest layer must not convert “dashboard/report/forecast/refund/channel ROI” intents away from their more specific skills.

## 7. Validation Plan

Run from `backend/`:

```bash
cargo fmt --all
cargo fmt --all --check
cargo test -p agent autosuggests_query_store_metrics -- --nocapture
cargo test -p agent autosuggests_more_specific_data_skills -- --nocapture
cargo test -p agent query_store_metrics -- --nocapture
cargo check -p agent
cargo clippy -p agent
git diff --check
```
