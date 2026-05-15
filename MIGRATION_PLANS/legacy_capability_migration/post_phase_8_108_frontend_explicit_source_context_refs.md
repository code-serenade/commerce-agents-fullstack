# Post 8-108：前端显式资料引用接入

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 108 个补强项：`Post 8-108`。

本次目标：

- 把 Post 8-107 后端新增的 `conversation_file:{uuid}` 显式上下文引用接入 Agent Chat 前端。
- 让当前会话已上传文件不再只是展示，而是可由用户手动勾选进入本轮问答上下文。
- 继续保持“未勾选不注入”的防串线规则，不自动读取历史商品、历史附件或测试资料。

本次修改：

- 前端 domain：新增 `AgentChatSourceContextSelection`、附件到资料引用的映射、资料引用切换和 `contextRefs` 合并逻辑。
- 前端 view-model：新增 `useAgentChatSourceContext`，按当前会话已上传文件派生可选资料，并清理失效选择。
- 前端 composer：新增紧凑“资料”按钮和资料选择浮层，浮层与输入栏等宽。
- 前端发送链路：`metadata.context.contextRefs` 现在合并商品引用和用户手动勾选的 `conversation_file:{fileId}`。
- 本地化：补充中英文资料引用文案。

用户可感知变化：

- 用户上传文件后，可以在输入栏工具区点击“资料”，看到当前会话已上传文件。
- 用户勾选某个文件后再提问，后端会把该文件作为本轮显式上下文来源。
- 如果文件只有元数据、没有可读正文，界面会标明“仅元数据”，不会误导用户以为系统已经读取了正文。
- 未勾选资料时，本轮问答不会自动混入历史文件、历史商品资料或千乐咖啡等测试内容。

当前验证：

- [x] 前端设计文档：`temp/DEVELOPMENT_DOCS/frontend/explicit_workspace_source_context_refs_post_8_108.md`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

后续缺口：

- 还需要继续接入工作区交付物、商品素材、商品资产的显式选择器。
- 还需要在历史任务/交付物详情中提供“引用此资料继续追问”的入口。
- 附件 OCR、PDF/Office/图片解析和后台 parser worker 仍属于后续 adapter 能力。
