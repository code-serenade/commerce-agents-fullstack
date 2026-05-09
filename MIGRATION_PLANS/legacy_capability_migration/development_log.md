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

## 4/8：工作区沉淀与交付物

状态：done

本次目标：

- 让 Agent SSE 问答先绑定真实工作区运行记录。
- 把 task graph 节点沉淀为 `workspace_tasks` 和 `workspace_run_tasks`。
- 把 worker report、evaluation 和最终输出写回工作区运行数据。
- 把最终回答沉淀为可查询、可更新、可归档的 `workspace_artifacts.final_report`。

涉及范围：

- 产品内核：Agent session 从临时流式过程升级为可追踪的 workspace run，并形成独立交付物记录。
- 后端：新增 `workspace_artifacts` 表和 repo/service/web API；`WorkspaceApi` 新增 agent run/task/report/evaluation/artifact 持久化方法；SSE handler 把 agent 事件转为 service 层写入。
- API：更新 `API_DOCS/sse.md` 和 `API_DOCS/app.md`。
- 前端：同步 workspace SSE event 类型，在右侧运行态展示真实 workspace/run/task/artifact 沉淀状态。

用户可感知变化：

- 发送问答后，后端会自动创建或复用一个 `agent_chat` 工作区。
- 复杂问答产生任务图时，每个虚拟员工节点会落成可查询的工作区任务快照。
- 问答完成后，最终回答会同时保存到 `workspace_run.report.output` 和 `workspace_artifacts`，右侧面板会提示交付物已经生成。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo test -p web-server sse`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`

## 8/8：完整商业化体验

状态：done

本次目标：

- 让新产品入口更像“电商多智能体/虚拟员工”，而不是普通聊天。
- 后端提供商业场景 playbook，前端按后端契约展示，不再硬编码样例 prompt。
- 点击场景后进入已有问答、策略、RAG、工作区、质量和审批链路。

涉及范围：

- 产品内核：问答入口从前端样例升级为后端商业场景目录，场景自带推荐策略、所需上下文、预期产出和虚拟员工角色。
- 后端：新增 `CommercePlaybookApi`、`GET /api/app/agent/playbooks`、6 个内置电商场景和 service 单测。
- API：更新 `API_DOCS/app.md`，记录 playbook 字段、鉴权和当前内置场景。
- 前端：新增 playbook API/domain/hook，问答 composer 改为消费后端 playbook，点击场景自动填充任务并切换推荐策略。

用户可感知变化：

- 问答页输入区上方出现 6 个后端场景入口：新品冷启动、投放复盘、客诉危机、选品定价、素材优化、日报复盘。
- 用户点击任一场景后，输入框自动填入完整电商任务，不需要自己从零描述。
- 场景会自动选择合适回答策略，例如复杂协作类走 `collab`，解释判断类走 `learning`，危机/日报类走 `execution`。
- 商品上下文选择、RAG、工作区沉淀、质量检查、审批告警等前面已迁移链路继续保留。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `cargo test -p service commerce_playbook`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `git diff --check`

## 7/8：搜索/RAG/外部数据

状态：done

本次目标：
- 用新产品后端框架实现真实本地 RAG，而不是伪造外部搜索结果。
- 用户发送问题后，后端自动按当前用户检索 active 商品和 active 商品知识。
- 检索结果既注入 agent metadata，也通过 SSE 事件展示给前端。
- 外部搜索适配器未接入时，前端展示明确降级原因。

涉及范围：
- 产品内核：问答链路从“用户手动选择商品上下文”扩展为“问题自动触发本地知识检索”；RAG 摘要追加到 `context.summary`，agent runtime 无需改动即可使用。
- 后端：`product_knowledge` repo 增加当前用户全局知识检索；`AgentContextApi` 增加 `build_retrieval_context`；SSE 增加 `rag_context_loaded`，并写入 `rag.*` metadata。
- API：更新 `API_DOCS/sse.md` 和 `API_DOCS/app.md`，记录本地 RAG 来源、降级状态和外部搜索边界。
- 前端：`AgentRuntimeEvent` 增加 `rag_context_loaded`；时间线和右侧过程卡片展示知识检索来源、检索词、知识数量和降级原因。

用户可感知变化：

- 不手动选择商品时，用户直接提问也会触发后端本地知识检索。
- 命中商品知识时，右侧过程区会显示“知识检索”，并展示来源数量和部分来源标题。
- 没有命中本地知识时，右侧过程区会明确显示本地知识未命中且外部搜索适配器未接入，不会假装查到了外部资料。
- 模型回答会更容易围绕已维护的商品知识展开，而不是只依赖通用模型常识。

当前验证：
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

## 5/8：质量、信任、学习闭环

状态：done

本次目标：

- Agent SSE 回答完成后，由后端自动检查回答质量。
- 质量结果更新 agent 信任分，并沉淀为学习记录。
- 下一次问答前，后端读取学习记录和信任摘要并注入 agent runtime metadata。
- 前端只展示后端真实 `quality_checked` 事件，不伪造质量状态。

涉及范围：

- 产品内核：回答质量从一次性输出扩展为“检查 -> 信任分 -> 学习记录 -> 下次上下文”的闭环。
- 后端：新增 `agent_quality_checks`、`agent_trust_scores`、`agent_learning_records`；新增 repo/service/web API；SSE 自动质量检查和 `quality_checked` 事件。
- API：更新 `API_DOCS/app.md` 与 `API_DOCS/sse.md`。
- 前端：`AgentRuntimeEvent`、SSE 字段归一化、时间线、右侧工作区卡片、中英文文案。

用户可感知变化：

- 问答完成后，右侧过程区会看到“质量检查完成”事件。
- 工作区卡片会显示质量分、信任分和本轮学习记录数量。
- 如果质量不足，卡片会明确显示“需要改进”，并由后端沉淀改进经验。
- 用户反馈后端接口已具备，后续可以在前端接评分或反馈按钮。

当前验证：

- [x] `cargo check -p migration`
- [x] `cargo xtask migrate up`
- [x] `sea-orm-cli generate entity -o crates/repo/src/entity --with-serde both --date-time-crate time`
- [x] `cargo check -p repo`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `cargo test -p service`
- [x] `cargo test -p service quality_checker`
- [x] `cargo test -p service trust_ema`
- [x] `cargo test -p web-server sse`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] 前端 dev server HTTP smoke：`http://localhost:5173` 返回 200

## 6/8：审批、告警、平台写操作

状态：done

本次目标：

- 把 Agent 高风险平台写 skill 的 `pending_approval` 状态接入真实审批队列。
- 后端自动创建或复用审批、审计、告警记录。
- 前端只展示后端真实 `approval_required` 事件，不伪造审批状态。

涉及范围：

- 产品内核：高风险 skill result 从过程事件升级为治理入口，平台写操作默认阻断到审批队列。
- 后端：SSE handler 调用 `ApprovalAuditApi` 和 `BriefingOpsApi`，创建审批、审计和告警；新增 helper 和单元测试覆盖触发条件、风险等级映射、去重 key 长度。
- API：更新 `API_DOCS/sse.md` 与 `API_DOCS/app.md`。
- 前端：`AgentRuntimeEvent` 新增 `approval_required`；时间线和右侧工作区卡片展示审批 ID、风险等级、原因和告警消息。

用户可感知变化：

- 平台写操作不会显示成已经执行，而是显示“需要审批”。
- 问答页右侧会看到审批编号、能力名、审批状态和风险等级。
- 如果告警创建成功，右侧卡片会同步显示告警消息。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p web-server`
- [x] `cargo test -p web-server sse`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`

## Post 8-1：工作区交付物刷新可见

状态：done

本次目标：

- 把 Phase 4 已经沉淀到后端的 workspace artifact 接入完整工作区页面。
- 工作区页面刷新后仍能查看 Agent 交付物列表和详情。
- 前端只消费真实后端 artifact API，不展示假交付物。

涉及范围：

- 产品内核：问答交付物从“过程事件可见”补强为“业务工作区可沉淀、可刷新、可复用”。
- 后端：未新增接口，复用已有 workspace artifact 查询和详情接口。
- 前端：新增 artifact API client、查询状态、工作区“交付物”tab、交付物列表、详情预览和指标卡。
- 文档：更新 Phase 4 验收记录、前端体验检查清单，并新增本次补强归档。

用户可感知变化：

- 用户进入“执行工作区”后，选中工作区可以打开“交付物”tab。
- 问答生成的 final report 等交付物，刷新页面后仍能在工作区里看到。
- 点击交付物后，右侧能看到标题、类型、来源、校验状态、格式、存储、文件信息、运行 ID 和正文/预览。

当前验证：

- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`
- [x] 测试账号 `test123456` 登录后真实 HTTP smoke：创建临时工作区、创建 `final_report` artifact、查询列表、查询详情均通过；临时工作区已归档清理。

运行注意：

- smoke 前本地 `19878` 上运行的是旧后端进程，对 artifact 路由返回 404。
- 重启本地后端并加载当前代码后，artifact 路由正常。

## Post 8-2：问答运行反馈前端接入

状态：done

本次目标：

- 把 Phase 5 已经完成的用户反馈后端接口接入问答工作台。
- 用户可以对真实 workspace run 评分并提交文字反馈。
- 前端只在后端返回真实运行记录后开放提交，不做假状态。

涉及范围：

- 产品内核：质量闭环补强为“自动质量检查 + 用户反馈学习记录”的双输入闭环。
- 后端：未新增接口，复用 `POST /api/app/workspaces/{workspaceId}/runs/{runId}/feedback`。
- 前端：新增反馈 API client、运行目标推导、反馈 hook、右侧工作台反馈面板和中英文文案。
- 文档：新增 Post 8-2 归档，更新 Phase 5 验收和前端体验清单。

用户可感知变化：

- 问答形成后端运行记录后，右侧虚拟员工工作区出现“本轮反馈”。
- 用户可以点 1 到 5 分星级，并填写可选反馈说明。
- 提交成功后，页面显示“已提交”，反馈写入后端学习记录。
- 未登录、生成中、无运行记录或已提交时，按钮会明确禁用并说明原因。

当前验证：

- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`
- [x] 测试账号真实 HTTP smoke：创建临时 workspace/run，提交反馈返回 `feedback_positive` learning record，临时 workspace 已归档。

测试注意：

- 冒烟测试确认 workspace run 状态应使用 `succeeded`，不能使用 `completed`。
- 后端反馈响应中的学习记录嵌套字段可能是 snake_case，前端 API client 已归一化。
