# Post 8-23：Coordination 创建任务真实落地归档

## 本轮结论

本轮已把 `coordination_create_task` 从骨架状态推进为真实后端能力：

1. agent 侧已改为 native deterministic skill
2. ops 自然问法已可自动命中该 skill
3. service 层已可把 skill result 幂等落成真实 `workspace_task`
4. SSE 已可回传 `workspace_task_created`

## 本轮涉及文件

- `temp/DEVELOPMENT_DOCS/backend/agent_coordination_create_task_native_landing_post_8_23.md`
- `backend/crates/agent/src/skills/coordination.rs`
- `backend/crates/agent/src/agent/worker/runner.rs`
- `backend/crates/service/src/dto/workspace.rs`
- `backend/crates/service/src/api/workspace.rs`
- `backend/crates/web-server/src/handlers/sse.rs`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`

## 后端变化

### 1. agent skill

`coordination_create_task` 不再返回 `pending_approval` 骨架，而是返回：

- `status=success`
- `task_creation.should_persist=true`
- 标题 / 描述 / 负责人 / 优先级 / 验收标准

### 2. 自然问法可达性

ops 岗位下，以下类表达更容易触发建任务能力：

- 创建任务
- 建个任务
- 新建任务
- 安排任务
- 分派任务
- 待办

### 3. service 持久化

新增了 skill result 到 `workspace_task` 的真实落地方法：

- 仅处理 `coordination_create_task`
- 仅处理 `status=success`
- 以 `request_id -> workspaceTaskId` 做幂等
- 成功后回写 `agent_skill_run.metadata.workspaceTaskId`

### 4. SSE 事件

当任务成功落地后，SSE 会新增：

- `workspace_task_created`

该事件返回真实 `WorkspaceTaskResponse`，前端后续可以直接消费。

## 用户可感知变化

用户现在不只是收到“建议去建任务”的回答。

如果当前问答过程命中 `coordination_create_task`：

- 当前工作区会新增真实任务
- 任务可在后续工作区任务列表中继续使用
- SSE 过程里会看到真实任务创建事件

## 验证结果

- `cargo fmt` 通过
- `cargo test -p agent autosuggest` 通过
- `cargo test -p service workspace_task_artifact` 通过
- `cargo check -p agent` 通过
- `cargo check -p service` 通过
- `cargo check -p web-server` 通过
- `cargo check` 通过
- `cargo clippy` 通过

## 当前意义

这轮不是前端补按钮，而是把“协同建议”真正推进成“工作区可执行待办”。

它让新产品更接近“用户在指挥一组电商虚拟员工完成任务”，而不是只得到一段回答。
