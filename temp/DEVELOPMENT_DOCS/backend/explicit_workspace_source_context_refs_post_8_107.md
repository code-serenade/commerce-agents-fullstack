# Post 8-107 显式工作区来源上下文引用后端设计

## 需求摘要

用户需要新产品继续用同事提供的新产品后端框架承接旧产品能力。上一轮已经让历史 `workspace_artifact` 可以通过 `contextRefs` 显式进入 Agent 问答上下文；本轮继续补齐旧产品中“用户手动选择附件、商品素材、商品资产作为回答依据”的能力。

核心原则不变：

- 后端先提供真实上下文读取能力，前端后续只消费真实 API。
- 不从历史对话、工作区绑定商品、默认测试商品或用户问题关键词中自动推断商品资料。
- 只有用户显式传入的引用才进入本轮上下文。
- 所有引用都必须按当前登录用户做归属校验。

## 数据库设计

不新增表，不新增字段，不做 migration。

复用现有表：

- `conversation_files`
- `product_assets`
- `product_materials`
- `workspace_artifacts`
- `conversations`
- `products`
- `workspaces`

## 引用协议

继续支持：

- `workspace_artifact:{artifactUuid}`
- `workspace_artifact/{artifactUuid}`
- `artifact:{artifactUuid}`
- `artifact/{artifactUuid}`

新增支持：

- `conversation_file:{fileUuid}`
- `conversation_file/{fileUuid}`
- `file:{fileUuid}`
- `file/{fileUuid}`
- `product_asset:{assetUuid}`
- `product_asset/{assetUuid}`
- `asset:{assetUuid}`
- `asset/{assetUuid}`
- `product_material:{materialUuid}`
- `product_material/{materialUuid}`
- `material:{materialUuid}`
- `material/{materialUuid}`

规则：

- `contextRefs` 仍最多 20 个，每项最多 160 字符。
- 支持的引用 ID 必须是 UUID。
- 不支持的引用类型暂时忽略，避免影响已有 `product:{uuid}` 等其他上下文入口。
- 重复引用去重。

## Service 设计

修改：

- `WorkspaceApi::context_summary`
- `WorkspaceApi::context_pack`

新增内部解析结构：

- `ExplicitWorkspaceContextRefs`

新增校验逻辑：

- 会话附件必须属于当前用户、状态 active，且所属会话未删除。
- 商品资产必须属于当前用户、状态 active，且所属商品未归档。
- 商品素材必须属于当前用户、状态 active，且所属商品未归档。
- 工作区交付物继续要求属于当前用户、当前 workspace，且状态 active。

聚合规则：

- `conversationId` 仍会读取该会话 active 附件。
- `productId` 或 `includeWorkspaceProductContext=true` 仍会读取该商品 active 资产和素材。
- `contextRefs` 中显式传入的附件、商品资产、商品素材、交付物会额外加入来源列表。
- 合并后按资源 ID 去重。
- 显式商品资产/素材不会自动改写本轮 `productId`，避免把某个素材反向当成本轮“手动选中商品”。

## Web / SSE 设计

不新增 HTTP 路由，不修改请求结构。

现有入口继续使用：

- `GET /api/app/workspaces/{workspaceId}/context-summary?contextRefs=...`
- `GET /api/app/workspaces/{workspaceId}/context-pack?contextRefs=...`
- `POST /api/sse/agent/session/stream` 的 `metadata.context.contextRefs`

SSE 仍由 web-server 把 `contextRefs` 传入 service 层，service 层统一读取和校验。

## 鉴权设计

- JWT 由 web-server 中间件提供。
- 当前用户 ID 继续来自 `AuthUser.user_id`。
- service 层按当前用户校验所有显式引用。
- 不允许跨用户读取附件、商品资产、商品素材或交付物。

## API 文档

同步更新：

- `API_DOCS/app.md`
- `API_DOCS/sse.md`

## 验收

- 显式 `conversation_file:{uuid}` 可以进入 context summary / pack。
- 显式 `product_asset:{uuid}` 可以进入 context summary / pack。
- 显式 `product_material:{uuid}` 可以进入 context summary / pack。
- 非当前用户、已归档、非法 UUID 的显式引用会失败。
- 未显式传入时不会自动读取历史附件、历史商品素材或历史交付物。
