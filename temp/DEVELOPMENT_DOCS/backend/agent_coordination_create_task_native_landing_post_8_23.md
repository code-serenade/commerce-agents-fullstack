# Post 8-23: coordination_create_task 原生落地设计

## 1. 背景

当前新产品已经具备真实的 `workspace_tasks` / `workspace_runs` / `agent_skill_runs` 持久化体系，但 `coordination_create_task` 仍只是 `pending_approval` 骨架。

这会导致一个明显断层：

- Agent 可以“识别需要建任务”
- 但不能把这件事真正落到当前工作区
- 前端和用户看不到真实新增的待办任务

本轮目标是把旧产品里的“协同生成工作任务”能力，按新产品架构真实迁入。

## 2. 目标

本轮只做以下真实能力：

1. `coordination_create_task` 从骨架升级为 Rust native skill
2. skill 返回结构化任务草案，而不是 `pending_approval`
3. SSE 持久化链路识别该类 skill 结果后，调用 `WorkspaceApi` 创建真实 `workspace_task`
4. 创建成功后通过 SSE 发送真实 `workspace_task_created`
5. 对同一个 `request_id` 做幂等控制，避免重复落任务

## 3. 不做的内容

本轮不做：

1. 不新增 HTTP API
2. 不改数据库 schema
3. 不做平台写操作审批
4. 不让 `agent` crate 直接跨层访问数据库或 service
5. 不做前端 UI 重构

## 4. 分层方案

### 4.1 agent crate

`coordination_create_task` 改为 native deterministic skill：

- 输入：
  - `title`
  - `description`
  - `owner_role`
  - `priority`
  - `acceptance_criteria`
- 输出：
  - `status=success`
  - `task_creation`
    - `should_persist=true`
    - `title`
    - `description`
    - `owner_role`
    - `priority`
    - `acceptance_criteria`
    - `workspace_id`
    - `source=session_skill`
  - `metadata`
    - `task_creation_kind=workspace_task`

注意：agent 只负责生成可信结构化意图，不直接写库。

### 4.2 service

在 `WorkspaceApi` 增加一个专用方法，把 skill result 落成真实 `workspace_task`：

- 输入：
  - 当前用户 / workspace / run / request_id / skill_id / role
  - skill result output
- 行为：
  1. 校验当前 workspace/run 归属
  2. 仅处理 `coordination_create_task`
  3. 仅处理 `output.status=success` 且 `task_creation.should_persist=true`
  4. 先查 `agent_skill_run.request_id`
  5. 如果该 skill run 的 metadata 已有 `workspaceTaskId`，直接返回既有任务，保证幂等
  6. 否则创建真实 `workspace_task`
  7. 回写 `agent_skill_run.metadata.workspaceTaskId`
  8. 返回 `WorkspaceTaskResponse`

这样可以保证：

- 不需要扩展 repo 过滤器去做脆弱的标题查重
- 幂等锚点和 agent 运行链路一致
- 任务落地和 skill 审计记录能双向关联

### 4.3 web-server SSE

在 `SessionEvent::SkillResult` 持久化后，追加一段：

- 调用 `WorkspaceApi::materialize_agent_workspace_task(...)`
- 如果返回 `Some(task)`，发 `workspace_task_created`
- 如果返回 `None`，说明本次 skill 不需要落任务，直接跳过

## 5. 用户可感知变化

本轮完成后：

1. 当用户或 Agent 提出“建个任务 / 生成待办 / 安排给某岗位处理”时，后端不再只说“需要审批”。
2. 如果命中 `coordination_create_task`，当前工作区会真正新增一条任务。
3. SSE 会出现真实 `workspace_task_created` 事件。
4. 刷新后在工作区任务列表里还能看到该任务，不是前端临时状态。

## 6. 风险与控制

风险：

1. 同一 skill result 被重复消费，导致重复任务
2. skill 输出字段不完整
3. 某些 prompt 命中率仍不足

控制：

1. 以 `request_id -> workspaceTaskId` 做幂等
2. service 层二次做字段归一化和兜底
3. 在 runner autosuggest 里补“创建任务/待办/安排任务”等自然问法

## 7. 验证

本轮完成后至少验证：

1. `cargo fmt`
2. `cargo test -p agent autosuggest`
3. `cargo test -p agent executor_runs_representative_native_categories`
4. `cargo check -p agent`
5. `cargo check -p service`
6. `cargo check -p web-server`
7. `cargo check`
8. `cargo clippy`

并补至少以下测试：

1. `coordination_create_task` native skill 输出测试
2. ops 自然问法命中 `coordination_create_task` 的 autosuggest 测试
3. service 层任务落地草案提取 / 幂等落地测试
