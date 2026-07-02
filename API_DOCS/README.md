# API 文档总览

本目录整理当前项目 `crates/web-server` 中已经暴露的全部 HTTP API。

## 文档目录

- `common.md`: 通用约定，包括鉴权、agent LLM 配置、统一响应格式、错误码
- `admin.md`: 后台 RBAC 相关接口
- `app.md`: 普通客户端接口
- `analytics.md`: 产品内部访问统计、账号级统计和后台统计查询接口
- `sse.md`: Agent session SSE 事件流协议
- `ws.md`: WebSocket 连接与小铃铛通知推送协议

## 路由前缀

- 后台管理接口前缀：`/api/admin`
- 后台账号管理接口前缀：`/api/admin/account`
- 后台权限中心接口前缀：`/api/admin/access`
- App 用户接口前缀：`/api/app`
- 产品内部统计匿名上报前缀：`/api/analytics`
- 产品内部统计后台查询前缀：`/api/admin/analytics`
- SSE 流式接口前缀：`/api/sse`
- WebSocket 连接路径：`/api/ws`

## 当前接口清单

### Admin

1. `POST /api/admin/account/admin-users`
2. `GET /api/admin/account/admin-users`
3. `PATCH /api/admin/account/admin-users/{userId}`
4. `DELETE /api/admin/account/admin-users/{userId}`
5. `POST /api/admin/access/roles`
6. `GET /api/admin/access/roles`
7. `DELETE /api/admin/access/roles/{roleId}`
8. `GET /api/admin/access/permissions`
9. `GET /api/admin/access/roles/{roleId}/permissions`
10. `PUT /api/admin/access/roles/{roleId}/permissions`
11. `POST /api/admin/access/menus`
12. `GET /api/admin/access/menus`
13. `GET /api/admin/account/admin-users/{userId}/roles`
14. `PUT /api/admin/account/admin-users/{userId}/roles`
15. `GET /api/admin/me/permissions`
16. `GET /api/admin/me/menus`
17. `GET /api/admin/account/app-users`
18. `GET /api/admin/account/app-users/metrics`
19. `PATCH /api/admin/account/app-users/{userId}`
20. `DELETE /api/admin/account/app-users/{userId}`
21. `GET /api/admin/account/app-users/{userId}/roles`
22. `PUT /api/admin/account/app-users/{userId}/roles`
23. `POST /api/admin/access/app-roles`
24. `GET /api/admin/access/app-roles`
25. `DELETE /api/admin/access/app-roles/{roleId}`
26. `GET /api/admin/access/app-permissions`
27. `GET /api/admin/access/app-roles/{roleId}/permissions`
28. `PUT /api/admin/access/app-roles/{roleId}/permissions`
29. `GET /api/admin/analytics/overview`
30. `GET /api/admin/analytics/sources`
31. `GET /api/admin/analytics/pages`
32. `GET /api/admin/analytics/accounts`
33. `GET /api/admin/analytics/school-teams`

### App

1. `POST /api/app/register`
2. `GET /api/app/me/permissions`
3. `GET /api/app/conversations`
4. `POST /api/app/conversations`
5. `GET /api/app/conversations/{conversationId}`
6. `PATCH /api/app/conversations/{conversationId}`
7. `DELETE /api/app/conversations/{conversationId}`
8. `POST /api/app/conversations/{conversationId}/messages`
9. `POST /api/app/conversations/{conversationId}/files/presign-upload` deprecated
10. `POST /api/app/conversations/{conversationId}/files`
11. `GET /api/app/conversation-files/{fileId}/access`
12. `DELETE /api/app/conversation-files/{fileId}`
13. `POST /api/app/analytics/events`

### Analytics

1. `POST /api/analytics/events`

### SSE

1. `POST /api/sse/agent/session/stream` Agent session stream

### WebSocket

1. `GET /api/ws` WebSocket upgrade
