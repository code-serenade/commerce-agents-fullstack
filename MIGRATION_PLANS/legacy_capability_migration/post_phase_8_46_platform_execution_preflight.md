# Post 8-46：平台执行 Preflight 预检契约

## 本轮结论

本轮把高风险平台写操作从“创建审批后看到 readiness”继续前移到“创建审批前即可预检”。

当前仍不执行真实淘宝/JD/PDD/抖音写接口，也不伪造成功。它解决的是审批前表单、Agent 执行建议和后续平台 adapter 接入的统一校验入口：同一份参数、连接和 adapter 状态判断可以在创建审批前先返回给用户。

## 涉及文件

- `temp/DEVELOPMENT_DOCS/backend/platform_execution_preflight_post_8_46.md`
- `backend/crates/service/src/dto/platform_connect.rs`
- `backend/crates/service/src/api/platform_connect.rs`
- `backend/crates/web-server/src/dto/app.rs`
- `backend/crates/web-server/src/handlers/app/platform_connect.rs`
- `backend/crates/web-server/src/routes/app/platform_connect.rs`
- `frontend/apps/web/src/api/commerce.ts`
- `API_DOCS/app.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `MIGRATION_PLANS/legacy_capability_migration/api_migration_checklist.md`

## 后端变化

新增 `PlatformConnectApi::execution_preflight` 和 HTTP 接口：

```text
POST /api/app/platform-connect/execution-preflight
```

请求包含：

- `skillName`
- `args`

参数解析兼容：

- 顶层字段。
- `normalized_inputs` / `normalizedInputs`。
- `output.normalized_inputs` / `output.normalizedInputs`。

预检结果覆盖：

- `validation_error`：缺平台、商品、价格或库存等必要参数。
- `connection_missing`：当前用户没有对应平台连接。
- `connection_disabled`：平台连接存在但被禁用。
- `unsupported_action`：不是平台执行动作，或当前服务不支持该平台动作。
- `adapter_not_implemented`：参数和连接齐全，但真实平台写 adapter 尚未实现。

## 前端变化

`apps/web` 已同步 API 类型和请求函数：

- `PlatformExecutionPreflightRequest`
- `PlatformExecutionPreflightResponse`
- `platformExecutionPreflightApi`

本轮不新增页面按钮或临时表单，避免制造“可以执行平台写操作”的假入口。

## 用户可感知变化

后续在创建平台写审批前，前端或 Agent 可以先调用 preflight，直接告诉用户当前动作为什么不能执行。

如果缺参数、缺连接、连接被禁用或 adapter 未实现，用户可以在审批创建前看到原因，不必等到审批之后才发现失败。

## 当前边界

- 当前真实平台写 adapter 仍未实现。
- 当前不会返回 `canExecuteNow=true`。
- 当前不创建审批，不创建执行队列，不写平台执行审计日志。
- 后续接入真实平台 adapter 时，应复用该 preflight、审批 readiness、adapter 目录和执行队列闭环。
