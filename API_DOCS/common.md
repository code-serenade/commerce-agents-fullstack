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

项目通过 `agent` crate 接入 `model-gateway-rs` 的 LLM client。服务启动时会读取 `config/services.toml` 中的多个 `[[llm]]` 配置，并用这些配置初始化 agent runtime。

配置样例：

```toml
[agent_llm_usage]
default = "ollama-gpt-oss-20b"
fast = "ollama-gpt-oss-20b"
thinking = "ollama-gpt-oss-20b"
vision = "vision-model"

[[llm]]
name = "ollama-gpt-oss-20b"
provider = "ollama"
base_url = "http://127.0.0.1:11434"
model = "gpt-oss:20b"
max_tokens = 20000
temperature = 0.2
```

字段说明：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `name` | 是 | LLM 配置名称，必须唯一 |
| `provider` | 否 | LLM client 类型：`chat_completions`、`ollama` 或 `doubao`；不配置时默认为 `chat_completions` |
| `base_url` | 是 | 模型服务地址；`ollama` 通常使用根地址如 `http://127.0.0.1:11434`，`doubao` 和 `chat_completions` 可使用根地址或版本化 API base |
| `model` | 是 | 模型名 |
| `api_key` | 否 | 远程 provider 需要鉴权时使用；`doubao` provider 必填 |
| `max_tokens` | 否 | 默认输出 token 上限 |
| `temperature` | 否 | 默认温度 |
| `reasoning_effort` | 否 | `chat_completions` provider 且上游支持 reasoning effort 时使用 |
| `thinking` | 否 | 是否启用 provider 的思考输出；`ollama` 和 `doubao` provider 会转成各自原生参数，`chat_completions` 会发送 OpenAI-compatible 扩展字段 |
| `chat_completions_endpoint` | 否 | 特殊 provider 覆盖 endpoint 时使用，普通情况不需要配置 |

`[agent_llm_usage]` 是唯一的模型用途路由入口。`default` 用于未命中特定用途时的 fallback；如果省略 `default`，Agent runtime 使用第一个 `[[llm]]`。`fast` 用于 commander 路由、chat worker、快速回复和配置为 `llm_usage = "fast"` 的 task graph 节点或动态角色；`thinking` 用于普通业务 worker、多角色分析和综合；`vision` 用于图片理解和系统图片预处理。task graph 节点和动态角色配置可配置 `llm_usage` 为 `fast`、`thinking` 或 `default` 覆盖默认 worker 路由。`vision` 未配置时，带图片处理请求会失败并返回明确错误，不会降级成文本模型。模型配置是服务端边界，普通前端不允许配置、展示、缓存或提交模型名称。

task graph 节点和动态角色配置还可设置 `skill_autosuggest = false`，用于轻量协作或最终润色节点跳过自动技能建议，避免没有外部资料或工具适配器时产生无效调用。

Agent runtime 还支持 worker 单节点交互超时：

```toml
[agent_runtime]
worker_cognition_timeout_sec = 45
```

该值统一限制单个 worker 等待 LLM cognition 的时间。task graph 节点、动态角色和最终节点不配置单独超时；超时后后端会把节点报告为 `timed_out`，并继续产生 `task_node_failed` / `task_graph_finished` 过程事件，避免 SSE 等到整轮 session 超时才失败。

要启用豆包，必须在部署机器的 `services.toml` 中添加真实 `[[llm]]` 配置和 `[agent_llm_usage]` 映射，不要把真实 key 提交到仓库。

豆包 Ark 示例：

```toml
[[llm]]
name = "doubao-external"
provider = "doubao"
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

## 7. 文件签名服务约定

文件签名服务由独立 file-server 提供，不在本仓库 `backend/` 目录内实现。前端统一通过 `group=file` 调用该服务域，`group` 只表示目标服务域，不作为 URL 路径前缀。

对象路径由 file-server 统一生成：

```text
private bucket:
  users/{userScope}/images/{sha256}.{ext}
  users/{userScope}/docs/{sha256}.{ext}
```

`userScope` 由 file-server 根据 JWT 当前用户生成。客户端不得自行拼接 S3 key。

### 7.1 服务端上传图片

- 方法：`PUT`
- 路径：`/file/sign/upload/image`
- 鉴权：需要 JWT

Query：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `ext` | `string` | 是 | 文件扩展名，不含点号，例如 `png` |

请求体：原始文件 bytes。前端不计算 SHA-256，不直传对象存储。

成功响应 `data`：

```json
{
  "key": "users/{userScope}/images/{sha256}.png",
  "url": "https://object-storage.example/private/users/{userScope}/images/{sha256}.png",
  "sha256": "{serverCalculatedSha256}",
  "size": 12345,
  "already_uploaded": false
}
```

file-service 会校验文件大小、扩展名和图片文件头，计算 SHA-256，并写入对象存储。如果 `already_uploaded=true`，表示相同内容对象已存在，服务端跳过对象存储写入并直接返回对象信息。

### 7.2 服务端上传文档

- 方法：`PUT`
- 路径：`/file/sign/upload/document`
- 鉴权：需要 JWT

Query：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `ext` | `string` | 是 | 文件扩展名，不含点号，例如 `pdf`、`docx`、`txt`、`md` |

请求体和响应结构同“服务端上传图片”。file-service 会校验文档扩展名和文件头或 UTF-8 文本内容。

文档 key 示例：

```json
{
  "key": "users/{userScope}/docs/{sha256}.pdf"
}
```

### 7.3 兼容旧上传签名

旧的 `GET /file/sign/upload/image` 和 `GET /file/sign/upload/document` 签名直传接口仍可兼容保留，但 Web 前端上传业务文件应优先使用上面的服务端上传接口，由 file-service 负责 SHA-256 计算、文件校验和对象存储写入。
