# Post 8-25：工具审批通过后的平台执行闭环归档

## 本轮结论

本轮已把高风险平台动作从“审批通过后停在 approved”推进为真实后端闭环：

1. 审批通过后会自动创建 `execution_queue`
2. 会立即尝试进入平台执行闭环
3. 执行结果会真实写回 `tool_approval_queue`、`execution_queue` 和 `audit_logs`
4. 未接远端 adapter 时返回真实失败，不伪造成功

## 本轮涉及文件

- `temp/DEVELOPMENT_DOCS/backend/tool_approval_platform_execution_closure_post_8_25.md`
- `backend/crates/service/src/api/approval_audit.rs`
- `backend/crates/web-server/src/handlers/sse.rs`
- `API_DOCS/app.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`

## 后端变化

### 1. 审批通过后自动进入执行队列

`approve_tool_approval` 不再只改审批状态，而是会：

- 创建真实 `execution_queue`
- 自动生成执行描述
- 补齐执行审计

### 2. 执行尝试会真实落失败语义

当前已覆盖：

- `connection_missing`
- `connection_disabled`
- `validation_error`
- `unsupported_action`
- `adapter_not_implemented`

### 3. SSE 审批参数更完整

SSE 自动创建审批记录时，现已把：

- `normalizedInputs`
- 原始 `output`

一并存进 `args`，供审批通过后的执行闭环读取。

## 用户可感知变化

用户批准高风险平台动作后，状态不再只是“已批准”。

当前后端会继续给出真实结果：

- 如果平台未接好，会明确失败原因
- 如果参数缺失，会明确提示缺什么
- 审批、执行、审计三条链路会真实打通

## 验证结果

- `cargo fmt` 通过
- `cargo check -p service` 通过
- `cargo check -p web-server` 通过
- `cargo clippy` 通过
- `cargo test -p service approval_execution_params_reads_normalized_inputs_from_skill_output -- --nocapture` 通过
- `cargo test -p service plan_platform_execution_reports_missing_connection -- --nocapture` 通过
- `cargo test -p service plan_platform_execution_reports_adapter_not_implemented_when_connected -- --nocapture` 通过

## 当前意义

这轮的价值不在于“已经真的调通远端平台”，而在于平台动作的审批后半段骨架已经真实成立。

后续接入真实淘宝/JD/PDD/抖音 adapter 时，不需要重做审批和审计主链路，只需把执行器补进现有闭环。
