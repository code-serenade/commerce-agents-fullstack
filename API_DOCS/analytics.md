# Product Access Analytics API

本文件记录“电商多智能体 / 电商小参谋”的产品内部访问统计接口。

统计系统分为两套：

1. 产品内部统计：前端把访问、来源、功能点击、AI 问答相关事件写入本产品数据库，管理后台通过本文件接口展示。
2. 百度统计：仅在用户端 Web 应用加载“电商多智能体-电商小参谋”的百度统计脚本，数据进入百度统计平台，不进入本产品数据库，也不作为管理后台数据源。

当前产品只接入以下百度统计脚本：

```text
https://hm.baidu.com/hm.js?4f69d896969b027f9fb40db4ca648741
```

不得接入“创习社”的百度统计脚本。

## 统一说明

- HTTP JSON 字段使用 `camelCase`。
- 时间字段使用 RFC3339，例如 `2026-06-30T08:00:00Z`。
- 所有接口返回 `CommonResponse<T>`。
- 内部统计接口不保存明文 IP；学校、队伍由服务端根据登录账号补齐。
- 埋点接口最大批量 `20` 条事件，按 `eventId` 幂等去重。
- 前端埋点调用应吞掉网络错误，不能阻塞用户正常操作。

## 事件枚举

`eventType` 支持：

| 值 | 说明 |
| --- | --- |
| `page_view` | 页面访问 |
| `session_start` | 会话开始 |
| `session_end` | 会话结束 |
| `feature_click` | 通用功能点击 |
| `agent_question_submit` | 提交 AI 问答 |
| `agent_answer_view` | 查看 AI 回答 |
| `process_panel_open` | 打开回答处理过程/像素小人工作过程面板 |
| `report_view` | 查看报告/结果页 |
| `report_copy` | 复制报告 |
| `report_download` | 下载报告 |
| `file_upload` | 上传文件 |
| `login_click` | 点击登录 |
| `login_success` | 登录成功 |
| `register_success` | 注册成功 |
| `frontend_error` | 前端错误 |
| `api_error` | 接口错误 |

`sourceKind` 支持：`direct` / `search` / `referral` / `social` / `ad` / `internal` / `unknown`。

`deviceType` 支持：`desktop` / `mobile` / `tablet` / `bot` / `unknown`。

`status` 支持：`ok` / `failed`。

## 1. 匿名访问事件上报

- Method: `POST`
- Path: `/api/analytics/events`
- JWT: 不需要
- Permission: 无

Request:

```json
{
  "events": [
    {
      "eventId": "analytics-event-uuid",
      "eventType": "page_view",
      "occurredAt": "2026-06-30T08:00:00Z",
      "anonymousId": "browser-anonymous-id",
      "sessionId": "browser-session-id",
      "routePath": "/agent-chat",
      "pageTitle": "电商小参谋",
      "referrer": "https://example.com/",
      "sourceKind": "referral",
      "sourceHost": "example.com",
      "utmSource": null,
      "utmMedium": null,
      "utmCampaign": null,
      "utmContent": null,
      "utmTerm": null,
      "featureKey": null,
      "deviceType": "desktop",
      "browserName": "Chrome",
      "osName": "Windows",
      "viewportWidth": 1440,
      "viewportHeight": 900,
      "status": "ok",
      "metadata": {}
    }
  ]
}
```

Response `data`:

```json
{
  "accepted": 1,
  "duplicated": 0,
  "rejected": 0
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `events` | array | 是 | 最多 20 条 |
| `eventId` | string | 是 | 客户端生成的幂等 ID，最大 128 字符 |
| `eventType` | string | 是 | 事件类型枚举 |
| `occurredAt` | string(datetime) | 是 | 客户端事件发生时间 |
| `anonymousId` | string \| null | 否 | 未登录访客 ID，最大 128 字符 |
| `sessionId` | string | 是 | 前端会话 ID，最大 128 字符 |
| `routePath` | string \| null | 否 | 当前路由，最大 256 字符 |
| `pageTitle` | string \| null | 否 | 页面标题，最大 256 字符 |
| `referrer` | string \| null | 否 | 来源页面，最大 512 字符 |
| `sourceKind` | string | 是 | 来源类型 |
| `sourceHost` | string \| null | 否 | 来源域名，最大 255 字符 |
| `utmSource` | string \| null | 否 | UTM source |
| `utmMedium` | string \| null | 否 | UTM medium |
| `utmCampaign` | string \| null | 否 | UTM campaign |
| `utmContent` | string \| null | 否 | UTM content |
| `utmTerm` | string \| null | 否 | UTM term |
| `featureKey` | string \| null | 否 | 功能点 key，例如 `agent_chat.submit` |
| `deviceType` | string | 是 | 设备类型 |
| `browserName` | string \| null | 否 | 浏览器名称 |
| `osName` | string \| null | 否 | 操作系统 |
| `viewportWidth` | number \| null | 否 | 视口宽度 |
| `viewportHeight` | number \| null | 否 | 视口高度 |
| `status` | string | 否 | 默认 `ok` |
| `metadata` | object | 否 | 扩展信息，默认 `{}` |

## 2. 登录账号访问事件上报

- Method: `POST`
- Path: `/api/app/analytics/events`
- JWT: 需要 App 用户登录态
- Permission: 有效 App 用户

Request / Response 与“匿名访问事件上报”一致。

服务端会根据 JWT 中的 `userId` 查询 `app_users`，并补齐：

- `actorKind = app_user`
- `userId`
- `schoolName`
- `teamName`

## 3. 统计总览

- Method: `GET`
- Path: `/api/admin/analytics/overview`
- JWT: 需要后台登录态
- Permission: `analytics:overview`

Query:

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `from` | string(datetime) | 否 | 统计开始时间，默认最近 30 天 |
| `to` | string(datetime) | 否 | 统计结束时间，默认当前时间 |
| `granularity` | string | 否 | `day` / `week`，第一阶段 `week` 暂按日返回 |

Response `data`:

```json
{
  "summary": {
    "pv": 1200,
    "uv": 320,
    "sessions": 480,
    "activeAccounts": 90,
    "agentQuestions": 210,
    "tokenUsed": 320000,
    "avgSessionDurationSec": 360,
    "bounceRate": 0.35,
    "frontendErrors": 2,
    "apiErrors": 1
  },
  "dailyTraffic": [
    {
      "date": "2026-06-30",
      "pv": 120,
      "uv": 40,
      "sessions": 50,
      "activeAccounts": 18,
      "agentQuestions": 22,
      "tokenUsed": 30000,
      "errors": 0
    }
  ]
}
```

## 4. 访问来源统计

- Method: `GET`
- Path: `/api/admin/analytics/sources`
- JWT: 需要后台登录态
- Permission: `analytics:overview`

Query:

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `from` | string(datetime) | 否 | 统计开始时间 |
| `to` | string(datetime) | 否 | 统计结束时间 |

Response `data`:

```json
{
  "sourceKinds": [
    { "key": "direct", "pv": 500, "uv": 180, "sessions": 220, "agentQuestions": 80 }
  ],
  "sourceHosts": [
    { "key": "example.com", "pv": 120, "uv": 60, "sessions": 70, "agentQuestions": 20 }
  ],
  "campaigns": [
    { "key": "summer_campaign", "pv": 30, "uv": 20, "sessions": 22, "agentQuestions": 8 }
  ]
}
```

## 5. 页面统计

- Method: `GET`
- Path: `/api/admin/analytics/pages`
- JWT: 需要后台登录态
- Permission: `analytics:overview`

Query 同“访问来源统计”。

Response `data`:

```json
{
  "items": [
    {
      "routePath": "/agent-chat",
      "pv": 600,
      "uv": 200,
      "sessions": 260,
      "avgSessionDurationSec": 420,
      "entryCount": 180,
      "exitCount": 120,
      "errors": 1
    }
  ]
}
```

## 6. 账号级统计

- Method: `GET`
- Path: `/api/admin/analytics/accounts`
- JWT: 需要后台登录态
- Permission: `analytics:accounts`

Query:

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `page` | number | 否 | 默认 1 |
| `pageSize` | number | 否 | 默认 20 |
| `keyword` | string | 否 | 匹配用户 ID、账号、姓名、学校、队伍 |
| `schoolName` | string | 否 | 学校精确过滤 |
| `teamName` | string | 否 | 队伍精确过滤 |
| `from` | string(datetime) | 否 | 统计开始时间 |
| `to` | string(datetime) | 否 | 统计结束时间 |
| `sortBy` | string | 否 | `lastSeenAt` / `pv` / `sessions` / `agentQuestions` / `activeDays` |
| `sortOrder` | string | 否 | `asc` / `desc` |

Response `data`:

```json
{
  "items": [
    {
      "userId": "app-user-uuid",
      "displayId": "student01",
      "displayName": "学生 01",
      "schoolName": "示例院校",
      "teamName": "第一小队",
      "pv": 30,
      "sessions": 8,
      "activeDays": 4,
      "agentQuestions": 12,
      "tokenUsed": 18000,
      "lastSeenAt": "2026-06-30T08:00:00Z",
      "topPages": [
        { "routePath": "/agent-chat", "pv": 20 }
      ]
    }
  ],
  "page": 1,
  "pageSize": 20,
  "total": 1,
  "totalPages": 1,
  "hasNext": false,
  "hasPrev": false
}
```

## 7. 学校/队伍统计

- Method: `GET`
- Path: `/api/admin/analytics/school-teams`
- JWT: 需要后台登录态
- Permission: `analytics:school_teams`

Query:

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `page` | number | 否 | 默认 1 |
| `pageSize` | number | 否 | 默认 20 |
| `schoolName` | string | 否 | 学校精确过滤 |
| `teamName` | string | 否 | 队伍精确过滤 |
| `from` | string(datetime) | 否 | 统计开始时间 |
| `to` | string(datetime) | 否 | 统计结束时间 |
| `sortBy` | string | 否 | `activeAccounts` / `pv` / `sessions` / `agentQuestions` / `lastSeenAt` |
| `sortOrder` | string | 否 | `asc` / `desc` |

Response `data`:

```json
{
  "items": [
    {
      "schoolName": "示例院校",
      "teamName": "第一小队",
      "accountCount": 30,
      "activeAccounts": 12,
      "pv": 180,
      "uv": 40,
      "sessions": 70,
      "agentQuestions": 45,
      "tokenUsed": 70000,
      "lastSeenAt": "2026-06-30T08:00:00Z"
    }
  ],
  "page": 1,
  "pageSize": 20,
  "total": 1,
  "totalPages": 1,
  "hasNext": false,
  "hasPrev": false
}
```

## 错误响应

常见错误：

| code | HTTP | 说明 |
| --- | --- | --- |
| `-12071` | 400 | 统计请求参数非法，例如批量超过 20 条、登录用户 ID 非法 |
| 通用鉴权错误 | 401/403 | 未登录或缺少后台权限 |

