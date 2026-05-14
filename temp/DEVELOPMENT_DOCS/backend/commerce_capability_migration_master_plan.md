# Commerce Capability Migration Master Plan

## 1. Document Position

This file is the current authoritative plan for migrating the old product at:

```text
D:\github\commerce-agents崩了之后重新做
```

into the new product at:

```text
D:\github\atlas\commerce-agents-fullstack-main
```

The goal is not to copy old code. The goal is to re-implement old product capability with the new product architecture:

```text
旧产品能力 = 后端内核能力来源
Kimi Agent 集群 = 前端交互和视觉体验参考
新产品架构 = 唯一落地框架
```

## 2. Current Constraints

- `backend/crates/agent/` can now be modified. The old no-agent-crate restriction is no longer active.
- Backend remains the primary migration axis. Frontend only adapts to real backend capability.
- All new or changed API contracts must be documented under root `API_DOCS/`.
- `service` must not access SeaORM or database primitives directly.
- `web-server` must only call `service` or `agent` boundaries, never bypass them.
- LLM clients remain owned by the `agent` crate. No handler-local LLM client creation.
- Frontend must not fake route, skill, approval, search, quality, artifact, or execution state.
- Each completed slice should be:
  - documented
  - validated
  - locally committed
  - not pushed unless the user explicitly asks

## 3. Migration Principles

### 3.1 Backend first

Every capability should move in this order:

1. Backend design clarification
2. Backend implementation
3. `API_DOCS/` sync
4. Rust validation
5. Frontend adaptation
6. Browser verification

### 3.2 Capability is only complete when it is usable

A slice is not complete just because a spec or page exists. It is complete only when:

- the user can trigger it naturally
- backend routes it correctly
- real skill / data / workflow execution occurs or is explicitly degraded
- SSE or API exposes the real status
- final answer or artifact uses real backend outputs

### 3.3 No fake frontend capability

If backend does not yet provide a real route, event, artifact, approval, or adapter result, frontend can only show:

- unavailable
- deferred
- degraded
- pending approval

It must not invent successful execution.

## 4. What Is Already Completed

The following major foundations are already in place:

- Q&A main chain with strategy metadata and real route events
- workspace run / task / artifact persistence
- product context binding and RAG context injection
- search adapter contract and live search path
- quality / trust / learning loop
- approval / audit / alert ingestion for high-risk skills
- commerce playbook entry points
- legacy builtin package governance sync
- campaign draft creation API
- competitor analysis API
- product lifecycle API
- conversation file extraction registration API
- workspace context summary / context pack / artifact export
- conversation file extraction status contract for pending / extracted / failed / unsupported states
- platform execution readiness diagnostics on tool approval records
- platform execution adapter capability catalog on platform connection pages
- platform execution preflight diagnostics before tool approval creation
- shared platform execution rule kernel for preflight, approval readiness, and execution failure details
- shared platform execution action registry for catalog, validation, idempotency fields, and adapter messages
- platform execution adapter request envelope with idempotency key in approval execution failure results
- platform execution adapter outcome contract for provider result and retryability
- platform execution adapter dispatch outcome boundary inside the service framework
- search evidence artifact persistence and reusable evidence summary contract
- agent skill run persistence and query API
- native coordination skill chain planning with structured phases, dependencies, adapter hints, approval hints, and availability flags
- native `query_store_metrics` can now summarize explicit user-provided store metrics, clearly defer when store metrics are missing, and be reached from natural store-metric query language
- search adapter skills can now be reached from natural external-search language while preserving live/degraded/deferred evidence boundaries
- multiple native skill migrations across ops / data / service / creative / web / design / accounting
- creative and design skill outputs can now materialize into reusable workspace artifacts when the skill result is successful
- data dashboard is now a native deterministic skill with explicit manual-input vs store-metrics-required readiness

## 5. Current Progress Estimate

This is the aligned progress baseline for ongoing work:

- backend core migration: `70% - 75%`
- frontend adaptation and workbench organization: `45% - 55%`
- overall product target: about `60%`

Interpretation:

- backend capability is ahead of frontend
- the product already has real backend foundations
- the frontend still does not fully organize those capabilities into a clean “电商虚拟员工工作台”

## 6. What Still Remains

### 6.1 Backend capability gaps

These remain the highest-value unfinished backend areas:

1. more old-product skill ecosystem migration and reachability
2. external search / real-time data productization
3. platform execution closed loop after approval
4. attachment / material / OCR / document parsing
5. richer coordination and status query capability

### 6.2 Frontend product gaps

These remain the highest-value unfinished frontend areas after backend slices are stable:

1. Kimi-style left / center / right workbench organization
2. better process visibility for roles / skills / evidence / approvals / quality / artifacts
3. reliable left/right collapsible interaction
4. stronger history / workspace / product / campaign -> chat linking

## 7. Phase Plan

### Phase A: Stabilize Q&A main chain

Status: mostly done, still under continuous hardening.

Keep verifying:

- model selection -> real backend model
- strategy -> real backend routing
- role selection -> real task graph
- context -> real backend injection
- skill execution -> real SSE and persistence
- final answer -> real workspace artifact

### Phase B: Continue old-product skill migration

Status: in progress.

Priority in this phase:

- make already migrated native skills easier to reach from real user phrasing
- continue migrating remaining old-product skill coverage
- prefer high-value e-commerce use cases over long-tail utilities

Focus order:

1. 商品分析 / 选品分析 / 运营计划 / 投放复盘 related native coverage
2. richer ops and data trigger coverage
3. service and engineering support coverage
4. coordination capability realness

### Phase C: External search and real-time data

Status: contract exists, productization incomplete.

Need:

- configured provider use in real environments
- better result reuse
- clearer evidence / degradation semantics
- stronger frontend source panel later

### Phase D: Platform execution closed loop

Status: approval creation, readiness, adapter catalog, preflight, shared execution rules, shared action registry, adapter request envelope, adapter outcome contract, and service-internal dispatch outcome boundary exist; actual execution adapter loop incomplete.

Need:

1. agent proposes execution
2. approval persists
3. approved action invokes adapter
4. result / failure / audit persists
5. frontend later displays the real outcome

### Phase E: Attachment / material / export / teaching

Status: summary and export foundations exist, parsing still incomplete.

Need:

- OCR
- PDF / Office / image text extraction
- richer file ingestion
- real reusable artifact / source pipeline

### Phase F: Frontend workbench reorganization

Status: deferred behind backend progress.

Target layout:

- center: main Q&A and final output
- left: chat history / workspaces / task history
- right: roles / process / RAG / skills / approvals / quality / artifacts
- top or input bar: model / strategy / roles / context
- output follow-up area: artifacts / export / feedback

## 8. Next Development Order

Continue in this order:

1. backend skill reachability and remaining old-product native skill migration
2. external search / real-time data productization
3. attachment / parsing / richer context ingestion
4. frontend adaptation to real backend slices
5. Kimi-style workbench reorganization
6. platform execution closure after official platform adapter decisions
7. full browser-driven product verification

## 9. Current Slice Selection

The latest active/completed slice is:

```text
Post 8-87: data trend forecast product_ref contract
```

Scope:

- add `product_ref=product:{uuid}` to `data_trend_forecast`
- echo the selected product reference in successful trend forecast output
- route generic trend / forecast phrasing to structured `data_trend_forecast` input
- parse explicit historical numeric series from the current text into `values`
- let data worker autosuggest pass selected product refs only from explicit `context.selected_summary`
- keep insufficient history as deferred rather than fake live platform history, industry cycle, or campaign-impact data

Reason:

- generic trend forecast is the most common fallback for user questions about GMV, order, UV, and conversion outlook
- selected-product and unspecified-product trend prediction must stay separate
- frontend artifacts and process panels need a stable product reference for product-bound forecast reports
- real platform sales history, industry cycles, campaign calendars, and activity impact remain adapter/service/search concerns and must not be faked

## 10. Validation Standard

Backend minimum for each slice:

- `cargo fmt`
- `cargo check`
- `cargo clippy`
- relevant crate tests

If frontend is touched for that slice:

- `pnpm -C frontend/apps/web rules`
- `pnpm -C frontend/apps/web lint`
- `pnpm -C frontend/apps/web test`
- `pnpm -C frontend/apps/web build`
- `pnpm -C frontend rules`

## 11. Tracking Files

This plan must stay aligned with:

- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `MIGRATION_PLANS/legacy_capability_migration/frontend_experience_checklist.md`
- `MIGRATION_PLANS/legacy_capability_migration/api_migration_checklist.md`

If one of those files conflicts with this plan, update the older file rather than splitting the truth.
