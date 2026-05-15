# Post 8-106 显式交付物上下文引用后端设计

## 需求摘要

用户需要新产品继续按“后端为主、前端适配后端”的路线，把旧产品的历史材料、搜索证据、交付物复用能力接入问答主链路。但必须避免历史对话、默认商品或测试商品被自动注入，只有用户显式选择的上下文才允许进入本轮 Agent。

本切片实现 `contextRefs` 对工作区交付物的显式引用支持，尤其让已沉淀的 `search_evidence` 可以作为可读摘要进入 Agent 上下文。

## 数据库设计

不新增表，不新增字段，不做 migration。

复用现有表：

- `workspaces`
- `workspace_runs`
- `workspace_artifacts`
- `conversation_files`
- `product_assets`
- `product_materials`

## 引用协议

新增支持的显式引用：

- `workspace_artifact:{artifactUuid}`
- `artifact:{artifactUuid}`

保留已有引用：

- `product:{productUuid}`

解析规则：

- 只解析显式传入的 `contextRefs`。
- 引用 ID 必须是 UUID。
- 交付物必须属于当前登录用户和当前 workspace。
- 交付物必须是 active 状态。
- 不从用户问题文本、会话历史、workspace 绑定商品或测试样例中推断交付物。
- 不因为传入 workspaceId 就自动读取历史 run 的全部 artifact。

## Service 设计

修改：

- `GetWorkspaceContextSummaryRequest`
- `GetWorkspaceContextPackRequest`

新增字段：

- `context_refs: Vec<String>`

`WorkspaceApi::context_summary`：

- 继续保留 `run_id` 过滤能力。
- 额外解析显式 artifact refs。
- 如果显式 artifact refs 指向不属于当前用户/workspace 或已归档的 artifact，返回业务错误。
- 合并 run 范围 artifact 与显式 artifact，按 ref 去重。

`WorkspaceApi::context_pack`：

- 继续复用已有 `context_pack_artifact_candidate`。
- `search_evidence` artifact 继续输出 `contentKind=search_evidence_summary`。
- 显式 artifact 与本轮 run artifact 合并后进入 pack，上限仍受 `maxItems/maxChars` 保护。

## Web / SSE 设计

`POST /api/sse/agent/session/stream`：

- 将 `metadata.context.contextRefs` 原样传入 workspace context summary/pack。
- Agent SSE 自动注入仍只默认读取当前新建 run；历史 artifact 必须通过 `workspace_artifact:{uuid}` 或 `artifact:{uuid}` 显式传入。

`GET /api/app/workspaces/{workspaceId}/context-summary`
`GET /api/app/workspaces/{workspaceId}/context-pack`

- 增加可选 query 参数 `contextRefs`。
- 支持逗号分隔，最多 20 个，每项最多 160 字符。

## 鉴权设计

- JWT 仍由 web-server 中间件提供。
- 当前用户 ID 继续使用 `AuthUser.user_id`。
- service 层统一通过 `ensure_request_workspace`、`ensure_active_artifact_owner` 做归属校验。

## API 文档

同步更新：

- `API_DOCS/sse.md`
- `API_DOCS/app.md`

## 验收

- `contextRefs=["workspace_artifact:{uuid}"]` 时，summary/pack 能包含该 artifact。
- `search_evidence` artifact 进入 pack 时为可读摘要，不注入原始 JSON。
- 未传显式 artifact refs 时，SSE 仍只读取当前 run artifact，不自动读取历史 artifact。
- 非当前用户、非当前 workspace、归档或非法 UUID 的 artifact ref 会失败。
