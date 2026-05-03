# SSE API

接口前缀：`/api/sse`

说明：本文件只记录服务端使用 SSE 向前端返回的流式接口。当前 SSE 用于 agent session 事件流；普通业务通知仍然使用 WebSocket `/api/ws`。

## 1. Agent Session 流

- 方法：`POST`
- 路径：`/api/sse/agent/session/stream`
- 鉴权：有效 App 用户 JWT
- 响应类型：`text/event-stream`

请求体：

```json
{
  "input": "基于数据诊断做详情页设计和预算风险评估，并给运营调整方案",
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

SSE 事件：

```text
event: started
data: {"type":"started","session_id":{"0":"session-001"}}

event: task_graph_planned
data: {"type":"task_graph_planned","session_id":{"0":"session-001"},"graph":{...}}

event: worker_started
data: {"type":"worker_started","session_id":{"0":"session-001"},"worker_id":{"0":"worker.data"},"task_id":{"0":"task-001"}}

event: output
data: {"type":"output","session_id":{"0":"session-001"},"content":"..."}

event: finished
data: {"type":"finished","session_id":{"0":"session-001"}}
```

事件说明：

| 事件 | 说明 |
| --- | --- |
| `started` | agent session 已启动 |
| `memory_loaded` / `memory_persisted` / `memory_compacted` | 记忆加载、写入或压缩事件 |
| `commander_thinking` | commander 正在规划或路由 |
| `task_graph_planned` | 已规划任务图 |
| `task_node_ready` / `task_node_started` / `task_node_reported` / `task_node_evaluated` / `task_node_passed` / `task_node_failed` | 任务节点生命周期事件 |
| `worker_started` / `worker_finished` | worker 生命周期事件 |
| `collaboration_started` / `collaboration_worker_started` / `collaboration_worker_finished` / `collaboration_finished` | 多角色协作事件 |
| `llm_chunk` | LLM token 增量；当前仅在 agent runtime 支持 token streaming 时出现 |
| `output` | agent 最终或阶段性输出 |
| `failed` | agent session 失败 |
| `finished` | agent session 结束 |

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 服务端直接转发 agent `SessionEvent`，`event` 名称来自 `SessionEvent::event_name()`
- 前端使用 `POST + fetch stream` 调用，不使用 `GET EventSource`
