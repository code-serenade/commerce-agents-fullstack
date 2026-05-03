# LLM Client 集成协议 (AI 专用)

> 当需求涉及 AI、LLM、图片识别、文本生成、多模型调用或外部模型服务时，AI 必须阅读本文档。

---

## 一、当前项目约定

项目通过 `model-gateway-rs` 接入 OpenAI-compatible Chat Completions 服务。

当前依赖：

```toml
model-gateway-rs = { version = "0.2.4" }
```

LLM client 统一初始化在：

```text
crates/web-server/src/statics/llm_client.rs
```

不要在 handler 里临时 new LLM client。业务代码应通过 `statics::llm_client` 获取已初始化的 client。

前端实时通信边界：

- WebSocket 只用于小铃铛通知、任务状态等业务推送。
- SSE 只用于 LLM 单次流式响应。

---

## 二、配置方式

`config/services.toml` 支持多个 `[[llm]]` 配置块：

```toml
[[llm]]
name = "ollama-gemma4"
base_url = "http://127.0.0.1:11434"
model = "gemma4:26b"
max_tokens = 20000
temperature = 0.2
```

字段说明：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `name` | 是 | 当前 LLM client 名称，必须唯一 |
| `base_url` | 是 | OpenAI-compatible 服务地址，可以是服务根地址，也可以是带 `/v1` 的 API base |
| `model` | 是 | 模型名，例如 `gemma4:26b` |
| `api_key` | 否 | 需要鉴权的 provider 使用 |
| `max_tokens` | 否 | 默认输出 token 上限 |
| `temperature` | 否 | 默认温度 |
| `reasoning_effort` | 否 | 支持 reasoning effort 的 provider 使用 |
| `chat_completions_endpoint` | 否 | 特殊 provider 才需要覆盖，普通情况不要配置 |

`base_url` 不需要用户手动拼完整 `/v1/chat/completions`。`model-gateway-rs` 会按 OpenAI-compatible 规则自动选择 endpoint。

---

## 三、代码使用方式

优先使用封装好的静态入口：

```rust
use model_gateway_rs::model::llm::{ChatMessage, LlmInput};
use crate::statics::llm_client;

let output = llm_client::chat_once(
    Some("ollama-gemma4"),
    LlmInput {
        messages: vec![ChatMessage::user("hello")],
    },
)
.await?;
```

如果不传名称，默认使用配置中的第一个 `[[llm]]`：

```rust
llm_client::chat_once(None, input).await?;
```

---

## 四、图片识别

图片识别使用 `ChatMessage::user_with_image(...)` 或 content parts。

示例：

```rust
use model_gateway_rs::model::llm::{ChatMessage, LlmInput};

let input = LlmInput {
    messages: vec![ChatMessage::user_with_image(
        "Describe the image briefly.",
        "data:image/png;base64,...",
    )],
};
```

模型必须本身支持 vision。若 provider 返回 `{ "error": ... }`，`model-gateway-rs` 会返回清晰的 API error。

---

## 五、验证命令

本项目提供本地 Ollama 图片识别 smoke example：

```bash
cargo run -p web-server --example llm_vision_smoke
```

默认测试：

```text
LLM_BASE_URL=http://127.0.0.1:11434
LLM_MODEL=gemma4:26b
```

也可以覆盖：

```bash
LLM_BASE_URL=http://127.0.0.1:11434 \
LLM_MODEL=gemma4:26b \
cargo run -p web-server --example llm_vision_smoke
```

预期输出类似：

```text
The image features a red circle.
```

---

## 六、前端 SSE 协议

LLM 流式输出使用 Axum SSE：

```http
POST /api/sse/llm/chat/stream
Authorization: Bearer <access_token>
Content-Type: application/json
Accept: text/event-stream
```

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

服务端事件：

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

约束：

1. SSE 只用于 LLM stream，不用于普通通知。
2. 前端普通通知继续使用 `/api/ws`。
3. LLM SSE 使用 `POST + fetch stream`，不使用 `GET EventSource`。
4. 如果 provider 返回 OpenAI-compatible stream，web-server 负责归一成 `delta` / `reasoning` / `done` / `error`。

---

## 七、开发规则

1. 新增 AI/LLM 功能前，服务端开发文档必须写清楚使用哪个 `llm.name`。
2. 如果需要图片识别，必须说明图片来源、格式、大小限制和错误处理方式。
3. 不要把 API key 写死在代码里，只允许从配置读取。
4. 不要在业务 handler 中重复创建 LLM client。
5. 修改 LLM 配置结构后，必须同步更新：
   - `config/services-example.toml`
   - `README.md`
   - `../API_DOCS/common.md`
   - 相关 `user_docs/`
6. 如果新增或修改 LLM stream 接口，必须同步更新 `../API_DOCS/sse.md`。
7. 修改 LLM 调用能力后，至少运行：

```bash
cargo check --workspace --examples
cargo run -p web-server --example llm_vision_smoke
```
