# 7/8：搜索、RAG 与外部数据

状态：done

## 本次目标

- 先用新产品框架实现真实可用的本地 RAG，不伪造外部搜索。
- 用户发送问题后，后端根据问题检索当前用户自己的商品和商品知识。
- 检索结果注入 agent metadata，让回答可引用真实后端知识。
- 前端展示检索来源、知识数量和降级原因。

## 产品内核底层变化

- 问答链路从“只接收用户显式选择的商品上下文”扩展为“用户问题自动触发本地知识检索”。
- agent runtime 不需要改动即可使用 RAG：web-server 会把检索摘要追加到 `context.summary`，同时保留独立的 `rag.*` metadata。
- 外部搜索仍保持真实边界：没有外部搜索适配器时，不生成假搜索结果，只返回明确降级状态。

## 后端变化

- `product_knowledge` repo 增加按当前用户全局检索知识的方法，仍保持单表查询原则。
- `AgentContextApi` 增加 `build_retrieval_context`：
  - 提取电商关键词。
  - 搜索当前用户 active 商品。
  - 搜索当前用户 active 商品知识。
  - 校验商品归属和 active 状态。
  - 生成可注入 LLM 的本地 RAG 摘要和来源列表。
- SSE handler 在 agent 启动前执行检索，并发出 `rag_context_loaded` 事件。
- API 文档补充 `rag_context_loaded` 契约。

## 前端变化

- `AgentRuntimeEvent` 增加 `rag_context_loaded`。
- 事件归一化支持 `queryTerms`、`sourceCount`、`degradedReason` 等字段。
- 时间线展示知识检索完成或检索降级。
- 右侧过程卡片优先展示 RAG 检索状态、来源标题、检索词和降级原因。

## 用户可感知变化

- 用户不手动选择商品时，直接提问也会触发后端本地知识检索。
- 如果命中商品知识，右侧过程区会显示“知识检索”，并列出来源数量和部分来源标题。
- 如果没有命中本地知识，右侧过程区会明确显示“本地知识未命中，外部搜索适配器未接入”，不会假装查到了外部资料。
- 模型回答会更容易围绕已维护的商品知识展开，而不是只依赖通用模型常识。

## 验证记录

本阶段验证：

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo test -p service agent_context`
- [x] `cargo check -p web-server`
- [x] `cargo test -p web-server sse`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
