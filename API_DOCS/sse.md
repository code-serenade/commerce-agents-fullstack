# SSE API

接口前缀：`/api/sse`

说明：SSE 用于把 agent 回复以文本流形式返回给前端；普通业务通知仍然使用 WebSocket `/api/ws`。

LLM 配置查询接口不走 SSE，可用 `GET /api/app/agent/llm-config` 查看当前选中模型和可选模型。该接口需要登录，但不会返回 `api_key` 明文。

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
  "modelName": "ollama-gemma4",
  "sessionId": "session-001",
  "workspaceId": "workspace-001"
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `input` | `string` | 是 | 用户输入，不能为空，最大 20000 字符 |
| `agentId` | `string` | 否 | agent ID，不传时使用 `web-agent` |
| `modelName` | `string` | 否 | 本次调用使用的 LLM 配置名，对应 `services.toml` 的 `[[llm]].name` |
| `sessionId` | `string` | 否 | 会话 ID，不传时服务端生成 |
| `workspaceId` | `string` | 否 | 工作区 ID |

SSE 事件：

```text
event: delta
data: {"content":"Agent session 是..."}

event: agent_event
data: {"type":"task_graph_planned","graph":{"graph_id":"graph-001","root_task":"...","nodes":[]}}

event: done
data: {}
```

失败事件：

```text
event: error
data: {"message":"agent failed"}
```

前端消费规则：

- 需要监听 `delta` / `agent_event` / `done` / `error`
- 收到 `delta` 后把 `data.content` 追加到当前回复文本
- 收到 `agent_event` 后更新 Agent 集群调度 UI；该事件不追加到回复文本
- 收到 `done` 后结束本次回复
- 收到 `error` 后展示错误信息并结束本次回复
- 前端使用 `POST + fetch stream` 调用，不使用 `GET EventSource`

`agent_event` 事件用于把 agent runtime 中已经存在的任务图和 worker 调度事件透出给前端。常见 `type`：

| type | 说明 | 关键字段 |
| --- | --- | --- |
| `commander_thinking` | Commander 正在拆解任务 | - |
| `worker_started` | Worker 开始执行普通任务 | `workerId`, `taskId` |
| `worker_finished` | Worker 完成普通任务 | `workerId`, `taskId` |
| `collaboration_started` | 多角色协作开始 | `primaryWorkerId`, `supportWorkerIds` |
| `collaboration_worker_started` | 协作 Worker 开始 | `workerId`, `taskId` |
| `collaboration_worker_finished` | 协作 Worker 完成 | `workerId`, `taskId`, `content` |
| `collaboration_finished` | 多角色协作完成 | - |
| `task_graph_planned` | Commander 规划出 task graph | `graph` |
| `task_node_ready` | task graph 节点已就绪 | `graphId`, `nodeId`, `workerId` |
| `task_node_started` | task graph 节点开始执行 | `graphId`, `nodeId`, `workerId`, `attempt` |
| `task_node_reported` | 节点产出 worker report | `graphId`, `nodeId`, `workerId`, `report` |
| `task_node_evaluated` | 节点评估完成 | `graphId`, `evaluation` |
| `task_node_rework_requested` | 节点需要返工 | `graphId`, `nodeId`, `workerId`, `attempt`, `instruction` |
| `task_node_passed` | 节点通过 | `graphId`, `nodeId` |
| `task_node_failed` | 节点失败 | `graphId`, `nodeId`, `reason` |
| `task_graph_finished` | task graph 完成 | `graphId` |

`task_graph_planned.graph` 结构：

```json
{
  "graph_id": "graph-001",
  "root_task": "抖音节日转化率提升方案",
  "nodes": [
    {
      "id": "data",
      "role": "data",
      "worker_id": "worker-data",
      "title": "数据诊断",
      "objective": "分析任务中的数据、指标、归因和约束",
      "input_refs": [],
      "acceptance": ["..."],
      "max_attempts": 2
    }
  ]
}
```

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 当前 agent runtime 如果只产生最终 `Output`，服务端会把最终内容作为一个 `delta` 发出
- 后续 agent 支持 token 级 `LlmChunk` 后，服务端会把每个 chunk 映射为 `delta`
