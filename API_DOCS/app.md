# App API

接口前缀：`/api/app`

说明：本文件只记录普通客户端使用的接口。后台管理 App 用户、App 角色、App 角色授权的接口属于后台管理端，统一记录在 `API_DOCS/admin.md`。

## App 权限码

当前初始化脚本 `scripts/init_app_permissions.sh` 会写入以下 App 权限：

| App 权限码 | 类型 | 用途 |
| --- | --- | --- |
| `profile` | `group` | 个人中心 |
| `profile:view` | `action` | 查看个人资料 |
| `profile:update` | `action` | 更新个人资料 |

## 1. 注册 App 用户

- 方法：`POST`
- 路径：`/api/app/register`
- 鉴权：有效 Auth JWT

请求体：

```json
{
  "displayId": "u_10001",
  "displayName": "App User",
  "remark": "optional note"
}
```

成功响应 `data`：`AppUserResponse`。

补充说明：

- `userId` 由服务端从 JWT 当前用户中读取，前端不得传入或伪造
- 首次注册会创建 `enabled` 状态 App 用户
- 首次注册会自动绑定默认 App 角色 `free`
- 如果默认 App 角色 `free` 不存在，服务端会自动创建；初始化脚本 `scripts/init_app_permissions.sh` 也会写入该角色
- 同一个 `userId` 重复注册时返回已有用户信息
- 该接口用于登录后的业务 App 用户初始化；前端应先查询 `/api/app/me/permissions`，仅当当前 Auth 用户尚未初始化为 App 用户时再调用本接口

## 2. 查询当前 App 用户权限

- 方法：`GET`
- 路径：`/api/app/me/permissions`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "userId": "1b1f4e1d-5b4f-4d25-ae07-520f587f8d13",
  "roleCodes": [
    "user"
  ],
  "permissionCodes": [
    "profile",
    "profile:view",
    "profile:update"
  ]
}
```

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 返回权限码来自该用户绑定的 App 角色
- 如果当前 Auth 用户尚未初始化为 App 用户，返回 `404`，错误码为 `-12000`
- 如果当前 App 用户已被禁用，返回 `403`，错误码为 `-12001`

## 3. 查询我的会话列表

- 方法：`GET`
- 路径：`/api/app/conversations`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ConversationResponse[]`。

## 4. 创建会话

- 方法：`POST`
- 路径：`/api/app/conversations`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "title": "New chat",
  "firstMessage": "hello",
  "agentId": "default-agent",
  "modelName": "default-model"
}
```

成功响应 `data`：`ConversationDetailResponse`。

## 5. 查询会话详情

- 方法：`GET`
- 路径：`/api/app/conversations/{conversationId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ConversationDetailResponse`，包含 `conversation`、`messages`、`files`。

## 6. 更新会话标题

- 方法：`PATCH`
- 路径：`/api/app/conversations/{conversationId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "title": "新的对话名称"
}
```

成功响应 `data`：`ConversationResponse`。

补充说明：

- 只能更新当前用户自己的会话
- `title` 最长 255 个字符，空字符串会按 `null` 处理

## 7. 删除会话

- 方法：`DELETE`
- 路径：`/api/app/conversations/{conversationId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 只能删除当前用户自己的会话
- 当前实现为软删除，删除后不会再出现在会话列表中

## 8. 添加会话消息

- 方法：`POST`
- 路径：`/api/app/conversations/{conversationId}/messages`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "role": "user",
  "contentText": "hello",
  "contentJson": null
}
```

成功响应 `data`：`ConversationMessageResponse`。

补充说明：

- `role` 只能是 `user`、`assistant`、`system`、`tool`
- `contentText` 和 `contentJson` 至少需要一个有值

## 9. 获取会话附件上传签名

- 方法：`POST`
- 路径：`/api/app/conversations/{conversationId}/files/presign-upload`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "extension": "png",
  "contentType": "image/png"
}
```

成功响应 `data`：

```json
{
  "method": "PUT",
  "uploadUrl": "http://s3.example/private/chat/...",
  "bucket": "private",
  "objectKey": "chat/userScope/conversationId/file.png",
  "contentType": "image/png",
  "expiresSecs": 600
}
```

补充说明：

- 聊天附件固定使用 private bucket
- 前端使用 `uploadUrl` 上传成功后，再调用注册附件接口写入业务表

## 10. 注册会话附件

- 方法：`POST`
- 路径：`/api/app/conversations/{conversationId}/files`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "messageId": null,
  "objectKey": "chat/userScope/conversationId/file.png",
  "fileName": "file.png",
  "mimeType": "image/png",
  "fileSize": 12345,
  "sha256": "optional"
}
```

成功响应 `data`：`ConversationFileResponse`。

补充说明：

- `bucket` 不由前端传入，服务端固定写入 `private`
- `objectKey` 必须来自当前用户当前会话的上传签名路径

## 11. 获取会话附件下载签名

- 方法：`GET`
- 路径：`/api/app/conversation-files/{fileId}/access`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "file": {
    "id": "file-uuid",
    "conversationId": "conversation-uuid",
    "messageId": null,
    "bucket": "private",
    "objectKey": "chat/userScope/conversationId/file.png",
    "fileName": "file.png",
    "mimeType": "image/png",
    "fileSize": 12345,
    "sha256": null,
    "status": "active",
    "createdAt": "2026-05-06T00:00:00Z"
  },
  "method": "GET",
  "downloadUrl": "http://s3.example/private/chat/...",
  "expiresSecs": 600
}
```

## 会话接口权限与错误

- 所有会话接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 会话不存在返回 `404`，错误码 `-12002`
- 会话不属于当前用户返回 `403`，错误码 `-12003`
- 文件不存在返回 `404`，错误码 `-12004`
- 文件不属于当前用户返回 `403`，错误码 `-12005`
- 请求参数不合法返回 `400`，错误码 `-12006`
