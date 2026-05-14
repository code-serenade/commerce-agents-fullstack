# Post 8-56: service_query_product Native Context Skill

## Scope

This slice migrates the old product customer-service product lookup capability into the new product Agent skill framework.

Old product source:

```text
D:\github\commerce-agents崩了之后重新做\server\src\skills\customer_service.py::ServiceQueryProduct
```

New product landing point:

```text
backend/crates/agent/src/skills/service.rs
```

## Backend Changes

- `service_query_product` is now a native deterministic data-source skill instead of a deferred placeholder.
- The skill reads only explicit Agent product context metadata:
  - product IDs
  - product refs
  - selected product summary
  - product context sources
  - source titles
- It supports both old integer product IDs and new string product IDs.
- It returns `success` only when explicit product context exists and optional filters match.
- It returns `deferred` when:
  - no explicit product context exists
  - a requested product ID/name is not present in the current selected context

## Architecture Boundary

The skill does not query the database from the `agent` crate. Product data must be injected by the existing service/web-server context pipeline.

No database migration, repo layer, service API, web-server route, or SSE event type is added in this slice.

## User-Visible Impact

When the user manually selects a product in the frontend, the Agent can use that product context in customer service workflows. If the user does not select a product, the system will not silently use a default product.

This directly protects the selected-product vs unspecified-product behavior:

```text
selected product -> product context skill can answer from that context
no selected product -> normal general answer path; no default product injection
```

## Validation

- [x] `cargo fmt --all`
- [x] `cargo test -p agent service_query_product`
- [x] `cargo test -p agent executor_runs_service_query_product_with_context`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
