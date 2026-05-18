# Post 8-56: service_query_product Native Product Context Skill

## Requirement Summary

Continue the old-product capability migration under the current new-product framework:

- Old product capability source: `D:\github\commerce-agents崩了之后重新做`
- Old skill: `server/src/skills/customer_service.py::ServiceQueryProduct`
- New framework landing point: `backend/crates/agent/src/skills/service.rs`

This slice migrates `service_query_product` from a deferred placeholder to a Rust-native Agent skill that can answer from explicit product context already injected into the Agent runtime.

## Architecture Boundary

This slice does not add a database table, migration, repo API, service API, HTTP route, or new SSE event type.

The `agent` crate must not directly query the product database. Product records and knowledge are already loaded by the existing web-server/service context pipeline and passed into Agent metadata. Therefore the skill reads only `SkillContext.metadata`.

## Product Context Rule

The selected-product and unspecified-product paths must stay separate.

- If the user selected product context for this run, metadata contains product IDs, refs, summaries, or sources. The skill may return `status=success`.
- If no explicit product context is present, the skill returns `status=deferred` and clearly states that product context is required.
- The skill must not inject a default product, infer "千乐咖啡", or fabricate product details.
- Input filters such as `product_id` or `product_name` only narrow already-visible context. They do not trigger an implicit database lookup.

## Metadata Inputs

The skill accepts both runtime-prefixed and direct keys, because worker execution may prefix request metadata as `request.*`:

- `request.context.product_ids`, `context.product_ids`
- `request.context.refs`, `context.refs`
- `request.context.combined_refs`, `context.combined_refs`
- `request.context.selected_summary`, `context.selected_summary`
- `request.context.summary`, `context.summary`
- `request.context.combined_summary`, `context.combined_summary`
- `request.context.sources_json`, `context.sources_json`
- `request.context.source_titles`, `context.source_titles`

## Skill Output Contract

Success output:

- `status=success`
- `has_data=true`
- `adapter_status=context_metadata`
- `data_readiness=explicit_product_context`
- includes query, matched product IDs, refs, sources, product context summary, and an answer policy

Deferred output:

- `status=deferred`
- `has_data=false`
- `adapter_status=context_required`
- `data_readiness=explicit_product_context_required`
- includes `deferred_reason`

## API / SSE Impact

No new endpoint or event type is added. Existing `skill_result` SSE payload carries the skill output. `API_DOCS/sse.md` should mention that `service_query_product` can now succeed from explicit Agent product context, while no-context runs still return `deferred`.

## Validation

Run targeted backend validation:

- [x] `cargo fmt --all`
- [x] `cargo test -p agent service_query_product`
- [x] `cargo test -p agent executor_runs_service_query_product_with_context`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
