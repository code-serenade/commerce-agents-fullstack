# Post 8-66: Product / Campaign Metrics Artifact Materialization

## 1. Requirement Summary

Continue the old-product capability migration using the new product backend framework.

The immediately previous slice added native Agent skills:

- `query_product_metrics`
- `query_campaign_metrics`

This slice connects their successful outputs to the existing workspace artifact system so that a user-provided product or campaign metric analysis can become a reusable report in the workspace.

## 2. Backend Scope

No new database table is required.

No new HTTP API is required.

Reuse the existing service boundary:

- `WorkspaceApi::materialize_agent_content_artifact`
- `build_content_artifact_from_skill_output`
- existing `workspace_artifact_created` SSE event
- existing workspace artifact list/detail/export APIs

## 3. Artifact Types

Add two data content artifacts:

- `data_product_metrics_report`
- `data_campaign_metrics_report`

Mapping:

```text
query_product_metrics  -> contentCategory=data, artifactType=data_product_metrics_report
query_campaign_metrics -> contentCategory=data, artifactType=data_campaign_metrics_report
```

## 4. Truthfulness Boundary

Artifacts are created only when:

```text
output.status == "success"
```

The skills already return `deferred` when explicit metrics are missing, so missing product/campaign data still does not produce a fake report.

This preserves the current product-context rule:

- no default product
- no implicit saved product
- no fake Taobao / Douyin / Qianchuan live platform data

## 5. API / SSE Documentation

Update root `API_DOCS/` only:

- `API_DOCS/app.md`
- `API_DOCS/sse.md`

Document that successful product/campaign metric skill outputs can create markdown workspace artifacts through the existing `workspace_artifact_created` event.

## 6. Validation Plan

Run:

```text
cargo fmt --all
cargo test -p service content_artifact_builds_from_product_and_campaign_metrics_skill_output -- --nocapture
cargo test -p service content_artifact_ignores_deferred_product_and_campaign_metrics -- --nocapture
cargo check -p service
cargo clippy -p service
git diff --check
```

If agent-level behavior is touched, also run the relevant `agent` tests. This slice should not need agent code changes.

## 7. User-Visible Result

When the user asks a product/SKU or campaign/activity metric question and provides explicit numbers, the backend can now persist the structured result as a workspace report.

Frontend can later show these reports in the artifact panel, history, and export flows without inventing new fake UI state.
