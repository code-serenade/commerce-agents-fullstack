# Post 8-67: Store Metrics Artifact Materialization

## 1. Requirement Summary

Continue the backend-first legacy capability migration. Post 8-66 connected product and campaign metric skills to workspace artifacts. Store-level `query_store_metrics` already supports explicit user-provided metrics and natural-language reachability, but successful results still only stay in `skill_result`.

This slice connects successful store metric outputs to the existing workspace artifact framework.

## 2. Backend Scope

No new database table is required.

No new HTTP API is required.

Reuse:

- `WorkspaceApi::materialize_agent_content_artifact`
- `build_content_artifact_from_skill_output`
- existing `workspace_artifact_created` SSE event
- existing artifact list/detail/export APIs

## 3. Artifact Type

Mapping:

```text
query_store_metrics -> data_store_metrics_report
```

The artifact is a markdown wrapper around the successful structured JSON output, consistent with existing content artifact behavior.

## 4. Truthfulness Boundary

Artifacts are created only when:

```text
output.status == "success"
```

If `query_store_metrics` returns:

```text
status=deferred
adapter_status=store_metrics_required
```

then no artifact is created.

This preserves the rule:

- no fake live Taobao / Douyin / other platform data
- no implicit store metrics adapter
- no frontend-only fake report

## 5. API / SSE Documentation

Update root `API_DOCS/`:

- `API_DOCS/app.md`
- `API_DOCS/sse.md`

Document `data_store_metrics_report` and the deferred no-artifact boundary.

## 6. Validation Plan

Run:

```text
cargo fmt --all
cargo test -p service content_artifact_builds_from_store_metrics_skill_output -- --nocapture
cargo test -p service content_artifact_ignores_deferred_store_metrics -- --nocapture
cargo check -p service
cargo clippy -p service
git diff --check
```

## 7. User-Visible Result

When users provide explicit store metrics and trigger `query_store_metrics`, the backend can now persist a reusable store metrics report in the workspace. Missing store metrics remain a clear deferred state.
