# Post 8-48：平台执行动作规格注册表

状态：done

## 目标

本轮把平台执行动作规格从 `platform_connect` 收敛到 service 内部共享模块，避免平台连接页 catalog、preflight、审批 readiness 和执行失败详情之间出现动作定义漂移。

## 本轮修改

### 产品内核

- `platform_update_price`
- `platform_update_inventory`
- `platform_sync_products`

这三类高风险平台动作现在共用同一份动作规格注册表。注册表包含：

- skill ID
- 展示名
- 描述
- required / optional 参数
- 幂等字段
- adapter 未实现文案

### 后端

- `backend/crates/service/src/api/platform_execution.rs`
  - 新增 `PlatformExecutionActionSpec`
  - 新增 `platform_execution_action_specs()`
  - 新增 `platform_execution_action_spec(skill_name)`
  - `supported_platform_action` 改为基于共享 spec 判断
  - `platform_execution_validation_error` 改为基于共享 spec 判断
  - `platform_adapter_not_implemented_message` 改为读取共享 spec

- `backend/crates/service/src/api/platform_connect.rs`
  - 删除本地 `PlatformExecutionActionSpec`
  - 删除本地 `platform_execution_action_specs()`
  - adapter catalog 改为消费共享 spec 生成 DTO

### 前端

- 本轮不修改前端。
- 既有接口响应结构不变。
- 不新增假执行按钮，不展示假成功。

## 用户可感知变化

- 平台连接页里看到的“平台执行能力目录”和 preflight / 审批背后的支持动作判断更一致。
- 当前真实平台 adapter 仍未实现，因此仍会显示 `adapter_not_implemented`。
- 用户不会看到新的可执行按钮，也不会被误导为平台动作已经真实执行。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service platform_execution_adapter_catalog -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`

## 后续建议

- 下一步可继续设计真实平台 adapter 的 service trait 和执行结果结构。
- 接入真实 adapter 前，应优先扩展该动作规格注册表，再让审批、执行队列和审计消费同一份动作定义。
