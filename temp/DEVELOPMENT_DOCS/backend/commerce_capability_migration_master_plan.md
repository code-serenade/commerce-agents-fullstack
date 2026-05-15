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

## Post 8-109 Update

- App external search API and Agent `search_*` skills now accept more common provider source shapes, including `data.results`, `data.sources`, `organic_results`, `webPages.value`, and top-level `value`.
- Source fields now accept common aliases such as `headline`, `href`, `description`, `text`, `excerpt`, and `datePublished`.
- Live search evidence now requires a traceable URL with a domain derived from that URL. Title-only, snippet-only, or relative-path weak results are filtered.
- If filtering leaves no reliable source, search still degrades instead of fabricating live evidence or creating a fake `search_evidence` artifact.

## Post 8-110 Update

- Agent Chat upload now uses the existing conversation file extraction API after file registration.
- Browser-readable text files are registered as `extracted`; empty text is registered as `failed`; non-text files are registered as `unsupported`.
- Upload success and extraction readability are now separated, so a file can be saved without pretending that PDF, image, Office, or binary content was parsed.
- Workspace context pack continues to read only trusted `extracted` text.

## Post 8-111 Update

- App external search API and Agent `search_*` skills now enforce the request/plan-level `maxResults` / `query.max_results` after provider response normalization.
- `sources`, `data.items`, `provenance`, `evidence`, and downstream search evidence artifacts are built only from the truncated trusted source set.
- The provider may return more raw items, but the new product backend exposes only the configured strategy limit, still capped by the global hard limit of 12.
- Degraded behavior is unchanged: missing provider, provider failure, or zero trusted traceable sources still never creates fake live evidence.

## Post 8-112 Update

- All six Agent search skills now declare optional `max_results` input.
- Runtime accepts both `max_results` and `maxResults`, giving explicit skill input priority over `[external_search].default_max_results`.
- Search plan result counts are clamped to `1..=12`, then reused consistently by provider requests, live source normalization, provenance, evidence, and search evidence artifacts.
- This moves old product search strategy controls further into the new product's Agent skill contract without adding frontend-only controls.

## Post 8-113 Update

- Worker autosuggest now extracts search result counts from natural language such as `最多3条来源`, `返回5条结果`, and `top 4 sources`.
- Inferred counts are written to `skill_requested.input.max_results` for all six `search_*` skills.
- Result-count inference is bounded to source/result wording and avoids treating period expressions such as `最近30天` as result limits.
- This makes old product search strategy controls reachable from user language before frontend-specific controls are added.

## Post 8-114 Update

- App external search API and Agent `search_*` skills now filter sources with parseable `published_at` / `publishedAt` / `datePublished` values outside the requested freshness window.
- Unknown or unparsable source dates are kept but do not add freshness confidence.
- `provenance.staleFilteredCount` on App API and `provenance.stale_filtered_count` / `freshness.stale_filtered_count` on Agent output expose how many stale sources were removed.
- If every traceable source is stale, search degrades instead of creating fake live evidence or a `search_evidence` artifact.

## Post 8-115 Update

- Service-role DSR / shop score / service score / NPS / satisfaction-driver prompts can now surface a visible two-step skill chain when the user asks for latest, realtime, online, or industry benchmark data.
- The first step remains a native service skill such as `service_dsr_improvement` or `service_nps_driver_analysis`; the second step is `search_industry_benchmarks`.
- `service` is allowed to use `search_industry_benchmarks` through the role skill catalog, so the chain is not rejected after autosuggest.
- Industry benchmark search now supports `metric=dsr` in addition to existing NPS and KPI metrics.
- Selected product context is still explicit-only: no product ref/name/category is injected unless the current turn selected a product.

## Post 8-116 Update

- Data-role competitor analysis and price-elasticity prompts can now surface visible two-step chains when the user explicitly asks for realtime/latest/online public market data.
- `data_competitor_analysis` can be followed by `search_competitor`; `data_price_elasticity` can be followed by `search_competitor` with `focus=price`.
- Native-only prompts still remain native-only, so the product does not imply realtime competitor intelligence unless the user asked for it and the search adapter produced real sources.
- `infer_search_max_results` now handles multi-digit limits before non-ASCII punctuation without byte-boundary panics.
- Selected product context remains explicit-only across both native data skill input and follow-up competitor search input.

## Post 8-117 Update

- Creative-role video script, seeding copy, content calendar, and trend-catch prompts can now surface visible two-step chains when the user explicitly asks for latest, realtime, online, hot-topic, viral-style, or marketing-node data.
- `creative_video_script`, `creative_seeding_copy`, and `creative_trend_catch` can be followed by `search_trends`; `creative_content_calendar` can be followed by `search_market_info`.
- The new product no longer hides old-product creative `_web_search` behavior inside native creative outputs; the search step is visible through the existing `skill_requested` / `skill_result` SSE contract.
- `creative` is allowed to use `search_market_info`, so content-calendar marketing-node search is not rejected by role permissions.
- Result-count language such as `最多4条来源` is kept as search `max_results` and no longer changes video-script duration.
- Selected product context remains explicit-only: no product ref/name/category is injected unless the current turn selected a product.

## Post 8-118 Update

- Web-role SEO optimization, keyword research, and page conversion prompts can now surface visible two-step chains when the user explicitly asks for latest, realtime, online, SEO algorithm, traffic-rule, keyword-competition, search-volume, CPC, or conversion-benchmark data.
- `web_seo_optimize` can be followed by `search_platform_policy(policy_type=algorithm)` for platform algorithm / traffic-rule requests or by `search_market_info` for keyword-competition / search-volume requests.
- `web_keyword_research` can be followed by `search_market_info` when the user asks for realtime keyword competition, search volume, CPC, or heat.
- `web_page_conversion` can be followed by `search_industry_benchmarks(metric=conversion)` when the user asks for industry conversion-rate benchmarks.
- The new product no longer hides old-product Web `_web_search` behavior inside native Web outputs; the search step is visible through the existing `skill_requested` / `skill_result` SSE contract.
- Selected product context remains explicit-only: no product ref/name/category is injected unless the current turn selected a product.

## Post 8-108 Update

- Agent Chat frontend now exposes explicit current-conversation file sources after upload.
- Manual source selection sends `conversation_file:{fileId}` through `metadata.context.contextRefs`, matching the Post 8-107 backend contract.
- Source refs are context-only and do not reverse-select products.
- No source is injected when the user has not selected it, preserving the separate paths for selected product, selected source, and ordinary general Q&A.

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
- search evidence artifacts now preserve explicit selected-product context in metadata and preview when live `search_*` results are product-bound
- search evidence artifacts now enter `workspace_context_pack` as readable evidence summaries instead of raw JSON
- explicit `contextRefs` can now reuse selected conversation files, product assets, product materials, and workspace artifacts as Agent context without automatically pulling unrelated history
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
Post 8-121: accounting finance search chain
```

Scope:

- make accounting autosuggest produce visible native + `search_market_info(topic=finance)` chains for finance benchmark questions
- keep ordinary accounting calculations and diagnoses on native skills only when no external finance benchmark is requested
- keep pure finance benchmark search as one search skill, not a fake budget plan or fake diagnosis
- preserve explicit selected-product context only; never infer product context from history or defaults
- keep search provider failures as transparent degraded search results rather than fabricated finance benchmark data

Reason:

- financial health analysis needs industry margin, net margin, and ad ROI benchmarks when the user explicitly asks for finance comparison
- old-product hidden accounting web search should become visible and auditable in new-product SSE skill events
- natural-language limits such as "max 4 sources" must map to search parameters while default finance benchmark lookback remains 60 days
- real financial data providers, Taobao, Douyin, Qianchuan, policy announcements, market data, and live external sources remain adapter/search concerns and must not be faked

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
