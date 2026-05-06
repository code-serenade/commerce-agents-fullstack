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
  "workspaceId": "workspace-001"
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `input` | `string` | 是 | 用户输入，不能为空，最大 20000 字符 |
| `agentId` | `string` | 否 | agent ID，不传时使用 `web-agent` |
| `sessionId` | `string` | 否 | 会话 ID，不传时服务端生成 |
| `workspaceId` | `string` | 否 | 工作区 ID |

SSE 事件分为两类：

1. 原始 agent 过程事件，事件名来自 `agent::api::SessionEvent::event_name()`。
2. 兼容旧前端的文本事件：`delta` / `done` / `error`。

原始过程事件示例：

```text
event: started
data: {"type":"started","session_id":"session-001"}

event: worker_started
data: {"type":"worker_started","session_id":"session-001","worker_id":"worker-001","task_id":"task-001"}

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
| `commander_thinking` | commander 正在规划或调度 |
| `worker_started` / `worker_finished` | 单个 worker 开始或完成 |
| `collaboration_started` / `collaboration_worker_started` / `collaboration_worker_finished` / `collaboration_finished` | 多 worker 协作过程 |
| `task_graph_planned` / `task_graph_finished` | 任务图规划或完成 |
| `task_node_ready` / `task_node_started` / `task_node_reported` / `task_node_evaluated` / `task_node_rework_requested` / `task_node_passed` / `task_node_failed` | 任务节点生命周期 |
| `output` | agent 产生最终输出 |
| `failed` | agent session 失败 |
| `finished` | agent session 完成 |

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
