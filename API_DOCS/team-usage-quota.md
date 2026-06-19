# Team Usage Quota API

本文件记录队伍维度 Agent 问答用量、配额和后台管理接口。

## Permission

后台接口统一需要：

- JWT：需要
- 权限码：`usage:team_usage`

Agent 问答限流由 `POST /api/sse/agent/session/stream` 在服务端入口强制执行。

## Data Model

### TeamUsageResponse

```json
{
  "id": "team-quota-uuid",
  "schoolName": "示例院校",
  "teamName": "第一小队",
  "memberCount": 4,
  "questionLimit": 50,
  "questionUsed": 12,
  "questionRemaining": 38,
  "tokenLimit": null,
  "tokenUsed": 0,
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
| `id` | string | `team_usage_quotas.id` |
| `schoolName` | string | 学校名 |
| `teamName` | string | 队伍名 |
| `memberCount` | number | 当前 `app_users` 中该队伍成员数量 |
| `questionLimit` | number | 队伍问答次数上限 |
| `questionUsed` | number | 已记录的 Agent 问答次数 |
| `questionRemaining` | number | 剩余次数，最低为 0 |
| `tokenLimit` | number \| null | 预留 token 上限；当前不用于拦截 |
| `tokenUsed` | number | 已记录 token 合计；当前可能为 0 |
| `status` | string | `enabled` / `disabled` |
| `remark` | string \| null | 后台备注 |
| `lastUsedAt` | string \| null | 最近一次 Agent 问答时间 |
| `createdAt` | string | 配额记录创建时间 |
| `updatedAt` | string | 配额记录更新时间 |

## 1. List Team Usage

- Method: `GET`
- Path: `/api/admin/usage/teams`
- JWT: required
- Permission: `usage:team_usage`

Query:

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `page` | number | 否 | 默认 1 |
| `pageSize` | number | 否 | 默认 20 |
| `keyword` | string | 否 | 匹配学校名、队伍名、备注 |
| `status` | string | 否 | `enabled` / `disabled` |
| `sortBy` | string | 否 | `updatedAt` / `questionUsed` / `questionRemaining` / `memberCount` / `lastUsedAt` |
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

## 2. Get Team Usage

- Method: `GET`
- Path: `/api/admin/usage/teams/{teamId}`
- JWT: required
- Permission: `usage:team_usage`

Response `data`: `TeamUsageResponse`

Errors:

- `-11007`: team usage quota not found

## 3. Update Team Quota

- Method: `PATCH`
- Path: `/api/admin/usage/teams/{teamId}/quota`
- JWT: required
- Permission: `usage:team_usage`

Request:

```json
{
  "questionLimit": 100,
  "tokenLimit": null,
  "status": "enabled",
  "remark": "比赛队伍手动调整"
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `questionLimit` | number | 是 | `0..=1000000`；为 0 时表示不允许继续问答 |
| `tokenLimit` | number \| null | 否 | 预留字段，本期不拦截 |
| `status` | string | 是 | `enabled` / `disabled` |
| `remark` | string \| null | 否 | 最长 255 |

Response `data`: `TeamUsageResponse`

Errors:

- `-11007`: team usage quota not found
- `-11008`: invalid quota request

## 4. List Team Members

- Method: `GET`
- Path: `/api/admin/usage/teams/{teamId}/members`
- JWT: required
- Permission: `usage:team_usage`

Response `data`:

```json
[
  {
    "userId": "user-uuid",
    "displayId": "student01",
    "displayName": "学生 01",
    "schoolName": "示例院校",
    "teamName": "第一小队",
    "status": "enabled",
    "createdAt": "2026-06-19T03:00:00Z",
    "updatedAt": "2026-06-19T03:00:00Z"
  }
]
```

## 5. Agent Stream Quota Errors

`POST /api/sse/agent/session/stream` 在启动 Agent 前检查队伍配额。

新增错误码：

| code | 说明 |
| --- | --- |
| `-12068` | 队伍问答次数已达到上限 |
| `-12069` | 队伍已被后台停用 |
| `-12070` | 队伍限流请求非法或配额记录不可用 |

当前版本配额以“允许启动一次 Agent SSE 问答”为一次计数单位。
