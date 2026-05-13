# Post 8-48：平台执行动作规格注册表

## 1. 业务理解

平台执行治理已经形成三类能力：

- 平台连接页的 adapter 能力目录。
- 创建审批前的 preflight。
- 审批创建和审批通过后的 readiness / failure details。

Post 8-47 已把参数解析、readiness 和失败原因收敛到 `platform_execution` 共享模块，但平台连接页 catalog 仍在 `platform_connect` 内部维护动作规格，包括：

- `skill_name`
- 展示名和描述
- required / optional 参数
- 幂等字段
- adapter 未实现文案

这些字段本质上也是平台执行动作定义。如果后续接入真实平台 adapter 时仍分散在 `platform_connect`，容易出现 catalog 展示一套动作，preflight / approval 支持另一套动作。

本轮目标是把平台执行动作规格迁入 service 内部共享模块，继续保持后端唯一事实来源。

## 2. 数据库设计

不新增表，不新增 migration。

本轮仅做 service 内部规则和规格整理。

## 3. Service 设计

继续使用：

```text
backend/crates/service/src/api/platform_execution.rs
```

新增或调整：

- `PlatformExecutionActionSpec`
- `platform_execution_action_specs()`
- `platform_execution_action_spec(skill_name)`

共享规格应覆盖：

- `platform_update_price`
- `platform_update_inventory`
- `platform_sync_products`

`supported_platform_action`、`platform_execution_validation_error` 和 `platform_adapter_not_implemented_message` 应基于同一份 action spec 判断，避免重复 match。

`platform_connect` 的 catalog 映射继续在本模块内生成 DTO，但动作规格来自共享模块。

## 4. HTTP/API 设计

不新增 HTTP API。

以下接口响应结构不变：

- `GET /api/app/platform-connect/execution-adapters`
- `POST /api/app/platform-connect/execution-preflight`

`API_DOCS/app.md` 只补充实现说明：平台执行动作目录、preflight 和审批 readiness 共用 service 内部动作规格注册表。

## 5. 权限与边界

- 鉴权和用户归属仍由 `platform_connect` / `approval_audit` 调用方负责。
- 共享模块不读数据库，不执行平台写操作。
- 当前真实 adapter 未实现，`canExecuteNow=false` 的边界不变。
- 不新增前端假执行入口。

## 6. 开发顺序

1. 将 catalog action spec 结构和列表迁入 `platform_execution`。
2. 改造 `platform_connect` 使用共享 spec。
3. 改造 `supported_platform_action`、validation 和 adapter message 复用共享 spec。
4. 更新测试，确保 catalog / preflight / approval readiness 仍返回同样结果。
5. 更新 API 说明和迁移归档。
6. 运行 cargo 和 pnpm 门禁。
