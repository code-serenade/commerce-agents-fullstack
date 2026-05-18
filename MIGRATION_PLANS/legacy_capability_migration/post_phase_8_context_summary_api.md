# Post 8-12：附件与素材上下文摘要 API

状态：done

## 目标

- 把旧产品里的“附件、素材、导出交付物可被问答使用”的能力，先落到新产品后端契约里。
- 聚合会话附件、商品资产、商品素材、工作区 artifact，返回统一的上下文就绪状态。
- 给后续 Agent 上下文注入、前端右侧来源面板、导出/教学链路提供同一份数据基础。

## 后端变化

- `WorkspaceApi::context_summary` 新增工作区上下文摘要能力。
- 新增 `GET /api/app/workspaces/{workspaceId}/context-summary`。
- 查询参数支持 `conversationId`、`productId`、`runId`。
- 返回 `readiness=ready|partial|empty`、各类来源计数、引用 key、缺失项、警告和 `agentContext`。
- 不新增数据库表，复用现有 `conversation_files`、`product_assets`、`product_materials`、`workspace_artifacts`。

## 权限与降级

- 所有数据仍按当前 JWT 用户隔离。
- 会话、商品、工作区、运行记录都复用已有归属校验。
- 没有可用附件或素材时返回 `empty/partial`，不伪造上下文来源。

## 前端影响

- 本阶段不改前端。
- 前端后续可用该接口判断问答前是否有附件/素材/交付物上下文，并展示 Kimi Agent 风格来源面板。
