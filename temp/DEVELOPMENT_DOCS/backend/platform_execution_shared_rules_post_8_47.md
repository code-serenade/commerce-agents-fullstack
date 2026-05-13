# Post 8-47：平台执行共享规则内核

## 1. 业务理解

平台执行治理现在已经有三处后端能力：

- 平台执行 adapter 能力目录。
- 创建审批时的 `executionReadiness`。
- 创建审批前的 `execution-preflight`。

当前 `approval_audit` 和 `platform_connect` 内部各自维护了一份平台执行参数解析、必要参数校验和 readiness 判断。短期能工作，但后续接真实 adapter 时容易出现：

- preflight 显示一个状态，审批创建写入另一个状态。
- 执行队列失败原因和平台连接页目录不一致。
- 新增平台动作时漏改其中一个模块。

本轮目标是把平台执行参数解析和 readiness 判断抽到 service 层共享模块，继续让审批、preflight 和后续执行闭环走同一套规则。

## 2. 数据库设计

不新增表，不新增 migration。

本轮只做 service 层代码组织与规则统一，不改变持久化结构。

## 3. Service 设计

新增 service 内部模块：

```text
backend/crates/service/src/api/platform_execution.rs
```

该模块不是 HTTP API，不注册 web-server 路由。它提供 service 内部共享函数和结构：

- `PlatformExecutionParams`
- `PlatformExecutionReadiness`
- `platform_execution_params(args)`
- `platform_execution_readiness_status(skill_name, params, connection)`
- `platform_execution_details(params)`
- `platform_adapter_not_implemented_message(skill_name)`
- `is_platform_execution_skill(skill_name)`
- `supported_platform_action(skill_name)`

改造目标：

- `ApprovalAuditApi::tool_approval_initial_result` 使用共享解析和 readiness。
- `ApprovalAuditApi::plan_platform_execution` 使用共享校验、详情和 adapter message。
- `PlatformConnectApi::execution_preflight` 使用共享解析和 readiness。
- `platform_connect` 删除本地重复的 readiness / validation / 参数解析 helper。

## 4. HTTP/API 设计

不新增 HTTP API。

既有 API 响应结构不变：

- `POST /api/app/platform-connect/execution-preflight`
- 工具审批 `result.executionReadiness`
- 审批通过后的执行队列 `result.adapterStatus/message/details`

由于响应结构不变，`API_DOCS/` 只需补充实现说明，不新增接口。

## 5. 权限与边界

- 继续由调用方 service 负责当前用户鉴权和资源归属校验。
- 共享模块只接收 `PlatformConnection` 引用和参数，不自行读取数据库。
- 不执行真实平台写操作。
- 不改变 `canApprove=false`、`canExecuteNow=false` 的当前边界。

## 6. 开发顺序

1. 新增 `platform_execution` service 内部模块。
2. 将两处重复规则迁入共享模块。
3. 修改 `approval_audit` 和 `platform_connect` 调用共享模块。
4. 保留并扩展现有单测，确保审批 readiness 和 preflight 仍返回相同状态。
5. 更新迁移记录、总计划和 API 文档说明。
6. 运行 cargo 和 pnpm 验证。
