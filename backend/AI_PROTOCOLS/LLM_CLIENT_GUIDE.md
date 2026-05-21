# LLM Client 集成协议 (AI 专用)

> 当需求涉及 AI、LLM、图片识别、文本生成、多模型调用或外部模型服务时，AI 必须阅读本文档。

---

## 一、当前项目约定

项目通过 `agent` crate 接入 OpenAI-compatible Chat Completions 服务。`model-gateway-rs` 是 `agent` crate 的内部依赖，`web-server` 不直接依赖或初始化模型 client。

LLM client 统一初始化在：

```text
crates/agent/src/runtime.rs
```

`web-server` 只负责读取 `config/services.toml`，把 `[[llm]]` 配置传给 `agent::api::build_agent_manager`，并通过 `statics::agent_manager` 运行 agent session。

前端实时通信边界：

- WebSocket 只用于小铃铛通知、任务状态等业务推送。
- SSE 用于 agent session 事件流。
- 不再提供裸 LLM chat stream HTTP 接口。

---

## 二、配置方式

`config/services.toml` 支持多个 `[[llm]]` 配置块：

```toml
default_llm = "ollama-gpt-oss-20b"

[agent_llm_usage]
default = "ollama-gpt-oss-20b"
fast = "ollama-gpt-oss-20b"
thinking = "ollama-gpt-oss-20b"
vision = "vision-model"

[[llm]]
name = "ollama-gpt-oss-20b"
base_url = "http://127.0.0.1:11434/v1"
model = "gpt-oss:20b"
api_key = "sk-local"
max_tokens = 20000
temperature = 0.2
```

字段说明：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `name` | 是 | 当前 LLM 配置名称，必须唯一 |
| `base_url` | 是 | OpenAI-compatible 服务地址，可以是服务根地址，也可以是带 `/v1` 的 API base |
| `model` | 是 | 模型名，例如 `gemma4:26b` |
| `api_key` | 否 | 需要鉴权的 provider 使用 |
| `max_tokens` | 否 | 默认输出 token 上限 |
| `temperature` | 否 | 默认温度 |
| `reasoning_effort` | 否 | 支持 reasoning effort 的 provider 使用 |
| `chat_completions_endpoint` | 否 | 特殊 provider 才需要覆盖，普通情况不要配置 |

顶层 `default_llm` 可选，用于指定默认使用哪个 `[[llm]].name`。如果不配置，agent runtime 默认优先使用真实 Doubao 配置，否则使用第一个 `[[llm]]`。

`[agent_llm_usage]` 用于把 agent 功能映射到不同 `llm.name`：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `default` | 否 | 未命中特定用途时的 fallback |
| `fast` | 否 | commander routing、chat worker、快速回复、配置为 `llm_usage = "fast"` 的 task graph 节点 |
| `thinking` | 否 | 普通业务 worker、多角色分析和综合 |
| `vision` | 否 | 图片输入和视觉理解 |

task graph 节点和动态角色配置可以用 `llm_usage` 覆盖默认 worker 路由。合法值为 `fast`、`thinking` 或 `default`；未配置时普通业务 worker 走 `thinking`，chat worker 走 `fast`。

轻量协作图还可以在节点或动态角色配置里设置 `skill_autosuggest = false`，用于跳过自动技能建议，减少没有外部资料或工具适配器时的无效调用。

模型用途路由必须在 `agent` crate 内处理，web-server 不维护第二套 LLM registry，也不在 handler 中创建模型 client。`vision` 未配置时，带图片输入的请求必须返回明确错误，不允许降级成文本模型假装看图。

---

## 三、代码使用方式

HTTP handler 不直接调用 LLM。需要 AI 能力时，从 agent manager 运行 session：

```rust
use crate::statics::agent_manager::run_agent_stream;

let events = run_agent_stream(
    auth_user.user_id,
    req.workspace_id,
    req.agent_id,
    req.session_id,
    req.input,
)?;
```

agent crate 内部如需新增 cognition，可继续通过 `CognitionEngine` 或 `model-gateway-rs` 实现，但对外只暴露 agent runtime API。

---

## 四、图片识别

图片识别也应作为 agent 能力进入 runtime。新增图片输入时，必须先设计 agent request 的输入协议，例如 multipart 上传、资源 URL 或 data URL，再由 agent 内部构造 vision message。

不要在 `web-server` handler 中临时创建 vision LLM client。

---

## 五、验证命令

本项目提供本地 Ollama agent smoke example：

```bash
cargo run -p agent --bin ollama_task_graph_smoke
```

默认测试：

```text
OLLAMA_BASE_URL=http://127.0.0.1:11434
OLLAMA_MODEL=gemma4:26b
```

也可以覆盖：

```bash
OLLAMA_BASE_URL=http://127.0.0.1:11434 \
OLLAMA_MODEL=gemma4:26b \
cargo run -p agent --bin ollama_task_graph_smoke
```

---

## 六、前端 SSE 协议

Agent session 使用 Axum SSE：

```http
POST /api/sse/agent/session/stream
Authorization: Bearer <access_token>
Content-Type: application/json
Accept: text/event-stream
```

请求体：

```json
{
  "input": "hello agent",
  "agentId": "web-agent",
  "sessionId": "session-001",
  "workspaceId": "workspace-001"
}
```

服务端事件直接来自 `agent::api::SessionEvent`：

```text
event: started
data: {"type":"started",...}

event: worker_started
data: {"type":"worker_started",...}

event: output
data: {"type":"output",...}

event: finished
data: {"type":"finished",...}
```

约束：

1. SSE 用于 agent session stream，不用于普通通知。
2. 前端普通通知继续使用 `/api/ws`。
3. Agent SSE 使用 `POST + fetch stream`，不使用 `GET EventSource`。
4. web-server 只把 `SessionEvent` 映射为 SSE event，不解析 provider 原始 stream。

---

## 七、开发规则

1. 新增 AI/LLM 功能前，服务端开发文档必须写清楚使用哪个 agent 能力和 `llm.name`。
2. 如果需要图片识别，必须说明图片来源、格式、大小限制和错误处理方式。
3. 不要把 API key 写死在代码里，只允许从配置读取。
4. 不要在 `web-server` 中重复创建 LLM client。
5. 修改 LLM 配置结构后，必须同步更新：
   - `config/services-example.toml`
   - `README.md`
   - `../API_DOCS/common.md`
   - 相关 `user_docs/`
6. 如果新增或修改 agent SSE 接口，必须同步更新 `../API_DOCS/sse.md`。
7. 修改 LLM 调用能力后，至少运行：

```bash
cargo check --workspace --examples
cargo run -p agent --bin ollama_task_graph_smoke
```
