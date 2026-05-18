# Post 8-50：平台执行 Adapter Outcome 契约

状态：done

## 目标

本轮在真实平台 adapter 接入前，先补齐统一 adapter outcome 契约。后续真实 adapter 无论成功、失败还是降级，都应该先返回 outcome，再由审批执行闭环写入 execution queue、tool approval 和 audit。

## 本轮修改

### 产品内核

平台执行失败结果现在由统一 outcome 语义承载：

- `status`
- `adapterStatus`
- `message`
- `adapterRequest`
- `providerResult`
- `retryable`

当前真实平台 adapter 未实现，因此 outcome 仍是：

- `status=failed`
- `adapterStatus=adapter_not_implemented`
- `providerResult=null`
- `retryable=false`

### 后端

- `backend/crates/service/src/api/platform_execution.rs`
  - 新增 `PlatformExecutionAdapterOutcome`
  - 新增 `platform_execution_adapter_outcome()`
  - 新增 `platform_execution_unimplemented_outcome()`

- `backend/crates/service/src/api/approval_audit.rs`
  - 平台执行失败结果改为从 outcome 写入
  - 保留 `adapterRequest`
  - 新增 `providerResult`
  - 新增 `retryable`

### 前端

- 本轮不修改前端。
- 既有接口和页面行为不变。
- 不新增假执行按钮，不展示假成功。

## 用户可感知变化

- 后续用户或前端查看平台执行失败详情时，可以看到失败是否可重试，以及平台 provider 是否有原始返回。
- 当前真实平台 adapter 仍未实现，因此 provider 原始返回为空，且不可重试。
- 系统不会把 outcome 误显示成真实平台执行成功。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`

## 后续建议

- 下一步可继续补真实 adapter trait / dispatcher。
- 真实 adapter 接入时，应返回 `PlatformExecutionAdapterOutcome`，并复用当前 execution queue、tool approval 和 audit 写回链路。
