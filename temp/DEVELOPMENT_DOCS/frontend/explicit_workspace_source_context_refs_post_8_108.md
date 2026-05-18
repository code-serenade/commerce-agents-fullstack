# Post 8-108 前端显式资料引用接入

## 背景

Post 8-107 已在后端支持 `conversation_file:{uuid}`、`product_asset:{uuid}`、`product_material:{uuid}` 和 `workspace_artifact:{uuid}` 等显式上下文引用。前端当前只把已选商品传给 Agent 问答，上传到当前会话的文件只能显示在输入区，不能由用户明确选择为本轮问答资料。

## 目标

本轮只打通当前会话已上传文件：

- 上传文件继续走现有会话文件上传和抽取文本登记链路。
- 已上传文件进入输入区的“资料”选择面板。
- 用户手动勾选后，发送问答时把 `conversation_file:{fileId}` 合并进 `metadata.context.contextRefs`。
- 商品选择仍只生成 `product:{id}` 和 `productIds`，不会从文件反向推断商品。
- 未勾选资料时，不自动把历史文件、商品素材或测试数据放入本轮上下文。

## 非目标

- 不新增后端 API。
- 不接 workspace artifact / product material / product asset 列表选择；这些需要后续基于真实 API 列表再接入。
- 不读取 object storage，不做 OCR、PDF、图片或 Office 解析。
- 不伪造“已读取正文”；不可读文件只显示为元数据来源。

## 前端落点

- `pages/site/agent-chat/domain/agent-chat-context-selection.ts`
  - 增加资料引用 selection 类型和 context ref 构造。
- `pages/site/agent-chat/view-model/use-agent-chat-source-context.ts`
  - 管理当前会话可选资料、已选资料、清空和恢复。
- `pages/site/agent-chat/components/`
  - 新增资料选择面板。
  - composer toolbar 增加紧凑“资料”按钮。
  - composer panels 增加与输入栏等宽的资料选择浮层。
- `pages/site/agent-chat/view-model/agent-chat-send-message.ts`
  - 发送时把已选资料 refs 合并进 `contextRefs`。

## 用户可感知变化

用户上传文件后，可以在输入栏工具区点“资料”查看当前会话已上传文件，并手动勾选哪些文件进入本轮回答。勾选后发送问题，后端会把这些文件作为显式来源处理；如果文件没有可读文本，界面会标明“仅元数据”，避免误导用户以为正文已经被读取。

## 验收

- `pnpm -C frontend/apps/web rules`
- `pnpm -C frontend/apps/web lint`
- `pnpm -C frontend/apps/web test`
- `pnpm -C frontend/apps/web build`
- `pnpm -C frontend rules`
- `git diff --check`
