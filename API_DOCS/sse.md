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

SSE 事件：

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

- 只需要监听 `delta` / `done` / `error`
- 收到 `delta` 后把 `data.content` 追加到当前回复文本
- 收到 `done` 后结束本次回复
- 收到 `error` 后展示错误信息并结束本次回复
- 前端使用 `POST + fetch stream` 调用，不使用 `GET EventSource`

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 当前 agent runtime 如果只产生最终 `Output`，服务端会把最终内容作为一个 `delta` 发出
- 后续 agent 支持 token 级 `LlmChunk` 后，服务端会把每个 chunk 映射为 `delta`
