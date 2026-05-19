# 通用约定

## 1. 认证方式

所有当前已暴露接口都需要 JWT 鉴权。

请求头：

```http
Authorization: Bearer <access_token>
```

WebSocket 连接同样必须通过 JWT 鉴权。服务端会在 WebSocket upgrade 前校验 token，没有 token 或 token 校验失败时不会建立连接。

支持两种 token 传递方式：

```http
Authorization: Bearer <access_token>
```

或浏览器原生 `WebSocket` 常用的 query 参数方式：

```text
ws://<host>/api/ws?token=<access_token>
```

前端浏览器通常不能直接为原生 `WebSocket` 设置 `Authorization` 请求头，因此前端测试建议先使用 `token` query 参数。无论使用哪种方式，token 都是必填的。

服务启动时会从远程 `auth` 服务拉取 JWT 验签配置，配置样例见 `config/services-example.toml`：

```toml
[jwt_verify]
url = "http://127.0.0.1:29001/internal/jwt_verify_config"
header = "x-internal-token"
token = "dev-internal-token-change-me"
```

## 1.1 Agent LLM 配置

项目通过 `agent` crate 接入 OpenAI-compatible Chat Completions 服务。服务启动时会读取 `config/services.toml` 中的多个 `[[llm]]` 配置，并用这些配置初始化 agent runtime。

配置样例：

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
| `name` | 是 | LLM 配置名称，必须唯一 |
| `base_url` | 是 | OpenAI-compatible 服务地址；可以是根地址，也可以是带 `/v1` 的 API base |
| `model` | 是 | 模型名 |
| `api_key` | 否 | 远程 provider 需要鉴权时使用 |
| `max_tokens` | 否 | 默认输出 token 上限 |
| `temperature` | 否 | 默认温度 |
| `reasoning_effort` | 否 | 支持 reasoning effort 的 provider 使用 |
| `chat_completions_endpoint` | 否 | 特殊 provider 覆盖 endpoint 时使用，普通情况不需要配置；当 `base_url` 已以 `/v1`、`/api/v1` 或 `/api/v3` 结尾时，客户端会自动请求 `chat/completions` |

顶层 `default_llm` 可选，用于指定默认使用哪个 `[[llm]].name`。如果不配置，运行选项接口和 Agent runtime 都会优先选择真实配置的豆包模型（`name` 或 `model` 包含 `doubao`），否则使用第一个 `[[llm]]`。前端只展示 `services.toml` 里真实存在的模型，不会伪造豆包选项。

`[agent_llm_usage]` 用于把 agent 内部能力映射到不同模型：`fast` 用于 commander 路由、chat worker 和快速回复；`thinking` 用于普通业务 worker、多角色分析和综合；`vision` 用于 `metadata.attachments.images` 图片输入。`vision` 未配置时，带图片请求会失败并返回明确错误，不会降级成文本模型。

当前仓库提交的 `backend/config/services.toml` 没有真实豆包 Ark 配置，也没有 Ark API key；因此当前本地运行时不会调用豆包。要启用豆包，必须在部署机器的 `services.toml` 中添加真实 `[[llm]]` 配置并设置 `default_llm`，不要把真实 key 提交到仓库。

豆包 Ark 示例：

```toml
[[llm]]
name = "doubao-external"
base_url = "https://ark.cn-beijing.volces.com/api/v3"
model = "doubao-seed-2-0-pro-260215"
api_key = "CHANGE_ME"
max_tokens = 20000
temperature = 0.2
```

本地 Ollama agent smoke 测试：

```bash
cargo run -p agent --bin ollama_task_graph_smoke
```

## 2. 统一响应格式

项目中的 handler 统一通过 `CommonResponse<T>` 返回成功结果，通过 `CommonError` 返回错误结果。

HTTP JSON 字段统一使用 `camelCase`。枚举值和权限码保持业务定义，例如 `enabled`、`disabled`、`access_control:roles`。

成功响应示例：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "userId": "demo-user-id"
  }
}
```

错误响应示例：

```json
{
  "code": -11002,
  "message": "permission denied: access_control:roles"
}
```

## 3. HTTP 状态码

- `200 OK`：请求成功
- `400 Bad Request`：参数校验失败，或一般业务错误
- `401 Unauthorized`：未登录、Token 无效、JWT 校验失败
- `403 Forbidden`：已登录，但不是有效后台管理员，或权限不足
- `500 Internal Server Error`：服务内部错误

## 4. 后台权限模型

后台相关接口默认要求当前用户：

1. 是 `admin_users` 中存在的后台用户
2. 状态为 `Enabled`
3. 拥有接口要求的权限码，或拥有 `root` 角色

`root` 角色有两个特殊限制：

- 角色编码 `root` 是保留值，不能通过普通创建接口创建
- 非 `root` 用户不能给别人分配 `root` 角色
- 非 `root` 用户不能修改 `root` 角色的权限

## 5. 后台业务错误码

来自 `crates/error-code/src/admin.rs`：

| 错误码 | 含义 |
| --- | --- |
| `-11000` | 后台用户不存在 |
| `-11001` | 后台用户已禁用 |
| `-11002` | 权限不足 |
| `-11003` | 保留角色限制触发，例如 `root` 角色相关操作 |

## 6. 文档维护方式

项目不再暴露 Swagger UI 或 OpenAPI JSON。

当前接口文档统一维护在本目录下的 Markdown 文件中：

- `API_DOCS/common.md`
- `API_DOCS/admin.md`
- `API_DOCS/app.md`
- `API_DOCS/sse.md`
- `API_DOCS/ws.md`
