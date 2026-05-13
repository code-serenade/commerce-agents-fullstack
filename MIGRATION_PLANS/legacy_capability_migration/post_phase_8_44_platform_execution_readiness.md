# Post 8-44：平台执行 Readiness 诊断契约

## 本轮结论

本轮把高风险平台写操作从“创建审批后才知道能不能执行”推进到“审批创建时就给出真实执行可行性诊断”。

当前仍不执行真实淘宝/JD/PDD/抖音写接口，也不伪造成功。它解决的是审批前的信息透明度：用户看到平台写操作审批时，可以先知道批准后是否会因为参数、连接或 adapter 问题失败。

## 涉及文件

- `temp/DEVELOPMENT_DOCS/backend/platform_execution_readiness_post_8_44.md`
- `backend/crates/service/src/api/approval_audit.rs`
- `frontend/apps/web/src/api/commerce.ts`
- `frontend/apps/web/src/pages/site/commerce/approvals/components/approval-panels.tsx`
- `frontend/apps/web/src/pages/site/commerce/workspaces/components/workspace-execution-columns.tsx`
- `frontend/apps/web/src/pages/site/commerce/components/commerce-status-utils.ts`
- `API_DOCS/app.md`
- `API_DOCS/sse.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`

## 后端变化

`ApprovalAuditApi::create_tool_approval` 现在会在创建平台写类审批时读取参数和平台连接状态，并把 `executionReadiness` 写入 `tool_approval_queue.result`。

当前诊断覆盖：

- `validation_error`：缺平台、商品、价格、库存等必要参数。
- `connection_missing`：当前用户没有对应平台连接。
- `connection_disabled`：连接存在但被禁用。
- `unsupported_action`：平台动作不在当前服务支持范围内。
- `adapter_not_implemented`：参数和连接存在，但真实平台写 adapter 还没实现。

审批通过后的执行规划也做了顺序修正：未知平台动作会优先报 `unsupported_action`，不会因为没有连接而误报 `connection_missing`。

## 前端变化

前端只消费后端真实字段：

- 审批页在审批原因下方展示 `executionReadiness.adapterStatus` 和 `message`。
- 工作区执行建议表在状态下方展示执行结果里的 `adapterStatus` 和 `message`。
- 状态色把 `adapter_not_implemented`、`connection_missing`、`connection_disabled`、`unsupported_action`、`validation_error` 显示为风险态。

没有新增假执行按钮，也没有把批准显示成执行成功。

## 用户可感知变化

用户在审批页看到平台写操作时，会直接看到当前为什么不能真实执行。例如：缺少平台连接、连接被禁用、参数不完整，或真实平台 adapter 尚未接入。

如果用户仍然批准，后端会继续走既有审批后执行闭环，并把真实失败原因写回审批记录和执行队列。用户在工作区执行建议里也能看到同一类失败原因。

## 当前边界

- 当前真实平台写 adapter 仍未实现。
- 当前不会返回 `ready`。
- 当前不新增数据库表，不新增第二套审批系统。
- 后续接入真实平台 adapter 时，应在现有 readiness 和执行闭环里补真实 adapter，而不是重做审批/审计链路。
