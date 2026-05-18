# Post 8-64: Search Skill Natural Reachability

## 1. Requirement Summary

Continue the old-product capability migration with backend-first priority. The existing `search_*` skills already have a unified external-search adapter contract, live/degraded/deferred output shape, and evidence structure. This slice makes those skills reachable from normal user language without adding fake frontend buttons.

## 2. Scope

In scope:

- Add worker autosuggest routing for clear external-search intent.
- Cover these existing skills:
  - `search_trends`
  - `search_competitor`
  - `search_market_info`
  - `search_product_reviews`
  - `search_platform_policy`
  - `search_industry_benchmarks`
- Preserve existing native skills when the user asks for local analysis rather than live search.
- Update role skill permissions so suggested search skills can actually run.
- Update migration logs and SSE documentation.

Out of scope:

- No new database table.
- No new HTTP endpoint.
- No frontend change.
- No Taobao, Douyin, Qianchuan, Xiaohongshu, or official platform adapter implementation.
- No fake search results. If the adapter is not configured or returns no trusted sources, existing deferred/degraded behavior remains.

## 3. Data / Database Design

No schema change.

This slice only maps text intent into existing `search_*` skill requests. Search execution continues to read adapter configuration from the existing agent runtime and `[external_search]` service config boundary.

## 4. API / SSE Contract

No event shape change.

The observable behavior change is that existing SSE may now emit:

```text
skill_requested.skill_id = search_trends | search_competitor | search_market_info | search_product_reviews | search_platform_policy | search_industry_benchmarks
```

The `skill_result` payload remains the existing search contract:

- `status=live` when configured provider returns trusted sources
- `status=deferred` and `adapter_status=adapter_required` when no adapter is configured
- `status=deferred` and `adapter_status=degraded` when provider fails or returns no accepted source

## 5. Auth / User Isolation

No user-owned DB resource is read in this slice. Future provider credentials or user-specific platform search must stay behind service/config boundaries and must not be read directly from web handlers.

## 6. Routing Rules

Search autosuggest should only trigger when the user clearly asks for live/external/current/source-backed information, such as:

- 搜索 / 搜一下 / 联网 / 实时 / 最新 / 外部资料 / 公开来源
- 平台规则 / 政策 / 算法 / 费率 / 违规 / 大促规则
- 用户评价 / 口碑 / 差评 / 真实反馈
- 行业基准 / 行业均值 / benchmark

It must not override these native intents:

- 普通竞品分析 -> `data_competitor_analysis`
- 趋势预测 -> `data_trend_forecast`
- 关键词研究 -> `web_keyword_research`
- 创意趋势捕捉 -> `creative_trend_catch`

## 7. Validation Plan

Run from `backend/`:

```bash
cargo fmt --all
cargo fmt --all --check
cargo test -p agent autosuggests_search -- --nocapture
cargo test -p agent search_skill -- --nocapture
cargo test -p agent keeps_roles_from_requesting_unowned_skills -- --nocapture
cargo check -p agent
cargo clippy -p agent
git diff --check
```
