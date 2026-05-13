# Post 8-44 平台执行 Readiness 诊断契约

## 需求摘要

继续按旧产品能力迁移计划开发，后端优先，前端只适配真实后端状态。本轮聚焦“平台写操作审批后执行”的闭环前置诊断：用户在看到工具审批请求时，应能知道这次平台动作是否具备真实执行条件，例如是否缺平台、缺商品参数、缺平台连接、连接被禁用，或当前平台 adapter 尚未实现。

## 业务边界

- 不新增第二套审批系统。
- 不绕过现有 `tool_approval_queue`、`execution_queue`、`audit_logs`。
- 不伪造平台执行成功。
- 不直接调用淘宝、京东、拼多多、抖音等真实平台写接口。
- 保留现有审批通过后创建执行队列并写入真实失败结果的逻辑。

## 数据表设计

本轮不新增表，不新增 migration。

复用已有表：

- `tool_approval_queue.result`：在审批创建阶段写入 `executionReadiness` 诊断。
- `execution_queue.result`：审批通过后的真实执行尝试结果继续写入这里。
- `audit_logs.metadata`：审批和执行过程继续写入审计上下文。
- `platform_connections`：只读取当前用户的平台连接状态，用于 readiness 判断。

## 后端服务设计

修改 `ApprovalAuditApi::create_tool_approval`：

1. 解析 `skill_name` 和 `args`。
2. 对平台写类 skill 生成 `executionReadiness`。
3. 创建审批记录时把 readiness 写入 `result`。
4. 如果平台连接缺失、连接禁用、参数缺失或 adapter 未实现，只记录真实诊断，不阻断审批记录创建。

`executionReadiness` 结构：

```json
{
  "status": "blocked",
  "adapterStatus": "adapter_not_implemented",
  "canApprove": false,
  "canExecuteNow": false,
  "willFailIfApproved": true,
  "platform": "taobao",
  "skillName": "platform_update_price",
  "message": "Platform execution adapter is not implemented yet for price update.",
  "details": {
    "productId": "prod-1",
    "skuId": null,
    "newPrice": 99,
    "quantity": null,
    "page": null,
    "pageSize": null
  },
  "checkedAt": "2026-05-13T00:00:00Z"
}
```

状态语义：

- `ready`：预留给后续真实平台 adapter 接入后使用；当前版本不会返回 ready。
- `blocked`：当前批准后会失败，原因见 `adapterStatus` 和 `message`。
- `unsupported`：当前 skill 不是受支持的平台执行动作。

`adapterStatus` 语义：

- `validation_error`：必要参数缺失或非法。
- `connection_missing`：当前用户没有对应平台连接。
- `connection_disabled`：连接存在但未启用。
- `adapter_not_implemented`：连接和参数存在，但真实平台执行 adapter 仍未实现。
- `unsupported_action`：该审批动作不是当前平台执行服务支持的动作。

## API 设计

不新增路由。

变更现有返回结构：

- `GET /api/app/approvals/tool-requests`
- `GET /api/app/approvals/tool-requests/{approvalId}`
- `POST /api/app/approvals/tool-requests`
- Agent SSE 自动创建的 `approval_required` 事件

`ToolApprovalResponse.result` 中会包含 `executionReadiness`。

审批通过后的 `ExecutionQueueResponse.result` 和 `ToolApprovalResponse.result` 会继续写入真实执行尝试结果，字段包括：

- `adapterStatus`
- `message`
- `details`
- `manualResult`
- `attemptedAt`

鉴权不变：所有接口继续使用当前 App 用户 JWT。

## 前端适配设计

前端只消费后端返回的真实字段：

- 审批列表在原因下方展示 readiness 的 `adapterStatus` 和 `message`。
- 执行队列表格在描述下方展示 `result.adapterStatus` 和 `result.message`。
- 不新增假执行按钮，不新增假成功状态。

## 验证计划

后端：

- `cargo fmt`
- `cargo test -p service approval_execution_readiness -- --nocapture`
- `cargo test -p service plan_platform_execution -- --nocapture`
- `cargo check -p service -p web-server`
- `cargo clippy -p service -p web-server`

前端：

- `pnpm -C frontend/apps/web rules`
- `pnpm -C frontend/apps/web lint`
- `pnpm -C frontend/apps/web test`
- `pnpm -C frontend/apps/web build`
- `pnpm -C frontend rules`

## 用户可感知变化

用户在审批页看到高风险平台写操作时，不再只能看到“待审批”。现在能提前看到这次动作为什么不能真实执行：缺平台连接、连接禁用、缺必要参数，或平台 adapter 尚未接入。批准后如果仍然执行失败，执行队列和审批记录仍会写入真实失败结果，不会显示假成功。
