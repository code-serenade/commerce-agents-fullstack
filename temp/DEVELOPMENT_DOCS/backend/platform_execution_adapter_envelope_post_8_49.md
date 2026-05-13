# Post 8-49：平台执行 Adapter 请求信封

## 1. 业务理解

平台执行现在已经具备：

- 连接状态。
- 高风险动作规格注册表。
- 创建审批前 preflight。
- 审批创建 readiness。
- 审批通过后执行队列失败记录。

但在真实平台 adapter 接入前，执行队列失败记录还缺少一个关键中间层：后端没有明确沉淀“如果 adapter 已实现，本次会以什么平台、什么 skill、什么参数、什么幂等键去调用 adapter”。

这会影响后续真实 adapter 接入，因为后续需要一个稳定请求信封来保证：

- 审批通过后的执行请求可审计。
- 幂等键可复用。
- 失败和重试能定位到同一个执行意图。
- 前端和审计日志能看到清晰的执行边界。

本轮目标是在 service 内部共享模块里补齐平台执行 adapter request envelope。当前仍不调用真实平台写接口，不伪造执行成功。

## 2. 数据库设计

不新增表，不新增 migration。

本轮将 request envelope 写入既有 JSON 字段：

- `execution_queue.result.adapterRequest`
- `tool_approval.result.adapterRequest`
- audit metadata 的 `result`

## 3. Service 设计

继续使用：

```text
backend/crates/service/src/api/platform_execution.rs
```

新增结构和函数：

- `PlatformExecutionAdapterRequest`
- `platform_execution_adapter_request(skill_name, params)`

request envelope 字段：

- `skillName`
- `platform`
- `adapterStatus`
- `executionMode`
- `idempotencyKey`
- `idempotencyKeyFields`
- `args`

其中 `idempotencyKey` 基于共享 action spec 的 `idempotency_key_fields` 生成。字段值来自已经规范化后的 `PlatformExecutionParams`。

`approval_audit` 的平台执行失败结果将增加 `adapterRequest` 字段。即使当前 adapter 未实现，也能清楚记录本次审批通过后准备执行的 adapter 请求。

## 4. HTTP/API 设计

不新增 HTTP API。

既有接口响应结构保持兼容，只在平台执行失败结果 JSON 中新增可选字段：

- `adapterRequest`

需要同步说明到：

- `API_DOCS/app.md`

## 5. 权限与边界

- 鉴权和当前用户归属仍由 `approval_audit` 调用方负责。
- 共享模块不读数据库。
- 不调用淘宝/JD/PDD/抖音等真实平台写接口。
- 当前真实 adapter 未实现，执行队列仍返回 failed / `adapter_not_implemented`。
- 不新增前端按钮，不展示假成功。

## 6. 开发顺序

1. 新增 adapter request envelope 结构和生成函数。
2. 将平台执行失败结果补充 `adapterRequest`。
3. 补充单测覆盖幂等键和失败结果字段。
4. 更新 API 说明和迁移归档。
5. 运行 cargo 和 pnpm 门禁。
