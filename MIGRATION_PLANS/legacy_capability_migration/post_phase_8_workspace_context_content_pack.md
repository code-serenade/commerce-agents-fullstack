# Post 8-14：工作区上下文内容包接入 Agent 问答链路

状态：done

## 目标

- 把旧产品里“附件、素材、报告可作为问答上下文”的能力继续迁到新产品后端。
- 不再只给 Agent 来源引用，而是把数据库里已经存在的可读文本整理成受限内容包。
- 严格避免伪造附件读取：不读 object storage，不做 OCR，不解析二进制文件。

## 后端变化

- `WorkspaceApi` 新增 `context_pack`。
- 新增 `GET /api/app/workspaces/{workspaceId}/context-pack`。
- 内容包来源：
  - `product_material.content`
  - `product_asset.description`
  - `workspace_artifact.content`，没有正文时退回 `preview`
  - `conversation_file.metadata` 中已有的抽取文本字段
- `POST /api/sse/agent/session/stream` 在 `workspace_context_loaded` 后继续发送 `workspace_context_pack_loaded`。
- Agent runtime metadata 新增 `workspaceContextPack.*`，并把可读文本追加到 `context.summary`。

## 前端变化

- `AgentRuntimeEvent` 增加 `workspace_context_pack_loaded`。
- 右侧上下文卡片展示可读来源数量、内容类型和警告。
- 时间线展示可读上下文是否已接入。

## 用户可感知变化

- 用户在问答页发送问题后，右侧过程区会看到“可读上下文”状态。
- 如果工作区已有商品素材正文、资产描述、报告正文或附件抽取文本，后端会把这些文本真实注入回答链路。
- 如果只是 PDF/图片等二进制附件但还没有抽取文本，界面会显示 metadata-only/警告，不会让用户误以为 Agent 已读正文。

## 验证

- `cargo fmt`
- `cargo test -p service context_pack`
- `cargo test -p web-server workspace_context_pack`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo check`
- `cargo clippy`
- `pnpm -C apps/web rules`
- `pnpm -C apps/web lint`
- `pnpm -C apps/web test`
- `pnpm -C apps/web build`
- `pnpm rules`
- `git diff --check`
