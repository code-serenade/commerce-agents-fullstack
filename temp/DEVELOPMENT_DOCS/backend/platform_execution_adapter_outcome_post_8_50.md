# Post 8-50：平台执行 Adapter Outcome 契约

## 1. 业务理解

平台执行闭环已经具备：

- 高风险动作规格注册表。
- 创建审批前 preflight。
- 审批 readiness。
- 审批通过后的 execution queue。
- adapter request envelope 和幂等键。

当前仍未接入真实平台写 adapter。审批通过后会明确失败，并返回 `adapter_not_implemented`。但失败结果仍由 `approval_audit` 本地拼装，缺少一个可复用的 adapter outcome 契约。

后续真实 adapter 接入时，执行器应统一返回 outcome，再由审批闭环写入 execution queue、tool approval 和 audit。否则成功/失败/降级结果容易继续分散在多个模块里。

本轮目标是新增 service 内部共享的 adapter outcome 契约，并先用它承载当前 `adapter_not_implemented` 结果。

## 2. 数据库设计

不新增表，不新增 migration。

本轮继续写入既有 JSON 字段：

- `execution_queue.result`
- `tool_approval.result`
- audit metadata 的 `result`

## 3. Service 设计

继续使用：

```text
backend/crates/service/src/api/platform_execution.rs
```

新增：

- `PlatformExecutionAdapterOutcome`
- `platform_execution_unimplemented_outcome(request, message)`

Outcome 字段：

- `status`
- `adapterStatus`
- `message`
- `adapterRequest`
- `providerResult`
- `retryable`

当前实现只生成 failed / `adapter_not_implemented` outcome，不调用真实平台 API。后续真实 adapter 接入时，可以扩展为 executed / failed / degraded 等结果。

`approval_audit` 中的 `platform_execution_failed` 改为接收 outcome，从 outcome 生成 result JSON。

## 4. HTTP/API 设计

不新增 HTTP API。

既有接口响应结构保持兼容，只在平台执行失败结果 JSON 中新增可选字段：

- `providerResult`
- `retryable`

并继续保留：

- `adapterRequest`
- `adapterStatus`
- `message`

需要同步说明到：

- `API_DOCS/app.md`

## 5. 权限与边界

- 鉴权和当前用户归属仍由 `approval_audit` 调用方负责。
- 共享模块不读数据库。
- 不调用淘宝/JD/PDD/抖音等真实平台写接口。
- 当前真实 adapter 未实现，执行队列仍返回 failed / `adapter_not_implemented`。
- 不新增前端按钮，不展示假成功。

## 6. 开发顺序

1. 新增 adapter outcome 结构和 unimplemented outcome 生成函数。
2. 改造 `approval_audit` 的平台执行失败结果使用 outcome。
3. 补充单测覆盖 `retryable=false`、`providerResult=null` 和 adapter request。
4. 更新 API 说明和迁移归档。
5. 运行 cargo 和 pnpm 门禁。
