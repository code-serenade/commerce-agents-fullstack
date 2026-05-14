# Post 8-60：外部搜索证据契约补强

状态：done

## 本次目标

- 继续按“旧产品能力 = 后端内核能力来源，新产品架构 = 唯一落地框架”迁移搜索能力。
- 不接淘宝、抖音、千川官方系统，不伪造任何实时来源。
- 把旧产品搜索技能里的 `_needs_fresh_data` 实时性语义迁入新产品 Rust Agent search adapter。
- 让 App 外部搜索 API、Agent `search_*` skill 和 `search_evidence` artifact 都能输出统一证据摘要，便于后续 Kimi Agent 风格来源面板消费。

## 本次修改

- 后端 agent：`SearchPlan` 新增 `bypass_cache`，支持 `bypass_cache`、`bypassCache` 和 `_needs_fresh_data`。
- 后端 agent：provider 请求会把 `bypass_cache` 转发给外部搜索 provider。
- 后端 agent：live/degraded/deferred 输出均包含 `evidence`，其中 live 输出会包含来源标题、域名和 LLM 可读证据文本。
- 后端 agent：来源归一化增加 URL/标题去重，重复来源不会进入 `sources`。
- 后端 service：`ExternalSearchResponse` 新增 `maxResults`、`bypassCache` 和 `evidence`。
- 后端 service：App 外部搜索 API 与 Agent search skill 使用同一套证据语义。
- 后端 service：`search_evidence` artifact metadata 增加 `evidence`，方便后续前端不解析完整 content 也能展示来源摘要。
- API/SSE：同步 `API_DOCS/app.md` 和 `API_DOCS/sse.md`。
- 迁移文档：同步迁移矩阵、开发记录、阶段验收记录和总计划。

## 用户可感知变化

- 用户触发趋势、竞品、行情、评价、平台政策、行业基准等搜索类任务时，如果 provider 返回真实来源，后端会输出更清楚的来源证据摘要。
- 如果用户或调度器要求“需要最新数据”，后端会把这个实时性偏好传给 provider，而不是固定走缓存。
- 如果没有 provider 或 provider 没返回可信来源，系统仍会明确显示降级，不会编造搜索结果。
- 后续前端可以把 `evidence.sourceTitles`、`evidence.sourceDomains` 和 `confidence/freshness` 做成右侧来源面板。

## 当前验证

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/external_search_evidence_contract_post_8_60.md`
- [x] API 文档：`API_DOCS/app.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt --all`
- [x] `cargo test -p agent search -- --nocapture`
- [x] `cargo test -p service external_search -- --nocapture`

## 后续缺口

- 真实部署仍需要配置可访问的 `[external_search]` provider。
- 前端仍需把 `evidence`、`sources`、`provenance`、`confidence` 和 `freshness` 整理成 Kimi Agent 风格来源面板。
- 官方平台直接 API 仍暂缓，等内部 skill、问答和通用搜索体验继续稳定后再做。
