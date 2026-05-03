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
