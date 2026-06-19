# Account Usage Quota API

本文件记录 Agent 问答次数额度、用量统计和后台管理接口。历史文件名与部分前端目录仍保留 `team-usage`，但当前业务口径已经调整为：

- `schoolName` 和 `teamName` 只作为账号注册资料与统计维度。
- 默认 50 次问答额度按账号限制，唯一限流主体是 `app_users.user_id`。
- 后台运营管理按账号查看和调整额度，同时展示学校、队伍用于筛选和统计。
- `tokenUsed` 只作为运营统计信息，不参与限流。

## Permission

后台接口统一需要：

- JWT：需要
- 权限码：`usage:team_usage`

Agent 问答限流由 `POST /api/sse/agent/session/stream` 在服务端入口强制执行，不依赖前端按钮状态。

当前版本的限流依据只使用问答次数：

- 用户端只展示当前账号的问答次数额度、已用次数和剩余次数。
- 后台按账号配置问答次数上限、启用或停用。
- token 只作为后台运营统计信息，不参与限流。
- 当前 agent runtime 尚未暴露模型精确 token usage，`tokenUsed` 先按输入和输出文本估算统计。

## Data Model

### AccountUsageResponse

```json
{
  "id": "account-quota-uuid",
  "userId": "app-user-uuid",
  "displayId": "student01",
  "displayName": "学生 01",
  "schoolName": "示例院校",
  "teamName": "第一小队",
  "questionLimit": 50,
  "questionUsed": 12,
  "questionRemaining": 38,
  "tokenUsed": 3200,
  "status": "enabled",
  "remark": null,
  "lastUsedAt": "2026-06-19T03:00:00Z",
  "createdAt": "2026-06-19T03:00:00Z",
  "updatedAt": "2026-06-19T03:00:00Z"
}
```

字段说明：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | `account_usage_quotas.id`，账号额度记录 ID |
| `userId` | string | `app_users.user_id`，真实限流主体 |
| `displayId` | string | 用户展示 ID |
| `displayName` | string | 用户展示名称 |
| `schoolName` | string | 学校名，仅统计维度 |
| `teamName` | string | 队伍名，仅统计维度 |
| `questionLimit` | number | 当前账号问答次数上限 |
| `questionUsed` | number | 当前账号已记录的 Agent 问答次数 |
| `questionRemaining` | number | 当前账号剩余次数，最低为 0 |
| `tokenUsed` | number | 当前账号后台统计 token 合计；当前为估算值，不用于限流 |
| `status` | string | `enabled` / `disabled` |
| `remark` | string \| null | 后台备注 |
| `lastUsedAt` | string \| null | 当前账号最近一次 Agent 问答时间 |
| `createdAt` | string | 额度记录创建时间 |
| `updatedAt` | string | 额度记录更新时间 |

## 1. List Account Usage

- Method: `GET`
- Path: `/api/admin/usage/accounts`
- Compatibility Path: `/api/admin/usage/teams`
- JWT: required
- Permission: `usage:team_usage`

Query:

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `page` | number | 否 | 默认 1 |
| `pageSize` | number | 否 | 默认 20 |
| `keyword` | string | 否 | 匹配账号 ID、显示名、学校名、队伍名、备注 |
| `status` | string | 否 | `enabled` / `disabled` |
| `sortBy` | string | 否 | `updatedAt` / `displayId` / `questionUsed` / `questionRemaining` / `lastUsedAt` |
| `sortOrder` | string | 否 | `asc` / `desc` |

Response `data`:

```json
{
  "items": [],
  "page": 1,
  "pageSize": 20,
  "total": 0,
  "totalPages": 0,
  "hasNext": false,
  "hasPrev": false
}
```

`items` 中每一项为 `AccountUsageResponse`。

## 2. Get Account Usage

- Method: `GET`
- Path: `/api/admin/usage/accounts/{usageId}`
- Compatibility Path: `/api/admin/usage/teams/{usageId}`
- JWT: required
- Permission: `usage:team_usage`

Response `data`: `AccountUsageResponse`

Errors:

- `-11007`: account usage quota not found

## 3. Update Account Quota

- Method: `PATCH`
- Path: `/api/admin/usage/accounts/{usageId}/quota`
- Compatibility Path: `/api/admin/usage/teams/{usageId}/quota`
- JWT: required
- Permission: `usage:team_usage`

Request:

```json
{
  "questionLimit": 100,
  "status": "enabled",
  "remark": "比赛账号手动调整"
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `questionLimit` | number | 是 | `0..=1000000`；为 0 时表示不允许继续问答 |
| `status` | string | 是 | `enabled` / `disabled` |
| `remark` | string \| null | 否 | 最长 255 |

Response `data`: `AccountUsageResponse`

Errors:

- `-11007`: account usage quota not found
- `-11008`: invalid quota request

## 4. Get Current Account Usage Quota

- Method: `GET`
- Path: `/api/app/usage/quota`
- JWT: required

用户端当前登录用户查看当前账号的次数额度。该接口不返回 token。

Response `data`:

```json
{
  "userId": "app-user-uuid",
  "displayId": "student01",
  "displayName": "学生 01",
  "schoolName": "示例院校",
  "teamName": "第一小队",
  "questionLimit": 50,
  "questionUsed": 12,
  "questionRemaining": 38,
  "status": "enabled",
  "policyText": "当前账号前 50 次 AI 问答可使用"
}
```

字段说明：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | string | 当前账号 user_id，真实限流主体 |
| `displayId` | string | 当前账号展示 ID |
| `displayName` | string | 当前账号展示名称 |
| `schoolName` | string | 当前账号学校，统计维度 |
| `teamName` | string | 当前账号队伍，统计维度 |
| `questionLimit` | number | 当前账号问答次数上限 |
| `questionUsed` | number | 当前账号已用问答次数 |
| `questionRemaining` | number | 当前账号剩余问答次数，最低为 0 |
| `status` | string | `enabled` / `disabled` |
| `policyText` | string | 用户端展示的次数额度说明 |

## 5. Agent Stream Quota Errors

`POST /api/sse/agent/session/stream` 在启动 Agent 前检查当前账号配额。

错误码：

| code | HTTP | 说明 |
| --- | --- | --- |
| `-12068` | 429 | 当前账号问答次数已达到上限 |
| `-12069` | 403 | 当前账号问答已被后台停用 |
| `-12070` | 400 | 当前账号限流请求非法或配额记录不可用 |

当前版本配额以“允许启动一次 Agent SSE 问答”为一次计数单位。学校和队伍字段只作为事件快照写入 `agent_usage_events`，方便后台按统计维度观察用量。
