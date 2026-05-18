# Post 8-20：Skill run 记录查询 API 后端设计

## 目标

把 Agent SSE 中已经真实发生的 `skill_requested` / `skill_result` 从“只在前端流式过程里可见”，沉淀成新产品后端可查询的 Skill run 记录。这样旧产品的 skill run audit 能力可以落到新产品自己的 workspace/run/agent 体系里，前端后续能展示历史工具调用、执行模式、风险、审批状态和输出摘要。

## 数据表

新增表：`agent_skill_runs`

字段：

- `id uuid primary key`
- `user_id uuid not null`：当前 App 用户，来自 JWT
- `workspace_id uuid not null`：所属工作区
- `run_id uuid not null`：所属 workspace run
- `session_id varchar(128) null`：Agent session id
- `worker_id varchar(128) not null`
- `task_id varchar(128) not null`
- `request_id varchar(128) not null`：一次 skill 调用的请求 ID
- `skill_id varchar(128) not null`
- `role varchar(64) not null`
- `status varchar(32) not null default 'requested'`
- `reason text null`：请求 skill 的原因
- `summary text null`：执行摘要
- `error text null`：错误摘要
- `input jsonb not null default '{}'::jsonb`：预留给后续 skill 输入摘要，本轮不从 SSE 伪造输入
- `output jsonb not null default '{}'::jsonb`：`skill_result.output`，没有则 `{}`
- `execution_mode varchar(32) null`：native / adapter / deferred 等
- `risk_level varchar(32) null`
- `approval_required boolean not null default false`
- `metadata jsonb not null default '{}'::jsonb`
- `requested_at timestamptz not null default current_timestamp`
- `completed_at timestamptz null`
- `created_at timestamptz not null default current_timestamp`
- `updated_at timestamptz not null default current_timestamp`

约束与索引：

- unique：`(user_id, workspace_id, run_id, request_id)`
- index：`(user_id, run_id, requested_at)`
- index：`(workspace_id, run_id, status)`
- index：`(user_id, skill_id, requested_at)`
- 外键：`user_id -> app_users.user_id`
- 外键：`workspace_id -> workspaces.id`
- 外键：`run_id -> workspace_runs.id`

## 后端流程

1. SSE 收到 `SkillRequested`：
   - 创建或更新 `agent_skill_runs`
   - 状态写为 `requested`
   - 写入 worker、task、request、skill、role、reason、session/workspace/run

2. SSE 收到 `SkillResult`：
   - 按 `(user_id, workspace_id, run_id, request_id)` 找已有记录
   - 找到则更新结果字段
   - 找不到则补写一条结果记录，避免漏掉没有 request 事件的情况
   - 写入 status、summary、error、output、executionMode、riskLevel、approvalRequired、completedAt

3. HTTP 查询：
   - `GET /api/app/workspaces/{workspaceId}/runs/{runId}/skill-runs`
   - 仅查询当前用户自己的 workspace/run
   - 支持 `status`、`skillId`、`role`、`page`、`pageSize`
   - 返回分页结构和 `agentContext`

## API 响应

`AgentSkillRunResponse`：

```json
{
  "id": "uuid",
  "workspaceId": "uuid",
  "runId": "uuid",
  "sessionId": "session-id",
  "workerId": "worker.ops",
  "taskId": "task-1",
  "requestId": "task-1-skill-1",
  "skillId": "ops_campaign_plan",
  "role": "ops",
  "status": "success",
  "reason": "需要生成运营计划",
  "summary": "已生成运营计划摘要",
  "error": null,
  "input": {},
  "output": {},
  "executionMode": "native",
  "riskLevel": "low",
  "approvalRequired": false,
  "metadata": {},
  "requestedAt": "2026-05-12T00:00:00Z",
  "completedAt": "2026-05-12T00:00:02Z",
  "createdAt": "2026-05-12T00:00:00Z",
  "updatedAt": "2026-05-12T00:00:02Z"
}
```

## 鉴权

- 路由必须使用 App JWT。
- `user_id` 只来自 `AuthUser`。
- service 层校验当前用户存在且 enabled。
- service 层校验 workspace 属于当前用户且未归档。
- service 层校验 run 属于该 workspace 和当前用户。

## 错误边界

- workspace 不存在或已归档：`WORKSPACE_NOT_FOUND`
- run 不存在或不属于该 workspace：`WORKSPACE_RUN_NOT_FOUND`
- 请求参数非法：`WORKSPACE_REQUEST_INVALID`

## 本轮不做

- 不修改 `backend/crates/agent` 内核。
- 不从 LLM 文本里反推 skill input。
- 不把平台执行结果伪造成成功；只记录 SSE 真实 `skill_result`。
- 不新增前端页面，只同步 API adapter。
