# Post 8-132：问答发送 / 停止 / 错误状态反馈

状态：done

## 本次目标

本轮不新增后端 API，也不伪造执行进度，而是把 Agent Chat 已有的真实前端状态、后端调用状态和 abort 控制展示得更清楚：

- 正在调用后端 Agent 时，输入区上方显示“正在调用后端智能体，可随时停止”。
- 后端返回错误时，输入区附近同步显示真实错误信息，便于用户立刻知道本轮失败原因。
- 用户点击停止时，如果助手还没有产出正文，对话流显示“本轮回答已停止”，避免继续显示“正在生成”。
- 如果停止前已经有部分正文，则保留已有正文，不误标为失败。

## 修改范围

- `frontend/apps/web/src/pages/site/agent-chat/types.ts`
  - `AgentChatMessage.status` 增加 `stopped` 状态。

- `frontend/apps/web/src/pages/site/agent-chat/view-model/agent-chat-page-actions.ts`
  - `stopAgentStreaming` 在空回答场景下把 streaming 助手消息转成 `stopped`。

- `frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-composer-status.tsx`
  - 新增输入区状态条，只在必要状态显示：正在发送、错误、未登录。

- `frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-composer.tsx`
  - 接入输入区状态条。

- `frontend/apps/web/src/pages/site/agent-chat/components/thread-message-blocks.tsx`
  - 新增停止卡片。

- `frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-thread.tsx`
  - 在助手消息状态为 `stopped` 时展示停止卡片。

- `frontend/apps/web/lang/zhCN/messages.ts`
- `frontend/apps/web/lang/en/messages.ts`
  - 增加状态条和停止卡片文案。

## 产品内核变化

- 后端内核、SSE、Agent skill 和 API 本轮没有改动。
- 前端只是把已经存在的 `isStreaming`、`error`、`AbortController` 和消息状态显式反馈给用户。

## 用户可感知变化

- 发送问题后，输入区上方会明确显示当前正在调用后端智能体，用户知道系统不是卡死。
- 用户可以看到“可随时停止”的提示，停止操作更可信。
- 停止后如果还没有正文，对话流会显示“本轮回答已停止”，不会继续像还在等待模型。
- 后端错误会在输入区附近同步露出，用户不必只看顶部提示。

## 验证记录

- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web build`
- [x] `pnpm rules`

## 后续缺口

- 真实超时仍然由后端 / stream adapter 返回并通过已有 `streamTimedOut` 文案显示。
- 后续可继续补 route decision、worker started / finished、task graph 的更细粒度可视化，但必须来自后端 SSE。
