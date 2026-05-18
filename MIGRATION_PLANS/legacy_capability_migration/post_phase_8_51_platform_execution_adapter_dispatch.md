# Post 8-51：平台执行 Adapter 调度边界

状态：done

## 目标

本轮继续在同事给的新产品框架内推进平台执行闭环，不引入新框架，不复制旧产品代码。

目标是把审批通过后“应该如何进入 adapter、当前为什么降级失败”的判断收敛到 `service::api::platform_execution`，让 `approval_audit` 只负责审批事务、执行队列、审批状态和审计写回。

## 本轮修改

### 产品内核

平台执行现在有统一 dispatch outcome 入口：

- 缺平台参数：`validation_error`
- 缺连接：`connection_missing`
- 连接禁用：`connection_disabled`
- 未支持动作：`unsupported_action`
- 已连接但真实 adapter 未接入：`adapter_not_implemented`

这些结果仍然统一输出为 failed outcome，不会伪造成平台执行成功。

### 后端

- `backend/crates/service/src/api/platform_execution.rs`
  - 新增 `platform_execution_dispatch_outcome()`
  - 复用现有 `platform_execution_readiness_status()`
  - 复用现有 `PlatformExecutionAdapterOutcome`
  - 补充 dispatch 单测

- `backend/crates/service/src/api/approval_audit.rs`
  - `plan_platform_execution()` 改为调用 `platform_execution_dispatch_outcome()`
  - `platform_execution_failed()` 改为接收 outcome
  - 删除审批模块内重复的平台执行状态判断分支

### 前端

- 本轮不修改前端。
- 接口字段保持兼容。
- 不新增假执行入口，不展示假成功。

## 用户可感知变化

- 当前页面表现基本不变，已连接平台仍会显示真实 adapter 未接入。
- 后续前端展示审批执行详情时，所有平台执行失败原因来自同一个后端内核入口，结果会更稳定一致。
- 这一步为后续真实平台 adapter 接入做准备，但本轮没有真实调用淘宝、京东、拼多多或抖音等平台写接口。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`

## 后续建议

- 下一步可继续在该 dispatch 边界后面增加真实 adapter trait / provider registry。
- 真实 adapter 接入后仍应返回 `PlatformExecutionAdapterOutcome`，并复用当前 execution queue、tool approval 和 audit 写回链路。
