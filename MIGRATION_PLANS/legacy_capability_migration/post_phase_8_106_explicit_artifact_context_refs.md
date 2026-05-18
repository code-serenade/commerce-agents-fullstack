# Post 8-106：显式交付物上下文引用接入 Agent SSE

## 本轮定位

本轮属于旧产品能力迁移计划后的第 106 个补强项：`Post 8-106`。

目标是让用户在前端手动选择历史交付物、报告或搜索证据后，后端可以把这些资料作为本轮问答上下文传给 Agent，同时继续避免默认商品、历史报告或测试样例自动串入普通问答。

## 后端变化

- `WorkspaceApi::context_summary` 和 `WorkspaceApi::context_pack` 新增 `context_refs` 入参。
- 新增显式 artifact 引用解析：
  - `workspace_artifact:{artifactUuid}`
  - `artifact:{artifactUuid}`
- 显式 artifact 会经过当前用户、当前 workspace、active 状态校验。
- 显式 artifact 会和本轮 `runId` 范围 artifact 合并去重。
- 未传显式 artifact refs 时，SSE 仍只读取当前新建 run 范围，不自动读取历史 run 的交付物。
- `search_evidence` artifact 继续通过 `contentKind=search_evidence_summary` 进入上下文，不直接注入原始 JSON。

## Web / API 变化

- Agent SSE `metadata.context.contextRefs` 支持 `workspace_artifact:{id}` / `artifact:{id}`。
- `GET /api/app/workspaces/{workspaceId}/context-summary` 增加 `contextRefs` 查询参数。
- `GET /api/app/workspaces/{workspaceId}/context-pack` 增加 `contextRefs` 查询参数。
- HTTP `contextRefs` 使用逗号分隔字符串，最多 20 个。

## 用户可感知变化

用户后续在前端历史交付物、搜索证据或报告列表中主动选择某个 artifact 后，再继续提问，Agent 可以基于这份真实历史交付物回答。用户没有主动选择时，系统不会把历史报告或旧商品资料自动带入当前问答。

## 边界

- 不新增数据库表。
- 不接入外部平台。
- 不从自然语言中猜测 artifact。
- 不读取 object storage 原始二进制文件。
- 不伪造搜索来源或平台数据。

## 验证重点

- artifact ref 解析支持 `workspace_artifact:` 和 `artifact:`。
- 非 artifact ref 如 `product:` 不会被 workspace artifact 解析器误处理。
- invalid UUID 会返回 workspace 请求错误。
- 超过 20 个 refs 会被拒绝。
