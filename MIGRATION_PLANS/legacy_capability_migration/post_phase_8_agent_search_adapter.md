# Post 8-10：Agent 搜索 Skill 接入外部搜索 Adapter

状态：done

## 本次目标

- 让 Agent `search_*` skill 能读取新产品 `[external_search]` 配置。
- 配置完整时，搜索 skill 真实请求 provider，并归一化为 `sources`、`data.items`、`provenance`、`confidence`。
- provider 不可用或无可信来源时，继续返回 `deferred/degraded`，不伪造外部搜索结果。
- 保持前端不改，继续消费真实 `skill_result`。

## 涉及范围

- 产品内核：搜索能力从“只返回 adapter_required 合同”升级为“可按配置尝试真实 adapter 执行”。
- 后端：新增 Agent 内部 `SearchAdapterConfig`、搜索 provider client、来源归一化、runtime 配置传递。
- Web-server：启动时把同一份 `[external_search]` 配置同时传给 App API 和 Agent runtime。
- API/SSE：不新增 HTTP 路由；更新文档说明 `search_*` 的 `skill_result` 可为 `success/live` 或 `deferred/degraded`。
- 前端：本阶段不改前端。

## 用户可感知变化

用户在问答中触发趋势搜索、竞品搜索、市场行情、商品评价、平台政策、行业基准等搜索类 skill 时，如果部署环境配置了真实外部搜索 provider，过程区会收到真实来源和置信度；如果 provider 未配置、请求失败或无可采信来源，仍会明确显示降级原因，不会假装已经联网查到资料。

## 后续缺口

- 真实部署环境需要配置 `[external_search]` provider。
- 前端尚未把搜索 `sources/provenance/confidence/degraded_reason` 做成专门来源面板。
- App 外部搜索 API 与 Agent 内部搜索 adapter 目前共享字段语义，但实现仍各自归一化；后续可抽共享 crate 降低重复。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo test -p agent executor_returns_deferred_for_search`
- [x] `cargo test -p agent executor_returns_structured_deferred_for_search_skills`
- [x] `cargo test -p agent executor_reports_degraded_when_configured_search_provider_fails`
- [x] `cargo test -p agent executor_runs_representative_native_categories`
- [x] `cargo test -p agent`
- [x] `cargo check -p agent`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`
