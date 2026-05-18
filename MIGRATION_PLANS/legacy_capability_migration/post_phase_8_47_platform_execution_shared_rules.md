# Post 8-47：平台执行共享规则内核

状态：done

## 目标

本轮把平台执行相关的重复后端规则收敛到 service 内部共享模块，保证以下三条链路使用同一套判断：

- 创建审批前的 `execution-preflight`
- 工具审批记录中的 `result.executionReadiness`
- 审批通过后执行队列中的失败详情

## 本轮修改

### 产品内核

- 平台写动作的可执行性判断从“多个模块各自判断”收敛为“一个后端规则内核判断”。
- 支持动作、必要参数、平台连接状态、禁用状态和 adapter 未实现状态的结果保持一致。
- 当前仍不执行真实平台写操作，不伪造改价、改库存或商品同步成功。

### 后端

- 新增 `backend/crates/service/src/api/platform_execution.rs`。
- `approval_audit` 改为复用共享参数解析、readiness、详情和 adapter 未实现文案。
- `platform_connect` 的 preflight 改为复用共享参数解析和 readiness。
- 删除两处本地重复的参数解析、validation 和 readiness helper。
- 不新增数据库表，不新增 migration。
- 不新增 HTTP route。

### 前端

- 本轮不修改前端。
- 既有前端请求和展示契约不变。
- 后续前端如果展示 preflight、审批 readiness 或执行队列失败详情，可以依赖这些结果来自同一套后端规则。

## 用户可感知变化

- 用户在不同位置看到的平台执行失败原因更一致。
- 如果平台参数缺失、连接缺失、连接禁用、动作不支持或 adapter 未实现，后端会给出统一状态。
- 当前用户仍不能真实执行平台写操作；系统会明确提示不可执行原因，而不是显示假成功。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`

## 后续建议

- 下一步可继续做真实平台 adapter 接口边界，但仍要保持 approval、execution_queue、audit 和失败降级闭环。
- 接入真实 adapter 前，应先扩展 `platform_execution` 共享规则，避免 preflight 和审批执行结果再次分叉。
