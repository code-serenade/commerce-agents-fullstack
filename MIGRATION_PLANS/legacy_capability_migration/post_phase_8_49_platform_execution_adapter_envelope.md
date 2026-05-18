# Post 8-49：平台执行 Adapter 请求信封

状态：done

## 目标

本轮在真实平台 adapter 接入前，先补齐规范化 adapter request envelope。这样审批通过后的执行失败结果不仅知道“为什么失败”，也能知道“如果 adapter 已实现，本次准备如何调用 adapter”。

## 本轮修改

### 产品内核

支持的平台执行动作失败结果现在会包含 `adapterRequest`：

- `skillName`
- `platform`
- `adapterStatus`
- `executionMode`
- `idempotencyKey`
- `idempotencyKeyFields`
- `args`

该字段用于审计、重试和后续真实 adapter 接入，不代表平台动作已经真实执行。

### 后端

- `backend/crates/service/src/api/platform_execution.rs`
  - 新增 `PlatformExecutionAdapterRequest`
  - 新增 `platform_execution_adapter_request()`
  - 新增规范化 `platform_execution_args()`
  - 新增基于 action spec 的幂等键生成

- `backend/crates/service/src/api/approval_audit.rs`
  - 平台执行失败结果写入 `adapterRequest`
  - unsupported 动作或缺少平台时 `adapterRequest=null`

### 前端

- 本轮不修改前端。
- 既有接口和页面行为不变。
- 不新增假执行按钮，不展示假成功。

## 用户可感知变化

- 用户或后续前端查看审批通过后的执行失败结果时，可以看到更清晰的 adapter 请求边界。
- 当前真实平台 adapter 仍未实现，因此仍会显示 `adapter_not_implemented`。
- 系统不会把生成 request envelope 误显示成真实平台执行成功。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`

## 后续建议

- 下一步可继续补真实 adapter trait 和执行器边界。
- 真实 adapter 接入时应消费 `adapterRequest`，并把真实执行结果写回 `execution_queue.result` 和审计日志。
