# Phase 1 后端设计：问答策略 metadata 与路由事件

## 目标

把前端回答策略从“拼进用户输入文本”升级为后端可识别的结构化契约，让 agent runtime 能明确知道本次请求采用 `execution`、`learning` 还是 `collab` 策略，并通过 SSE 输出真实路由结果。

## 本次开发范围

- 扩展 Agent SSE request，新增 `metadata.strategy`。
- 扩展 agent `AgentRequest`，让 metadata 进入 `MessageContext`。
- 新增 `route_decided` SSE event，输出：
  - `strategy_id`
  - `response_mode`
  - `collaboration_mode`
  - `route_mode`
  - `primary_runtime_role`
  - `support_runtime_roles`
- `collab` 策略优先走多角色任务图，不再被普通快速单 worker 路由吞掉。
- 前端发送策略 metadata，并消费 `route_decided` 事件。

## 不在本次开发范围

- 不迁移 skill executor 的新能力。
- 不新增数据库表。
- 不新增工作区产物沉淀。
- 不实现质量评分和学习闭环。

## 后端分层

- `backend/crates/agent/`
  - `AgentRequest.metadata`
  - `MessageContext.metadata`
  - `SessionEvent::RouteDecided`
  - session runtime 根据 metadata 决定路由策略
- `backend/crates/web-server/`
  - SSE DTO 接收 `metadata`
  - `run_agent_stream` 传递 metadata
- `API_DOCS/`
  - 更新 `/api/sse/agent/session/stream` 请求体和事件说明

## 前端适配

- `AgentStreamRequest` 增加 `metadata.strategy`。
- 发送请求时不再把策略说明强行拼进用户原始问题，而是通过 metadata 传给后端。
- 保留必要的业务身份提示，但策略差异以后端事件为准。

## 验收

- 后端事件流包含 `route_decided`。
- `execution` / `learning` / `collab` 都能被后端识别。
- `collab` 策略触发任务图路线，SSE 中能看到 `task_graph_planned`。
- `execution` / `learning` 对简单问题仍可走轻流程。
- API 文档同步。
- Rust 与前端基础门禁通过。
