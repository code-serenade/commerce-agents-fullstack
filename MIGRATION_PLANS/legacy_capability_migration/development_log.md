# 迁移开发记录

## 1/8：问答策略 metadata 与后端路由事件

状态：done

本次目标：

- 把回答策略从前端 prompt 拼接迁移为后端结构化 metadata。
- 后端 SSE 输出 `route_decided`，让前端展示真实策略和路由结果。
- `collab` 策略优先触发多角色任务图。

涉及范围：

- 产品内核：agent request/context 新增 metadata 通道，策略成为 runtime 输入契约。
- 后端：`AgentRequest`、`MessageContext`、`SessionEvent`、session runtime、commander routing、SSE DTO。
- API：更新 `API_DOCS/sse.md`。
- 前端：stream request 发送 `metadata.strategy`，时间线消费 `route_decided`。

用户可感知变化：

- 切换回答策略后，不再只是把策略文字塞进问题里，而是由后端识别并进入真实路由。
- 问答过程面板会出现“后端已采用某策略、路由到某主角色”的真实事件。
- 选择协同策略时，更稳定进入多虚拟员工任务图。

验证：

- [x] `cargo check -p agent`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] agent strategy routing test
- [x] frontend rules/lint/build

## 2/8：Skill 执行与 SSE 可视化

状态：done

本次目标：

- 后端 worker 执行 skill 前后输出真实 SSE 事件。
- 前端过程面板消费 `skill_requested` / `skill_result`，展示工具调用状态。
- 保持 skill 执行结果进入 worker report 的既有链路，不用前端伪造工具状态。

涉及范围：

- 产品内核：skill 调用从“只写入 worker 内存”提升为 agent runtime 可观察事件。
- 后端：`SessionEvent`、worker runtime、session runtime worker 构造、agent stream 测试。
- API：更新 `API_DOCS/sse.md`。
- 前端：`AgentRuntimeEvent` 类型、字段兼容、时间线和运行详情卡。

用户可感知变化：

- 后端真正调用 skill 时，右侧过程面板会看到“能力调用”和返回状态。
- success、deferred、pending_approval、failed 等状态不会被前端伪造，全部来自 SSE。
- 高风险或平台写操作会展示需要审批，不会让用户误以为已经自动执行。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p agent`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `cargo test -p agent --test default_roles stream_emits_skill_execution_events`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web build`

## 3/8：产品知识与上下文绑定

状态：done

本次目标：

- 让问答请求可以绑定真实商品上下文。
- 后端按当前用户权限读取商品和商品知识，并把摘要注入 agent runtime。
- 前端从商品 API 读取 active 商品，用户可选择商品 chip 后随 SSE 请求传入。

涉及范围：

- 产品内核：agent runtime 新增 `context_loaded` 事件，commander/worker cognition 都能收到商品上下文 facts。
- 后端：新增 `service::api::agent_context` 聚合产品与知识；SSE DTO 支持 `metadata.context.productIds/contextRefs`；web-server 在启动 agent 前装配上下文 metadata。
- API：更新 `API_DOCS/sse.md`。
- 前端：agent-chat composer 显示后端商品上下文选择；请求发送真实 `metadata.context`；过程面板展示后端 `context_loaded` 事件和上下文卡片。

用户可感知变化：

- 登录后，问答输入框上方会显示当前后端 active 商品，可点选作为本轮问答上下文。
- 发送问题后，右侧过程面板会出现“已从后端读取商品/知识”的真实事件。
- 绑定商品后，模型拿到的是后端读取的商品名称、平台、类目、品牌、价格、描述和商品知识摘要；未绑定时不会假装知道商品详情。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service agent_context`
- [x] `cargo test -p web-server sse`
- [x] `cargo test -p agent --test default_roles context`
- [x] `cargo check -p service`
- [x] `cargo check -p agent`
- [x] `cargo check -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web build`
