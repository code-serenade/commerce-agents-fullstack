# Post 8-55 Backend Design: External Search Priority-2 RAG/SSE Availability Contract

## Goal

The user raised external search / real-time information enhancement from a later priority to priority 2.

This slice keeps the implementation backend-first and does not connect official Taobao, Douyin, Qianchuan, or other platform write APIs. It tightens the existing Agent RAG/SSE contract so the frontend and Agent runtime can tell whether a real external search adapter is configured.

## Scope

- No new database tables.
- No migration.
- No frontend changes in this slice.
- No fake external search results.
- No official platform direct integration.
- Keep product context explicit: only user-selected products can be injected into the Agent answer context.

## Current Problem

`rag_context_loaded.externalSearchAvailable` is always `false` because `AgentContextApi::build_retrieval_context` does not know the web-server `[external_search]` configuration.

That makes the frontend unable to distinguish:

1. local product knowledge did not match, and no external search adapter exists
2. local product knowledge did not match, but search skills can request a configured live provider

## Design

The boundary stays layered:

- `web-server` owns environment configuration access through `get_external_search_config()`.
- `service` owns RAG response shaping and degraded reason wording.
- `agent` owns LLM clients and search skill execution.

Implementation:

1. Add `external_search_available: bool` to `BuildAgentRetrievalContextRequest`.
2. In Agent SSE, read `get_external_search_config().is_configured()`.
3. Pass that boolean into `build_retrieval_context`.
4. In `AgentRetrievalContextResponse`, set `external_search_available` from the request.
5. When local RAG has no sources:
   - if false: keep the existing unavailable degraded reason
   - if true: return a configured degraded reason that says search skills may request live external sources
6. Update the metadata warning so the model does not cite external facts unless a same-run search skill returns live sources.

## User-Visible Meaning

When no product knowledge is found:

- If external search is not configured, the UI can say the system has no local source and no live search adapter.
- If external search is configured, the UI can say local knowledge did not match but live search capability exists for search skills.

The system still will not invent market facts. A live source must come from a successful `search_*` skill result.

## Validation

- `cargo fmt`
- targeted service tests for degraded reason semantics
- targeted web-server SSE metadata tests
- `cargo check -p service -p web-server`
- `cargo clippy -p service -p web-server`
- `git diff --check`
