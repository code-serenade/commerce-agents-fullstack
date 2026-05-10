# Post 8-13：工作区上下文摘要接入 Agent 问答主链路

状态：done

## 目标

- 把 Post 8-12 已完成的工作区上下文摘要，接入 Agent SSE 主链路。
- 让会话附件、商品资产、商品素材、工作区交付物不只可查询，也能在问答时进入 runtime metadata。
- 前端用现有 Kimi Agent 风格右侧过程区展示真实上下文来源状态。

## 后端变化

- `POST /api/sse/agent/session/stream` 的 `metadata.context` 支持 `conversationId`。
- SSE 绑定 `workspace_run` 后调用 `WorkspaceApi::context_summary`。
- 新增 `workspace_context_loaded` SSE 事件。
- Agent runtime metadata 新增 `workspaceContext.*`，并把工作区来源 refs 追加到 `context.refs`。
- 不读取文件正文，不做 OCR，不伪造附件摘要。

## 前端变化

- `AgentRuntimeEvent` 增加 `workspace_context_loaded`。
- 问答发送时将当前 `conversationId` 传入 SSE metadata。
- 右侧过程卡片和时间线展示工作区上下文来源数量、状态和缺失/警告。

## 用户可感知变化

- 用户在问答页发送问题时，后端会自动识别当前会话和工作区可用的附件、素材、交付物。
- 右侧过程区会显示工作区上下文是否 ready、partial 或 empty。
- 如果没有上传附件或没有素材，产品会明确显示空/缺失状态，而不是假装已经读取资料。

## 验证

- `cargo fmt`
- `cargo fmt --check`
- `cargo test -p web-server workspace_context`
- `cargo test -p web-server metadata_parses_conversation_id_context_selection`
- `cargo test -p web-server metadata_rejects_too_long_conversation_id`
- `cargo test -p web-server sse`
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
