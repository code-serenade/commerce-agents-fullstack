# Post 8-134 Backend Design: Douyin Ads Native Skills

## 1. Business Understanding

The old product contains Douyin / Qianchuan operating knowledge around campaign race logic, cold-start plans, audience packages, budget pacing, ROI / ROAS stop-loss rules, and live-room funnel diagnosis. The new product already has a `douyin_ads` role, prompt, and task graph node, but the role still lacks real native skill execution.

This slice turns the `douyin_ads` role into a real backend capability inside the existing Rust Agent framework. It does not copy old Python code and does not build a second skill framework.

## 2. Scope

Add native deterministic skills in `backend/crates/agent`:

- `douyin_ads_plan_structure`
- `douyin_ads_audience_package`
- `douyin_ads_budget_pacing`
- `douyin_ads_live_funnel_diagnosis`

Update:

- skill registry and runtime dispatch
- role skill whitelist for `douyin_ads`
- worker autosuggest rules for natural Qianchuan / Douyin ads prompts
- workspace artifact materialization for successful Douyin ads skill output
- root `API_DOCS/` and migration tracking docs

## 3. Database Design

No database migration is required.

The new skills use existing runtime and persistence paths:

- `skill_requested` / `skill_result` SSE events
- existing agent skill run persistence
- existing workspace artifact persistence for supported successful skill output

## 4. API Design

No new HTTP API is required.

The frontend will consume the existing Agent SSE contract:

```text
POST /api/sse/agent/session/stream
event: skill_requested
event: skill_result
event: workspace_artifact_created
```

The new skill IDs appear in the existing `skill_id` field.

## 5. Auth Design

No auth change is required.

User isolation remains handled by the existing Agent SSE / workspace run / artifact persistence path. The skill itself does not read database records directly and does not bypass current user context.

## 6. Data Boundary

These skills are native deterministic analysis skills. They must not pretend to have connected to Douyin, Qianchuan, Douyin E-commerce Compass, CRM, DMP, data warehouse, or any official platform system.

Allowed inputs:

- current turn user text parsed by worker autosuggest
- explicit `product_ref=product:{uuid}` from the existing selected product context
- direct skill input from LLM `skill_requests`

If key metrics are missing:

- planning and audience skills can use clearly marked deterministic defaults
- budget pacing and live funnel diagnosis return `status=deferred` when required metrics are absent
- output must include `adapter_status`, `data_readiness`, and a clear explanation

## 7. Skill Contracts

### `douyin_ads_plan_structure`

Purpose: produce a Qianchuan campaign / plan structure with cold-start, test, scale, and stabilize phases.

Inputs:

- `goal`
- `budget`
- `platform`
- `product_name`
- `product_ref`
- `stage`
- `days`
- `target_roas`

Output highlights:

- plan groups
- budget split
- KPI and stop-loss rules
- user-provided vs default readiness

### `douyin_ads_audience_package`

Purpose: produce audience package matrix, exclusions, retargeting, and testing order.

Inputs:

- `target_audience`
- `product_name`
- `category`
- `budget`
- `product_ref`
- `stage`

Output highlights:

- audience packages
- test order
- exclusions
- retargeting pool
- risks

### `douyin_ads_budget_pacing`

Purpose: calculate daily pacing, current ROAS, budget remaining, and scale / hold / stop action.

Inputs:

- `budget`
- `days`
- `spend`
- `gmv`
- `target_roas`
- `current_roas`
- `product_ref`

Required data boundary:

- requires `budget`
- requires either `current_roas`, or both `spend` and `gmv`

### `douyin_ads_live_funnel_diagnosis`

Purpose: diagnose live-room ad funnel conversion from exposure to payment.

Inputs:

- `impressions`
- `clicks`
- `live_room_entries`
- `product_clicks`
- `orders`
- `payments`
- `ad_spend`
- `gmv`
- `product_ref`

Required data boundary:

- requires at least `impressions` and `clicks`
- richer bottleneck diagnosis depends on deeper funnel metrics

## 8. Worker Autosuggest

When `runtime_role=douyin_ads`, natural prompts should map to:

- plan / structure / cold-start / campaign: `douyin_ads_plan_structure`
- audience / targeting / people package: `douyin_ads_audience_package`
- budget / spend / pacing / scale / stop-loss / ROI / ROAS: `douyin_ads_budget_pacing`
- live room / exposure / click / product click / order / payment / funnel: `douyin_ads_live_funnel_diagnosis`

If the user explicitly asks for latest, realtime, online, public source, platform rules, or industry benchmark, autosuggest may add the existing search skills as a second visible step. The search step must remain separate and degraded if no real provider is configured.

## 9. Frontend Impact

No frontend change in this slice.

Later frontend alignment should display these skills through the existing process cards, right-side process panel, and artifact list instead of creating fake buttons.

## 10. Validation Plan

Run:

```text
cargo fmt --all
cargo test -p agent douyin_ads
cargo test -p agent keeps_roles_from_requesting_unowned_skills
cargo test -p service content_artifact_builds_from_douyin_ads_skill_output
cargo check -p agent -p service
cargo clippy -p agent -p service
```

If workspace-level validation is too slow or blocked by unrelated files, record the exact command and failure.
