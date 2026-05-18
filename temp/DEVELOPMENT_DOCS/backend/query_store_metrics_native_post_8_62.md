# Post 8-62: Query Store Metrics Native Explicit Input

## 1. Requirement Summary

The migration direction remains:

```text
旧产品能力 = 后端内核能力来源
Kimi Agent 集群 = 前端交互和视觉体验参考
新产品架构 = 唯一落地框架
```

This slice continues backend-first migration by improving old-product `query_store_metrics` inside the new Rust Agent skill framework.

The old product could read `store_metrics` data for the current user. The new product must not copy the old Python runtime and must not pretend Taobao, Douyin, or other platform metrics are available before adapters are implemented. The practical goal for this slice is:

- if the user or upstream Agent step provides explicit metrics in the current skill input, return a structured store-metrics summary;
- if no explicit metrics are available, return a clear deferred state explaining the missing `store_metrics` adapter or missing user data.

## 2. Scope

In scope:

- Update `backend/crates/agent/src/skills/data.rs`.
- Make `query_store_metrics` deterministic and useful from explicit input.
- Keep the output contract compatible with existing Agent `skill_result` SSE.
- Preserve truthful data boundaries: no fake platform data, no official platform adapter in this slice.
- Add unit tests for success and deferred paths.

Out of scope:

- Direct database repository/service read from the `agent` crate.
- Taobao / Douyin / Qianchuan official platform APIs.
- New HTTP endpoint.
- Database schema changes.
- Frontend changes.

## 3. Data / Database Design

No new table is required.

This slice does not read `store_metrics` from database because the `agent` crate should not bypass service-layer user isolation. Future DB-backed metrics should be exposed through a service/adapter boundary and then injected into Agent context or skill input.

## 4. API / SSE Contract

No new endpoint is added.

Existing Agent SSE `skill_result` for `query_store_metrics` can now return one of two shapes.

Explicit metrics:

```json
{
  "status": "success",
  "skill": "query_store_metrics",
  "has_data": true,
  "adapter_status": "manual_input",
  "data_readiness": "explicit_metrics",
  "数据来源": "用户提供指标",
  "days": 30,
  "platform": "douyin",
  "totals": {
    "gmv": 180000.0,
    "orders": 1800,
    "uv": 36000
  },
  "derived_metrics": {
    "conversion_rate": "5.00%",
    "aov": 100.0
  },
  "anomalies": [],
  "anomaly_count": 0
}
```

Missing metrics:

```json
{
  "status": "deferred",
  "skill": "query_store_metrics",
  "has_data": false,
  "adapter_status": "store_metrics_required",
  "data_readiness": "missing_user_input",
  "deferred_reason": "当前没有真实店铺指标输入，也没有 store_metrics adapter；不会伪造平台实时数据"
}
```

## 5. Auth / User Isolation

No user-owned resource is read directly in this slice.

When future DB-backed store metrics are connected, authenticated service-layer logic must validate current `user_id` before injecting metrics into Agent. The Agent skill must only consume already-authorized data.

## 6. Degradation Rules

- No explicit metrics means `deferred`, not fake sample success.
- Platform filter only labels or filters provided records; it does not imply live platform read.
- If `include_anomalies=false`, anomaly output is skipped.
- If previous-period metrics are missing, week-over-week fields are omitted or marked insufficient.

## 7. Validation Plan

Run from `backend/`:

```bash
cargo fmt --all
cargo fmt --all --check
cargo test -p agent query_store_metrics -- --nocapture
cargo test -p agent data_dashboard -- --nocapture
cargo check -p agent
cargo clippy -p agent
git diff --check
```
