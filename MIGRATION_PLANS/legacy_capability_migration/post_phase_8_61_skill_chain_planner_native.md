# Post 8-61：技能链规划 skill native 补强

状态：done

## 本次目标

- 继续按“旧产品能力 = 后端内核能力来源，新产品架构 = 唯一落地框架”迁移协同类能力。
- 把旧产品 `coordination_skill_chain_planner` 的核心价值迁入新产品 Rust Agent skill。
- 让后端返回可供 Kimi Agent 风格过程面板消费的结构化技能链，而不是只返回几行静态建议。
- 不做外部 Agent 调用，不做平台写操作，不伪造 adapter 执行结果。

## 本次修改

- 后端 agent：`coordination_skill_chain_planner` 输出新增 `status=success`、`角色`、`动作`、`前置依赖缺口`、`不可用技能`、`审批提示`、`adapter提示` 和 `可用技能过滤`。
- 后端 agent：每个推荐步骤新增 `阶段`、`负责角色`、`执行模式`、`风险等级`、`前置依赖`、`可用` 和状态说明。
- 后端 agent：把旧产品常见的 `data_query_store_metrics` 规范到新产品真实 skill ID `query_store_metrics`，避免推荐不存在的旧 ID。
- 后端 agent：补强 NPS、SLA、CTR、ABC-XYZ、广告疲劳、RFM、SEO、财务、预测、库存、大促、新品、数据、客服、创意等常见电商场景模板。
- 后端 agent：`skills_available` 会被用于标记推荐技能是否可用，但不会伪造执行。
- API/SSE：同步 `API_DOCS/sse.md`，说明该 skill 仍通过既有 `skill_result` 事件返回结构化规划。
- 迁移文档：同步迁移矩阵、开发记录、阶段验收记录和总计划。

## 用户可感知变化

- 后续前端接入后，用户问“这个任务应该让哪些虚拟员工和 skill 先后处理”时，右侧过程区可以展示更清楚的执行链。
- 用户能看到每个步骤是数据加载、分析、运营规划、内容生成、审批执行还是外部 adapter 依赖。
- 如果某个步骤需要外部数据或审批，系统会明示，不会把它展示成已经完成。
- 如果前端只开放了部分 skill，后端会标记哪些推荐技能当前不可用，避免用户误以为按钮可直接执行。

## 当前验证

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/skill_chain_planner_native_post_8_61.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt --all`
- [x] `cargo test -p agent coordination_skill_chain -- --nocapture`

## 后续缺口

- 前端仍需把结构化技能链接入右侧过程面板，做成可折叠、可展开、可追踪的 Kimi Agent 风格任务过程。
- 当前 skill 链只做 deterministic planning，不自动批量执行链路；未来如做自动链式执行，必须继续走 `SkillExecutorSet`、SSE、审批和审计。
- `coordination_external_agent_call` 仍保持审批/安全边界，真实 A2A 外部调用需要白名单、鉴权、审计和失败降级后再接入。
