# Post 8-55：外部搜索提升为第 2 优先级后的 RAG/SSE 可用性契约

状态：done

## 本次目标

- 按用户最新要求，把“外部搜索 / 实时信息增强”从后续第 5 优先级提升到第 2 优先级。
- 先补后端主链路契约，而不是先做前端假入口。
- 让 Agent SSE 的 `rag_context_loaded.externalSearchAvailable` 真实反映后端 `[external_search]` 是否完整配置。
- 不接淘宝、抖音、千川等官方平台直接 API；这些平台写/读集成仍按后续 adapter 决策处理。

## 涉及范围

- 后端 service：`BuildAgentRetrievalContextRequest` 增加 `external_search_available`，RAG 降级原因根据该状态区分。
- 后端 web-server：SSE 入口读取 `get_external_search_config().is_configured()` 并传给 `AgentContextApi`。
- Agent runtime metadata：`rag.external_search_available` 现在跟随真实配置，不再固定为 `false`。
- API 文档：`API_DOCS/sse.md` 说明本地 RAG 未命中时，外部搜索已配置和未配置是两种不同降级语义。
- 前端：本轮不改前端，不新增假来源面板。

## 产品内核变化

外部搜索现在作为第 2 优先级继续推进，但仍保持“真实来源才展示”的原则：

- 本地商品知识命中时，回答仍优先使用当前用户显式选择的商品和知识。
- 本地商品知识未命中且外部搜索未配置时，返回“adapter is not configured”。
- 本地商品知识未命中但外部搜索已配置时，返回“search skills may request live sources”。
- 即使外部搜索已配置，也不会凭空把外部事实注入回答；必须由同一轮 `search_*` skill 成功返回 `live` 来源后，前端和 Agent 才能引用。

## 用户可感知变化

用户发起需要实时市场、竞品、趋势或政策信息的问题时，系统现在能更准确地区分：

- 当前只是本地知识没命中，也没有外部搜索能力；
- 当前本地知识没命中，但后端已经具备外部搜索 adapter，可以由搜索类 skill 继续请求实时来源。

前端后续做 Kimi Agent 风格来源面板时，可以直接消费这个状态，不需要猜测外部搜索是否可用。

## 后续缺口

- 还需要在前端右侧过程/来源区把 `externalSearchAvailable`、`search_evidence` artifact、`sources/provenance/confidence` 做成清楚的来源面板。
- 真实部署仍需要配置可访问的 `[external_search]` provider。
- 官方平台直接 API 集成仍暂缓，等系统内部问答、skill 和外部搜索能力稳定后再做。

## 当前验证

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/agent_external_search_priority_post_8_55.md`
- [x] 迁移总计划已把外部搜索提升为第 2 优先级。
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt`
- [x] `cargo test -p service agent_context`
- [x] `cargo test -p web-server retrieval_metadata`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`
- [x] `git diff --check`
