# Phase 6：审批、告警、平台写操作

状态：done

## 本次迁移目标

把旧产品里高风险工具、平台写操作、人工审批和告警提醒的能力，迁移到新产品的 Agent SSE + 业务治理框架内。本阶段以后端真实审批、审计、告警记录为准，前端不伪造“等待审批”状态。

## 产品内核底层变化

- Agent runtime 的 `skill_result` 不再只是过程展示事件。
- 当 skill 返回 `pending_approval` 或 `approval_required=true`，web-server 会把本次平台写操作转成可追踪的治理记录。
- 平台写操作默认停在审批队列，不会由 Agent 自动执行。
- 同一次 run、skill、request 会使用确定性 key 去重，避免 SSE 重试造成重复审批和重复告警。

## 后端变化

- 复用既有 `tool_approval_queue`、`audit_logs`、`alerts`，未新增数据库表。
- `backend/crates/web-server/src/handlers/sse.rs` 新增高风险 skill 结果处理：
  - 创建或复用工具审批记录。
  - 写入 `tool_approval.create` 审计日志。
  - 创建或复用 `agent_approval_required` 告警。
  - 发出 `approval_required` SSE 事件。
- Agent 风险等级映射到业务审批等级：
  - `low` -> `L1`
  - `medium` 或缺省 -> `L2`
  - `high` -> `L3`
- 审批失败会发 `workspace_run_persistence_error:create_tool_approval`。
- 审计或告警失败不会中断回答，会分别发 `create_tool_approval_audit`、`create_approval_alert`。
- 新增后端设计文档：`temp/DEVELOPMENT_DOCS/backend/agent_approval_alert_platform_phase6.md`。

## 前端变化

- `AgentRuntimeEvent` 新增 `approval_required` 类型。
- SSE runtime event set 已允许 `approval_required`。
- 事件归一化支持 `skillId`、`requestId`、`riskLevel`。
- 问答右侧时间线展示“审批已创建：某能力，风险等级”。
- 右侧工作区卡片展示后端返回的审批 ID、审批状态、能力名、原因和告警消息。
- 中英文文案已补齐。

## 用户体验变化

- 用户触发高风险或平台写操作时，前端会看到真实审批卡片。
- 卡片明确说明该操作已进入后端审批队列，不会自动执行。
- 用户可以拿卡片里的审批 ID 去审批页面继续处理。
- 如果告警创建成功，用户能看到后端告警消息；如果告警失败，审批记录仍会显示，错误会进入沉淀异常事件。

## 用户可感知变化

用户在问答页让虚拟员工做平台写操作，例如调价、改库存、同步商品时：

- 右侧时间线出现“审批已创建”。
- 工作区卡片显示“需要审批”。
- 卡片显示能力名、审批编号、审批状态和风险等级。
- 操作不会被误认为已经执行，必须进入审批流程后再处理。

## API 文档

- 已更新 `API_DOCS/sse.md`：新增 `approval_required` 事件、风险等级映射和失败 action。
- 已更新 `API_DOCS/app.md`：说明 Agent SSE 可自动创建工具审批、审计和告警。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo check -p web-server`
- [x] `cargo test -p web-server sse`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
