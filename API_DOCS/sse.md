# SSE API

接口前缀：`/api/sse`

说明：本文件只记录服务端使用 SSE 向前端返回的流式接口。当前 SSE 只用于 LLM 单次流式响应；普通业务通知仍然使用 WebSocket `/api/ws`。

## 1. LLM 流式聊天

- 方法：`POST`
- 路径：`/api/sse/llm/chat/stream`
- 鉴权：有效 App 用户 JWT
- 响应类型：`text/event-stream`

请求体：

```json
{
  "llmName": "ollama-gemma4",
  "messages": [
    {
      "role": "user",
      "content": "hello"
    }
  ]
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `llmName` | `string` | 否 | `config/services.toml` 中的 LLM client 名称；不传时使用第一个 `[[llm]]` |
| `messages` | `ChatMessage[]` | 是 | OpenAI-compatible chat messages，不能为空 |

图片消息示例：

```json
{
  "llmName": "ollama-gemma4",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "Describe this image."
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/png;base64,..."
          }
        }
      ]
    }
  ]
}
```

SSE 事件：

```text
event: delta
data: {"content":"hello"}

event: reasoning
data: {"content":"optional reasoning text"}

event: done
data: {}

event: error
data: {"message":"model unavailable"}
```

事件说明：

| 事件 | 说明 |
| --- | --- |
| `delta` | 模型输出正文增量 |
| `reasoning` | 模型 reasoning 增量；只有 provider 返回 reasoning 字段时才会出现 |
| `done` | 流结束 |
| `error` | 流式过程中发生错误 |

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 服务端会把 OpenAI-compatible provider stream 归一为上面的 SSE 事件
- 前端使用 `POST + fetch stream` 调用，不使用 `GET EventSource`
