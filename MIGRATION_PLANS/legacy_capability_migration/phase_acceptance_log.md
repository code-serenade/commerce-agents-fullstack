# 阶段验收记录

## 总门禁

每个阶段必须先通过后端门禁，再进入前端适配。

后端门禁：

- [ ] 后端能力边界确认完成。
- [ ] 涉及 API、DB、SSE、权限、错误码或 LLM 调用时，后端设计文档完成。
- [ ] Rust agent/service/web-server 实现完成。
- [ ] `API_DOCS/` 已同步。
- [ ] `cargo fmt`、`cargo check`、`cargo clippy`、相关测试或 smoke 已通过。
- [ ] 真实 SSE/HTTP 请求验证完成。

前端适配门禁：

- [ ] 对应后端 API/SSE 契约已稳定。
- [ ] 前端只消费真实后端数据和事件。
- [ ] 未接入的后端能力不做假流程展示。
- [ ] `pnpm rules` 和相关前端验证已通过。

## Phase 0：迁移台账与能力核对

状态：pending

验收项：

- [ ] 旧 46 skill 全部映射完成。
- [ ] 新 93 staged spec 核对完成。
- [ ] 每个 skill 标记 `native_ready` / `adapter_required` / `approval_required` / `missing`。
- [ ] 角色矩阵核对完成。

## Phase 1：问答策略与路由增强

状态：done

验收项：

- [x] 后端：execution/learning/collab 策略进入 request metadata。
- [x] 后端：Commander 能按策略影响路由。
- [x] 后端：SSE 输出真实 route decision / strategy applied。
- [x] 简单问答走轻流程。
- [x] 复杂任务触发多角色。
- [x] 前端：策略切换只展示后端真实行为差异。

## Phase 2：Skill 执行与 SSE 可视化

状态：done

验收项：

- [x] 后端：`skill_requested` 事件。
- [x] 后端：`skill_result` 事件。
- [x] 后端：失败、deferred、approval_required 状态结构化。
- [x] 后端：至少 6 个代表 skill 可真实执行或明确 deferred。
- [x] 前端：运行详情卡和时间线接真实 skill 事件。

## Phase 3：产品知识与上下文绑定

状态：done

验收项：

- [x] 后端：SSE request 携带 context refs。
- [x] 后端：读取真实产品知识。
- [x] 后端：权限、空数据和超长上下文降级规则明确。
- [x] 回答引用真实产品信息。
- [x] 前端：可选择产品上下文并展示引用来源。

## Phase 4：工作区沉淀与交付物

状态：pending

验收项：

- [ ] 后端：Agent run 关联 workspace run。
- [ ] 后端：Worker report 可沉淀任务。
- [ ] 后端：artifact 创建、查询、权限闭环。
- [ ] 前端：交付物可刷新后查看。

## Phase 5：质量、信任、学习闭环

状态：pending

验收项：

- [ ] 后端：质量评分结构可用。
- [ ] 后端：用户反馈可保存。
- [ ] 后端：trust/learning 可影响后续回答。
- [ ] 前端：质量评分和反馈入口展示真实数据。

## Phase 6：审批、告警、平台写操作

状态：pending

验收项：

- [ ] 后端：平台写操作默认 pending approval。
- [ ] 后端：审批记录可查。
- [ ] 后端：告警可推送。
- [ ] 前端：审批和告警只展示后端真实记录。

## Phase 7：搜索/RAG/外部数据

状态：pending

验收项：

- [ ] 后端：搜索结果有来源。
- [ ] 后端：产品知识可检索。
- [ ] 后端：外部不可用可降级。
- [ ] 前端：来源和降级原因可见。

## Phase 8：完整商业化体验

状态：pending

验收项：

- [ ] 新品冷启动场景全通。
- [ ] 投放复盘场景全通。
- [ ] 客诉危机场景全通。
- [ ] 选品定价场景全通。
- [ ] 素材优化场景全通。
- [ ] 日报复盘场景全通。
