# WebSocket API

连接路径：`/api/ws`

说明：本 WebSocket 主要用于后台向前端推送小铃铛通知。连接必须通过 JWT 鉴权，没有 token 或 token 校验失败时不会建立 WebSocket 连接。前端暂时不需要发送业务消息，只需要建立连接并监听服务端消息。当前保留 `ping/pong` 用于连通性测试。

LLM 流式输出不使用 WebSocket，统一使用 `POST /api/sse/llm/chat/stream` 的 SSE 协议；见 `API_DOCS/sse.md`。

## 1. 连接方式

连接时必须携带有效 JWT access token。服务端会在 WebSocket upgrade 前校验 token，校验通过后才会建立连接。

开发环境示例：

```text
ws://127.0.0.1:<http_port>/api/ws?token=<access_token>
```

线上 HTTPS 环境通常使用：

```text
wss://<host>/api/ws?token=<access_token>
```

如果客户端可以设置请求头，也可以通过 `Authorization` 请求头传递同一个 JWT access token：

```http
Authorization: Bearer <access_token>
```

浏览器原生 `WebSocket` 通常无法设置 `Authorization` 请求头，所以前端页面测试建议使用 query 参数 `token`。不管使用哪种方式，token 都是必填的。

## 2. 鉴权失败

如果缺少 token 或 token 校验失败，服务端会拒绝 WebSocket upgrade，不会进入已连接状态，也不会发送 `connected` 消息。

常见 HTTP 状态：

| 状态码 | 含义 |
| --- | --- |
| `401 Unauthorized` | 未提供 token、token 无效或 JWT 校验失败 |

## 3. 连接成功消息

连接成功后，服务端会主动发送一条 `connected` 消息。

服务端消息：

```json
{
  "type": "connected",
  "userId": "user-id-from-jwt",
  "payload": {}
}
```

字段说明：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `type` | `string` | 固定为 `connected` |
| `userId` | `string` | 当前连接对应的用户 ID，来自 JWT subject |
| `payload` | `object \| null` | JWT 扩展信息；没有扩展信息时可能不存在 |

## 4. Ping/Pong 测试

前端可以发送：

```json
{
  "type": "ping"
}
```

服务端会返回：

```json
{
  "type": "pong",
  "payload": {}
}
```

说明：

- `ping/pong` 只用于前端测试连接是否正常。
- 当前不支持前端发送其他业务消息。
- 如果发送非 JSON 或不支持的 `type`，服务端会返回错误消息。

错误消息示例：

```json
{
  "type": "error",
  "message": "unsupported message type"
}
```

## 5. 小铃铛通知消息

后续业务需要推送通知时，服务端会向指定用户或全部在线用户发送 `notification` 消息。

服务端消息：

```json
{
  "type": "notification",
  "payload": {
    "title": "测试通知",
    "content": "WebSocket 小铃铛通知连接正常",
    "level": "info"
  }
}
```

字段说明：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `type` | `string` | 固定为 `notification` |
| `payload.title` | `string` | 通知标题 |
| `payload.content` | `string` | 通知内容 |
| `payload.level` | `info \| success \| warning \| error` | 通知等级 |
| `payload.data` | `object \| null` | 可选扩展数据 |

当前后端已经预留测试通知结构，但不会在连接后自动发送 `notification`。目前前端测试应以 `connected` 和 `ping/pong` 为准。

## 6. 前端测试示例

浏览器控制台示例：

```javascript
const token = "<access_token>";
const ws = new WebSocket(`ws://127.0.0.1:<http_port>/api/ws?token=${encodeURIComponent(token)}`);

ws.onopen = () => {
  console.log("ws opened");
  ws.send(JSON.stringify({ type: "ping" }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log("ws message", message);
};

ws.onclose = (event) => {
  console.log("ws closed", event.code, event.reason);
};

ws.onerror = (event) => {
  console.error("ws error", event);
};
```

前端处理建议：

- 收到 `connected`：标记 WebSocket 已连接。
- 收到 `pong`：用于测试链路正常。
- 收到 `notification`：追加到小铃铛通知列表，或触发未读数量刷新。
- 收到 `error`：打印日志或展示开发环境错误提示。
