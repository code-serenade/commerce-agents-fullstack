# Post 8-8：外部搜索 Adapter 契约化

状态：done

## 本次目标

- 把旧产品的趋势、竞品、市场行情、评价、平台政策、行业基准搜索能力，迁成新产品 Agent runtime 中稳定的后端契约。
- 当前不伪造实时搜索，不写死 provider，不新增假来源。
- 在真实 search adapter 未接入前，所有搜索 skill 返回统一 `adapter_required` 降级结构，后续前端和真实 provider 可按该结构继续对接。

## 涉及范围

- 产品内核：搜索 skill 从简单 deferred 占位升级为统一 search adapter fallback 契约。
- 后端：`backend/crates/agent/src/skills/search.rs`、`backend/crates/agent/src/skills/runtime.rs`。
- API：本轮不新增 HTTP API，不修改 SSE 事件结构，不需要更新 `API_DOCS/`。
- 前端：本轮不改前端；后续真实 search provider/API 接入后再展示来源、置信度和降级原因。

## 产品内核/后端变化

- `search_trends`
- `search_competitor`
- `search_market_info`
- `search_product_reviews`
- `search_platform_policy`
- `search_industry_benchmarks`

以上六个 skill 现在都会返回统一结构：

```json
{
  "status": "deferred",
  "adapter_status": "adapter_required",
  "adapter": {
    "kind": "search",
    "provider": null,
    "configured": false
  },
  "query": {
    "raw": "...",
    "planned": "...",
    "topic": "general/news/finance",
    "days": 30,
    "max_results": 6
  },
  "sources": [],
  "provenance": {
    "source_count": 0
  },
  "confidence": 0.0,
  "degraded": true
}
```

## 用户可感知变化

用户问需要实时趋势、竞品搜索、平台政策、行业基准的问题时，后端会明确识别为搜索能力，但不会假装已经联网查到数据。过程区仍会显示 `deferred`，后端输出中会保留计划搜索词、时间范围和降级原因，方便后续接入真实搜索 adapter 后继续展示真实来源。

## 后续缺口

- Rust 外部搜索 API 已在 Post 8-9 建立；Agent `search_*` skill 尚未直接调用该 API。
- 前端尚未展示 search output 中的 `query/sources/provenance/confidence/degraded_reason` 细节。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo test -p agent executor_returns_deferred_for_search`
- [x] `cargo test -p agent executor_returns_structured_deferred_for_search_skills`
- [x] `cargo test -p agent`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`
