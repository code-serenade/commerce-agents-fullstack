# SSE API

接口前缀：`/api/sse`

说明：SSE 用于把 agent 回复以文本流形式返回给前端；普通业务通知仍然使用 WebSocket `/api/ws`。

## 1. Agent 回复流

- 方法：`POST`
- 路径：`/api/sse/agent/session/stream`
- 鉴权：有效 App 用户 JWT
- 响应类型：`text/event-stream`

请求体：

```json
{
  "input": "请用一句话解释 agent session 是什么",
  "agentId": "web-agent",
  "sessionId": "session-001",
  "workspaceId": "workspace-001",
  "metadata": {
    "context": {
      "productIds": ["product-uuid"],
      "contextRefs": ["product:product-uuid"]
    },
    "strategy": {
      "id": "collab",
      "responseMode": "execution",
      "collaborationMode": "manual"
    }
  }
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `input` | `string` | 是 | 用户输入，不能为空，最大 20000 字符 |
| `agentId` | `string` | 否 | agent ID，不传时使用 `web-agent` |
| `sessionId` | `string` | 否 | 会话 ID，不传时服务端生成 |
| `workspaceId` | `string` | 否 | 工作区 ID |
| `metadata.context.productIds` | `string[]` | 否 | 绑定到本轮问答的商品 ID，最多 5 个；后端会校验当前用户权限并读取真实商品/知识 |
| `metadata.context.contextRefs` | `string[]` | 否 | 上下文引用，当前支持 `product:{productId}`，最多 20 个 |
| `metadata.strategy.id` | `string` | 否 | 回答策略 ID，当前前端使用 `execution` / `learning` / `collab` |
| `metadata.strategy.responseMode` | `string` | 否 | 回答模式，当前前端使用 `execution` / `learning` |
| `metadata.strategy.collaborationMode` | `string` | 否 | 协作模式，当前前端使用 `single` / `manual`；`manual` 会要求后端优先进入多角色任务图 |

`metadata.context.*` 只作为引用入口，不允许前端直接塞商品事实。后端会通过 service 层读取当前用户自己的 active 商品和 active 商品知识，压缩成 agent runtime metadata：`context.product_ids`、`context.refs`、`context.summary`、`context.sources_json` 等。商品不存在、已归档、不属于当前用户或 ID 非法时返回商品相关错误码。

`metadata.strategy.*` 仅允许小写字母、数字、`_`、`-`，单字段最大 64 字符。后端会把这些字段转换为 agent runtime metadata：`strategy.id`、`strategy.response_mode`、`strategy.collaboration_mode`。前端不得把策略说明拼进 `input` 来伪造后端策略行为。

SSE 事件分为两类：

1. 原始 agent 过程事件，事件名来自 `agent::api::SessionEvent::event_name()`。
2. 兼容旧前端的文本事件：`delta` / `done` / `error`。

原始过程事件示例：

```text
event: started
data: {"type":"started","session_id":"session-001"}

event: route_decided
data: {"type":"route_decided","session_id":"session-001","strategy_id":"collab","response_mode":"execution","collaboration_mode":"manual","route_mode":"task_graph","primary_runtime_role":"data","support_runtime_roles":["accounting","creative","ops"]}

event: context_loaded
data: {"type":"context_loaded","session_id":"session-001","product_ids":["product-uuid"],"refs":["product:product-uuid"],"product_count":1,"knowledge_count":3,"source_count":4,"summary":"Product source product:...","sources_json":"[...]"}

event: worker_started
data: {"type":"worker_started","session_id":"session-001","worker_id":"worker-001","task_id":"task-001"}

event: skill_requested
data: {"type":"skill_requested","session_id":"session-001","worker_id":"worker.chat","task_id":"task-001","request_id":"task-001-skill-1","skill_id":"coordination_agent_handoff","role":"chat","reason":"需要交接给更合适的虚拟员工"}

event: skill_result
data: {"type":"skill_result","session_id":"session-001","worker_id":"worker.chat","task_id":"task-001","request_id":"task-001-skill-1","skill_id":"coordination_agent_handoff","role":"chat","status":"success","summary":"Agent交接建议已生成","error":null,"execution_mode":"native","risk_level":"low","approval_required":false}

event: task_graph_planned
data: {"type":"task_graph_planned","session_id":"session-001","graph":{...}}

event: task_node_reported
data: {"type":"task_node_reported","session_id":"session-001","graph_id":"graph-001","node_id":"node-001","worker_id":"worker-001","report":{...}}

event: finished
data: {"type":"finished","session_id":"session-001"}
```

常见原始事件：

| 事件名 | 说明 |
| --- | --- |
| `started` | agent session 已开始 |
| `memory_loaded` / `memory_persisted` / `memory_compacted` | 记忆加载、保存或压缩 |
| `context_loaded` | 后端已读取并注入真实业务上下文，当前包含商品 ID、context refs、商品数量、知识数量、来源数量、摘要和来源 JSON |
| `commander_thinking` | commander 正在规划或调度 |
| `route_decided` | 后端已识别策略和路由模式，包含策略、主角色和支持角色 |
| `worker_started` / `worker_finished` | 单个 worker 开始或完成 |
| `skill_requested` | worker 已按模型结构化输出请求调用某个 skill，包含请求 ID、skill ID、角色和原因 |
| `skill_result` | skill 已返回执行结果，包含状态、摘要、错误、执行模式、风险等级和是否需要审批 |
| `collaboration_started` / `collaboration_worker_started` / `collaboration_worker_finished` / `collaboration_finished` | 多 worker 协作过程 |
| `task_graph_planned` / `task_graph_finished` | 任务图规划或完成 |
| `task_node_ready` / `task_node_started` / `task_node_reported` / `task_node_evaluated` / `task_node_rework_requested` / `task_node_passed` / `task_node_failed` | 任务节点生命周期 |
| `output` | agent 产生最终输出 |
| `failed` | agent session 失败 |
| `finished` | agent session 完成 |

`skill_result.status` 当前使用：`success`、`deferred`、`pending_approval`、`not_found`、`validation_error`、`failed`。其中 `pending_approval` 和 `approval_required=true` 表示后端没有直接执行平台写操作，需要进入审批或人工确认链路；`deferred` 表示 skill 已被识别但依赖搜索、外部数据、数据库或平台适配器，当前不会伪造结果。

兼容文本事件：

```text
event: delta
data: {"content":"Agent session 是..."}

event: done
data: {}
```

失败事件：

```text
event: error
data: {"message":"agent failed"}
```

前端消费规则：

- 新前端应监听所有原始过程事件，用于展示 agent、worker、任务图和任务节点过程
- 文本回复仍然监听 `delta`，把 `data.content` 追加到当前回复文本
- 收到 `delta` 后把 `data.content` 追加到当前回复文本
- 收到 `done` 后结束本次回复
- 收到 `error` 后展示错误信息并结束本次回复
- 前端使用 `POST + fetch stream` 调用，不使用 `GET EventSource`

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 当前 agent runtime 如果只产生最终 `Output`，服务端会把最终内容作为一个 `delta` 发出
- 后续 agent 支持 token 级 `LlmChunk` 后，服务端会把每个 chunk 映射为 `delta`
