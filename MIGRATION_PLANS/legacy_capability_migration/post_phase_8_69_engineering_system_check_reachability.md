# Post 8-69：平台接口健康检查自然触达

## 背景

旧产品有 `EngineeringSystemCheck`，用于检查用户已配置的平台 API 连接状态。新产品已有 `engineering_system_check` 后端 skill，但用户用自然语言问“平台 API 连通性 / 接口健康 / 授权同步状态”时不容易命中该 skill。

本轮补强自然触达，同时保持真实性边界：不读取默认平台数据，不伪造真实连接检测，不绕过审批。

## 本次修改

- 工程 worker autosuggest 增加平台健康检查意图识别。
- 用户明确提到平台和接口健康 / 连通性 / 授权 / 同步状态时，自动补偿 `engineering_system_check`。
- 自动解析平台名，例如淘宝、天猫、抖音、京东、拼多多等。
- 文本包含“真实 / 实际 / 连通 / 测试”等词时，`test_live=true`，后端返回 `pending_approval`。
- runtime 元数据修正：`engineering_system_check` 按平台 adapter / 审批边界处理，`execution_mode=adapter`、`risk_level=high`、`approval_required=true`。
- 角色 skill catalog 展示时 `pending_approval` 优先于 `deferred`，避免模型或前端误解成普通缺数据能力。

## 用户可感知变化

- 用户问“检查一下淘宝平台 API 连通性和接口健康情况”时，后端会明确命中平台健康检查 skill。
- 如果只是查看配置状态，当前会说明平台连接配置读取尚未接入 Rust runtime。
- 如果要求真实测试连接，后端会进入待审批边界，而不是直接调用平台 API。
- 系统不会假装已经读取淘宝、抖音、千川等平台真实连接状态。

## 边界

- 本轮不实现官方平台 adapter。
- 本轮不创建健康检查报告 artifact。
- 本轮不新增 HTTP API。
- 本轮不改前端；前端后续应根据真实 `skill_result.status`、`execution_mode`、`risk_level`、`approval_required` 展示状态。

## 验证

- `cargo test -p agent autosuggests_engineering_system_check_with_platform_boundary -- --nocapture`
- `cargo test -p agent autosuggests_engineering_skill_by_intent -- --nocapture`
- `cargo test -p agent executor_marks_engineering_system_check_as_platform_boundary -- --nocapture`
- `cargo test -p agent engineering_system_check_stays_pending_approval_for_live_test -- --nocapture`
- `cargo test -p agent keeps_roles_from_requesting_unowned_skills -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
