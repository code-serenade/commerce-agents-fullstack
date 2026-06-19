# Team Usage Quota Backend Design

## Scope

本次只实现学校/队伍维度的 Agent 问答限流、用量记录和后台管理。

不做：

- 界面整体体验重构
- AI 角色个性化和 prompt 重构
- 付费、订单、套餐、支付流水

## Business Rules

1. 新注册用户必须填写 `schoolName` 和 `teamName`。
2. 业务库继续使用 `app_users.school_name` 和 `app_users.team_name` 保存注册资料。
3. 限流维度为队伍：`school_name + team_name`。
4. 历史用户若缺少学校或队伍，归入稳定兜底分组：
   - `school_name = 未填写学校`
   - `team_name = 未填写队伍`
5. 默认队伍问答上限为 50 次。
6. 后台可按队伍修改问答次数上限、启用/停用限流。
7. 限流拦截点在 `POST /api/sse/agent/session/stream` 后端入口，不依赖前端禁用按钮。
8. 每次通过限流并启动 Agent SSE 后，记录一条 usage event。当前版本以“启动一次 Agent 问答”为计数单位。
9. token 先记录与展示，不做 token 上限拦截。

## Database Design

### `team_usage_quotas`

队伍维度配额表。

| Field | Type | Constraint | Description |
| --- | --- | --- | --- |
| `id` | UUID | PK | 配额记录 ID |
| `school_name` | varchar(128) | not null | 学校名，归一化后保存 |
| `team_name` | varchar(128) | not null | 队伍名，归一化后保存 |
| `question_limit` | integer | not null, default 50 | 队伍总问答次数上限 |
| `token_limit` | integer | nullable | 历史预留字段，本期不作为接口配置项，不参与拦截 |
| `status` | varchar(32) | not null, default `enabled` | `enabled` / `disabled` |
| `remark` | varchar(255) | nullable | 后台备注 |
| `created_at` | timestamptz | not null | 创建时间 |
| `updated_at` | timestamptz | not null | 更新时间 |

Indexes:

- unique: `(school_name, team_name)`
- index: `(status, updated_at)`

### `agent_usage_events`

Agent 问答用量事件表。

| Field | Type | Constraint | Description |
| --- | --- | --- | --- |
| `id` | UUID | PK | 事件 ID |
| `user_id` | UUID | FK -> `app_users.user_id` | 当前 App 用户 |
| `school_name` | varchar(128) | not null | 事件发生时的学校快照 |
| `team_name` | varchar(128) | not null | 事件发生时的队伍快照 |
| `conversation_id` | UUID | nullable | 会话 ID |
| `workspace_id` | UUID | nullable | 工作区 ID |
| `run_id` | UUID | nullable | workspace run ID |
| `session_id` | varchar(128) | not null | Agent session ID |
| `agent_id` | varchar(100) | nullable | Agent ID |
| `question_count` | integer | not null, default 1 | 本事件消耗问答次数 |
| `prompt_tokens` | integer | nullable | 预留 |
| `completion_tokens` | integer | nullable | 预留 |
| `total_tokens` | integer | nullable | 本期可为空 |
| `status` | varchar(32) | not null | `started` / `completed` / `failed` |
| `metadata` | jsonb | not null | 附加信息 |
| `created_at` | timestamptz | not null | 创建时间 |
| `updated_at` | timestamptz | not null | 更新时间 |

Indexes:

- `(school_name, team_name, created_at)`
- `(user_id, created_at)`
- `(session_id)`
- `(conversation_id)`

## Service Design

新增 `TeamUsageApi`：

- `preflight_agent_question(user_id, conversation_id, session_id, agent_id)`
  - 校验 App 用户存在且 enabled。
  - 解析队伍 key。
  - 确保队伍配额记录存在；不存在时创建默认 `question_limit=50`。
  - 统计队伍已用 question count。
  - 若配额 disabled 或已达上限，返回业务错误。
  - 返回 preflight 上下文。
- `record_agent_question_started(preflight, workspace_id, run_id)`
  - 写入 `agent_usage_events(status=started, question_count=1)`。
- `list_team_usages(current_admin_user_id, query)`
  - 需要后台权限 `usage:team_usage`。
  - 聚合队伍成员数、已用次数、token 和配额。
- `update_team_quota(current_admin_user_id, team_id, request)`
  - 需要后台权限 `usage:team_usage`。
  - 修改问答次数上限、状态、备注。
- `list_team_members(current_admin_user_id, team_id)`
  - 需要后台权限 `usage:team_usage`。

## HTTP API

后台接口挂载在 `/api/admin/usage`，统一需要 JWT 和 `usage:team_usage` 权限。

- `GET /api/admin/usage/teams`
- `GET /api/admin/usage/teams/{teamId}`
- `PATCH /api/admin/usage/teams/{teamId}/quota`
- `GET /api/admin/usage/teams/{teamId}/members`

## Error Codes

新增 App 错误码：

- `-12068 TEAM_USAGE_QUOTA_EXCEEDED`
- `-12069 TEAM_USAGE_DISABLED`
- `-12070 TEAM_USAGE_REQUEST_INVALID`

新增 Admin 错误码：

- `-11007 ADMIN_USAGE_TEAM_NOT_FOUND`
- `-11008 ADMIN_USAGE_REQUEST_INVALID`

## Development Order

1. migration
2. entity
3. repo DTO/service
4. service DTO/API
5. web-server DTO/handler/route
6. SSE handler preflight + started record
7. `API_DOCS/` 更新
8. admin 前端页面
9. 验证与提交
