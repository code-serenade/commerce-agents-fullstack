# Post 8-51：平台执行 Adapter 调度边界

## 1. 业务理解

平台执行闭环已经具备审批、执行队列、预检、readiness、动作规格、adapter request 和 adapter outcome。

当前问题是：审批通过后的执行规划仍在 `approval_audit` 中逐段判断参数、连接和 adapter 未实现状态。虽然结果已经写入 outcome，但“应该调用 adapter、还是降级为失败”的调度边界还没有集中到平台执行内核模块里。

本轮目标是在同事现有框架内继续收敛平台执行逻辑：

- 不新建第二套执行框架。
- 不复制旧产品代码。
- 不调用真实平台写接口。
- 在 `service::api::platform_execution` 中提供统一 adapter dispatch outcome 入口。
- `approval_audit` 只负责审批事务、执行队列和审计写回，不再重复维护平台执行判断分支。

## 2. 数据库设计

不新增表，不新增 migration。

继续写入既有字段：

- `execution_queue.result`
- `tool_approval.result`
- `audit_logs.metadata.result`

## 3. Service 设计

主要落点：

```text
backend/crates/service/src/api/platform_execution.rs
```

新增 service 内部函数：

```text
platform_execution_dispatch_outcome(skill_name, params, connection)
```

职责：

- 复用现有 `platform_execution_readiness_status()`。
- 对 `validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action` 生成统一 failed outcome。
- 对当前 adapter 尚未实现的路径生成 `adapter_not_implemented` outcome。
- 保留 `adapterRequest`、`providerResult`、`retryable` 等字段语义。

审批模块：

```text
backend/crates/service/src/api/approval_audit.rs
```

调整为：

- `plan_platform_execution()` 调用 `platform_execution_dispatch_outcome()`。
- `platform_execution_failed()` 接收 outcome，而不是自己判断 adapter 状态。
- 审批模块继续负责事务、execution queue、tool approval 和 audit 落库。

## 4. HTTP/API 设计

不新增 HTTP API。

既有接口响应字段不变：

- `adapterStatus`
- `adapterRequest`
- `providerResult`
- `retryable`
- `message`
- `details`

本轮只调整后端内部生成路径。

需要同步说明到：

```text
API_DOCS/app.md
```

## 5. 鉴权与边界

- 鉴权仍由现有 web-server/auth 和 `approval_audit` 调用链处理。
- 本轮不绕过 service/repo 分层。
- 本轮不读写平台真实 API。
- 本轮不把任何平台动作标记为执行成功。
- 当前已连接平台仍返回 `adapter_not_implemented`。
- 前端不修改，不新增假执行入口。

## 6. 开发顺序

1. 在 `platform_execution.rs` 新增 dispatch outcome 入口。
2. 改造 `approval_audit.rs` 的平台执行规划，调用共享 dispatch。
3. 补充单测覆盖缺连接、已连接但 adapter 未实现、unsupported action。
4. 更新 `API_DOCS/` 和迁移归档。
5. 运行 cargo / pnpm 门禁。
6. 本地 commit，不 push。
