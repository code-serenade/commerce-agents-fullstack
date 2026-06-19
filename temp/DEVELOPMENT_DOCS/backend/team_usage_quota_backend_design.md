# Account Usage Quota Backend Design

## Scope

本阶段只实现账号级 Agent 问答限流、用量记录和后台运营管理。

不做：

- 界面整体体验重构
- AI 角色个性化和 prompt 重构
- 付费、订单、套餐、支付流水

## Business Rules

1. 新注册用户填写的 `schoolName` 和 `teamName` 继续保存到 `app_users`。
2. 学校和队伍只是统计字段，用于后台查询、分组和运营观察，不作为限流主体。
3. 限流维度为账号：`app_users.user_id`。
4. 默认账号问答上限为 50 次。
5. 后台可按账号修改问答次数上限、启用/停用限流和备注。
6. 限流拦截点在 `POST /api/sse/agent/session/stream` 后端入口，不依赖前端禁用按钮。
7. 每次通过限流并启动 Agent SSE 后，记录一条 usage event。当前版本以“启动一次 Agent 问答”为计数单位。
8. token 先记录与展示，不做 token 上限拦截。
9. 历史用户若缺少学校或队伍，统计展示归入稳定兜底字段：
   - `school_name = 未填写学校`
   - `team_name = 未填写队伍`

## Database Design

### `account_usage_quotas`

账号维度配额表。

| Field | Type | Constraint | Description |
| --- | --- | --- | --- |
| `id` | UUID | PK | 配额记录 ID |
| `user_id` | UUID | unique, FK -> `app_users.user_id` | 账号 ID，唯一限流主体 |
| `question_limit` | integer | not null, default 50 | 该账号总问答次数上限 |
| `status` | varchar(32) | not null, default `enabled` | `enabled` / `disabled` |
| `remark` | varchar(255) | nullable | 后台备注 |
| `created_at` | timestamptz | not null | 创建时间 |
| `updated_at` | timestamptz | not null | 更新时间 |

Indexes:

- unique: `user_id`
- index: `(status, updated_at)`

### `agent_usage_events`

Agent 问答用量事件表。

| Field | Type | Constraint | Description |
| --- | --- | --- | --- |
| `id` | UUID | PK | 事件 ID |
| `user_id` | UUID | FK -> `app_users.user_id` | 当前 App 用户，统计与限流聚合主体 |
| `school_name` | varchar(128) | not null | 事件发生时的学校快照，仅统计维度 |
| `team_name` | varchar(128) | not null | 事件发生时的队伍快照，仅统计维度 |
| `conversation_id` | UUID | nullable | 会话 ID |
| `workspace_id` | UUID | nullable | 工作区 ID |
| `run_id` | UUID | nullable | workspace run ID |
| `session_id` | varchar(128) | not null | Agent session ID |
| `agent_id` | varchar(100) | nullable | Agent ID |
| `question_count` | integer | not null, default 1 | 本事件消耗问答次数 |
| `prompt_tokens` | integer | nullable | 输入 token 估算 |
| `completion_tokens` | integer | nullable | 输出 token 估算 |
| `total_tokens` | integer | nullable | token 统计值，不参与限流 |
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

`TeamUsageApi` 保留历史命名，但业务语义调整为账号用量：

- `preflight_agent_question(user_id, conversation_id, session_id, agent_id)`
  - 校验 App 用户存在且 enabled。
  - 确保当前账号 `account_usage_quotas` 记录存在；不存在时创建默认 `question_limit=50`。
  - 按 `user_id` 统计已用 question count。
  - 若账号额度 disabled 或已达上限，返回业务错误。
  - 返回 preflight 上下文，同时携带学校/队伍快照用于事件统计。
- `record_agent_question_started(preflight, workspace_id, run_id)`
  - 写入 `agent_usage_events(status=started, question_count=1)`。
- `record_agent_question_output(preflight, workspace_id, run_id)`
  - 写入 `question_count=0` 的输出 token 估算事件，不重复计次数。
- `get_current_team_usage_quota(current_user_id)`
  - 返回当前账号的次数额度，不返回 token。
- `list_team_usages(current_admin_user_id, query)`
  - 需要后台权限 `usage:team_usage`。
  - 返回账号级用量行，附带学校/队伍统计字段。
- `update_team_quota(current_admin_user_id, request)`
  - 需要后台权限 `usage:team_usage`。
  - 按 `account_usage_quotas.id` 修改问答次数上限、状态、备注。

## HTTP API

后台接口挂载在 `/api/admin/usage`，统一需要 JWT 和 `usage:team_usage` 权限。

- `GET /api/admin/usage/accounts`
- `GET /api/admin/usage/accounts/{usageId}`
- `PATCH /api/admin/usage/accounts/{usageId}/quota`

兼容旧路径：

- `GET /api/admin/usage/teams`
- `GET /api/admin/usage/teams/{usageId}`
- `PATCH /api/admin/usage/teams/{usageId}/quota`

用户端接口：

- `GET /api/app/usage/quota`

## Error Codes

App 错误码：

- `-12068 TEAM_USAGE_QUOTA_EXCEEDED`：当前账号问答次数已达上限。
- `-12069 TEAM_USAGE_DISABLED`：当前账号问答已被后台停用。
- `-12070 TEAM_USAGE_REQUEST_INVALID`：账号限流请求非法。

Admin 错误码：

- `-11007 ADMIN_USAGE_ACCOUNT_NOT_FOUND`：账号额度记录不存在。
- `-11008 ADMIN_USAGE_REQUEST_INVALID`：额度更新请求非法。

## Development Order

1. migration
2. entity
3. repo DTO/service
4. service DTO/API
5. web-server DTO/handler/route
6. SSE handler preflight + usage event record
7. `API_DOCS/` 更新
8. admin/web 前端接入
9. 验证与提交
