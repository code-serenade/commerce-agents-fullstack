# 3/8 产品知识与上下文绑定归档

## 状态

done

## 本次迁移目标

把旧产品“商品知识进入问答上下文”的能力迁移到新产品主问答链路中。实现后，用户可以在前端选择后端已有的 active 商品，SSE 请求携带商品引用，后端按当前用户权限读取商品和商品知识，再把压缩摘要注入 agent runtime。

## 产品内核底层变化

- `SessionEvent` 增加 `context_loaded`，让业务上下文加载成为可观察 runtime 事件。
- commander cognition 和 worker cognition 都会收到 `commerce.product_context` fact。
- 商品上下文不由前端拼文本伪造，而是由后端 service 读取真实数据后进入 agent metadata。

## 后端变化

- 新增 `backend/crates/service/src/api/agent_context.rs`。
- 新增 `backend/crates/service/src/dto/agent_context.rs`。
- SSE DTO 支持 `metadata.context.productIds` 和 `metadata.context.contextRefs`。
- SSE handler 在启动 agent 前调用 service 聚合器，读取当前用户自己的 active 商品和 active 商品知识。
- agent runtime 根据 `context.*` metadata 发出 `context_loaded` 事件。
- 更新 `API_DOCS/sse.md`。

## 前端变化

- agent-chat composer 读取 `/api/app/products` 的 active 商品列表。
- 用户可点选商品 chip，发送时写入 `metadata.context.productIds/contextRefs`。
- 前端 runtime event 类型增加 `context_loaded`。
- 右侧过程面板和时间线展示后端真实上下文加载事件。

## 用户可感知变化

登录后，问答输入区会出现“后端商品上下文”商品 chip。选中商品后再提问，后端会读取该商品及商品知识并注入回答上下文；右侧过程面板会显示已加载多少商品和知识。未选择商品时，系统不会假装知道商品详情。

## 验证记录

- `cargo fmt`：通过。
- `cargo test -p service agent_context`：3 passed。
- `cargo test -p web-server sse`：2 passed。
- `cargo test -p agent --test default_roles context`：3 passed。
- `cargo check -p service`：通过。
- `cargo check -p agent`：通过。
- `cargo check -p web-server`：通过。
- `cargo check`：通过。
- `cargo clippy`：通过。
- `pnpm -C frontend/apps/web rules`：通过。
- `pnpm -C frontend/apps/web lint`：通过。
- `pnpm -C frontend/apps/web test`：通过，无测试文件。
- `pnpm -C frontend/apps/web build`：通过。
- `POST http://127.0.0.1:3000/api/sse/agent/session/stream` 未带 JWT 返回 401：鉴权路由在线。

## 边界与后续

- 当前只加载商品基础信息和 active 商品知识，不做向量检索。
- 当前支持 `product:{productId}` context ref；campaign、workspace、asset 等 ref 留到后续阶段。
- 完整带 JWT 的浏览器 SSE 问答需要使用前端登录态进行人工验证。
