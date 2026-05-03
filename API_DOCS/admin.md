# Admin API

接口前缀：`/api/admin`

说明：本组接口用于后台用户、角色、权限读取、角色权限配置、菜单、用户角色绑定，以及当前用户可见权限和菜单查询。权限节点由程序员通过代码、迁移或初始化脚本维护；HTTP API 不提供权限节点创建、修改或删除。

## 权限码总览

| 权限码                                | 用途                                                                |
| ------------------------------------- | ------------------------------------------------------------------- |
| `dashboard`                           | Dashboard 一级菜单                                                  |
| `accounts`                            | Accounts 一级菜单                                                   |
| `accounts:admin_users`                | Admin Users 二级菜单；后台用户及后台用户角色配置                    |
| `accounts:app_users`                  | App Users 二级菜单；普通用户及普通用户角色配置                      |
| `access_control`                      | Access Control 一级菜单                                             |
| `access_control:roles`                | Roles 二级菜单；后台角色管理                                        |
| `access_control:role_permissions`     | Role Permissions 二级菜单；后台权限树、菜单、后台角色权限配置       |
| `access_control:app_roles`            | App Roles 二级菜单；普通用户角色管理                                |
| `access_control:app_role_permissions` | App Role Permissions 二级菜单；普通用户权限树、普通用户角色权限配置 |

`GET /api/admin/me/permissions` 和 `GET /api/admin/me/menus` 不要求单独权限码，但要求当前用户本身是有效后台用户。

## 统一返回说明

所有接口都返回 `CommonResponse<T>`，成功时结构类似：

```json
{
  "code": 0,
  "message": "ok",
  "data": {}
}
```

## 1. 创建后台用户

- 方法：`POST`
- 路径：`/api/admin/account/admin-users`
- 权限：`accounts:admin_users`

请求体：

```json
{
  "identifier": "zhangsan",
  "remark": "测试环境管理员"
}
```

字段说明：

| 字段         | 类型             | 必填 | 约束                                       |
| ------------ | ---------------- | ---- | ------------------------------------------ |
| `identifier` | `string`         | 是   | 长度 `1..=128`，用于调用 auth 服务定位用户 |
| `remark`     | `string \| null` | 否   | 有值时长度 `1..=255`                       |

成功响应 `data`：

```json
{
  "userId": "1b1f4e1d-5b4f-4d25-ae07-520f587f8d13",
  "displayId": "zhangsan",
  "displayName": "张三",
  "remark": "测试环境管理员",
  "status": "enabled",
  "roles": []
}
```

补充说明：

- 接口会先通过 `identifier` 查询 auth 用户，再写入后台用户表

## 2. 查询后台用户列表

- 方法：`GET`
- 路径：`/api/admin/account/admin-users`
- 权限：`accounts:admin_users`

成功响应 `data`：

```json
[
  {
    "userId": "1b1f4e1d-5b4f-4d25-ae07-520f587f8d13",
    "displayId": "zhangsan",
    "displayName": "张三",
    "remark": "测试环境管理员",
    "status": "enabled",
    "roles": [
      {
        "id": 1,
        "name": "系统管理员",
        "code": "system_admin"
      }
    ]
  }
]
```

补充说明：

- 带有 `root` 角色的用户不会出现在列表中

## 3. 更新后台用户

- 方法：`PATCH`
- 路径：`/api/admin/account/admin-users/{userId}`
- 权限：`accounts:admin_users`

请求体：

```json
{
  "remark": "新的备注",
  "status": "disabled"
}
```

字段说明：

| 字段     | 类型                  | 必填 | 约束                 |
| -------- | --------------------- | ---- | -------------------- |
| `remark` | `string \| null`      | 否   | 有值时长度 `1..=255` |
| `status` | `enabled \| disabled` | 是   | 后台用户状态         |

成功响应 `data`：

```json
{
  "userId": "1b1f4e1d-5b4f-4d25-ae07-520f587f8d13",
  "displayId": "zhangsan",
  "displayName": "张三",
  "remark": "新的备注",
  "status": "disabled",
  "roles": [
    {
      "id": 1,
      "name": "系统管理员",
      "code": "system_admin"
    }
  ]
}
```

补充说明：

- 仅允许修改 `remark` 和 `status`
- 非 `root` 用户不能更新带有 `root` 角色的后台用户

## 4. 删除后台用户

- 方法：`DELETE`
- 路径：`/api/admin/account/admin-users/{userId}`
- 权限：`accounts:admin_users`

成功响应 `data`：

```json
null
```

补充说明：

- 删除后台用户时会同步删除该用户的角色绑定
- 非 `root` 用户不能删除带有 `root` 角色的后台用户

## 5. 创建角色

- 方法：`POST`
- 路径：`/api/admin/access/roles`
- 权限：`access_control:roles`

请求体：

```json
{
  "name": "系统管理员",
  "code": "system_admin"
}
```

字段说明：

| 字段   | 类型     | 必填 | 约束                                 |
| ------ | -------- | ---- | ------------------------------------ |
| `name` | `string` | 是   | 长度 `1..=64`                        |
| `code` | `string` | 是   | 长度 `1..=64`，且不能为保留值 `root` / `admin` / `support` |

成功响应 `data`：

```json
{
  "id": 1,
  "name": "系统管理员",
  "code": "system_admin"
}
```

## 6. 查询角色列表

- 方法：`GET`
- 路径：`/api/admin/access/roles`
- 权限：`access_control:roles`

补充说明：

- `root` 角色不会出现在列表中

成功响应 `data`：

```json
[
  {
    "id": 1,
    "name": "系统管理员",
    "code": "system_admin"
  }
]
```

## 7. 删除角色

- 方法：`DELETE`
- 路径：`/api/admin/access/roles/{roleId}`
- 权限：`access_control:roles`

成功响应 `data`：

```json
null
```

补充说明：

- `root` / `admin` / `support` 角色是保留角色，不能删除
- 删除角色时会同步删除该角色的用户绑定和权限绑定

## 8. 查询总权限配置树

- 方法：`GET`
- 路径：`/api/admin/access/permissions`
- 权限：`access_control:role_permissions`

成功响应 `data`：

```json
[
  {
    "id": 1,
    "name": "Dashboard",
    "kind": "group",
    "children": []
  },
  {
    "id": 2,
    "name": "Accounts",
    "kind": "group",
    "children": [
      {
        "id": 3,
        "name": "Admin Users",
        "kind": "group",
        "children": []
      },
      {
        "id": 4,
        "name": "App Users",
        "kind": "group",
        "children": []
      }
    ]
  }
]
```

补充说明：

- 返回全部权限节点的树形结构，供角色配置使用
- 不提供权限节点创建、修改或删除接口

## 9. 查询角色权限配置树

- 方法：`GET`
- 路径：`/api/admin/access/roles/{roleId}/permissions`
- 权限：`access_control:role_permissions`

路径参数：

| 参数     | 类型      | 必填 | 说明    |
| -------- | --------- | ---- | ------- |
| `roleId` | `integer` | 是   | 角色 ID |

成功响应 `data`：

```json
[
  {
    "id": 1,
    "name": "Dashboard",
    "kind": "group",
    "checked": false,
    "children": []
  },
  {
    "id": 2,
    "name": "Accounts",
    "kind": "group",
    "checked": false,
    "children": [
      {
        "id": 3,
        "name": "Admin Users",
        "kind": "group",
        "checked": true,
        "children": []
      }
    ]
  }
]
```

补充说明：

- 返回树形结构给前端配置使用
- 不返回权限码，避免后台直接感知内部权限命名

## 10. 更新角色权限配置

- 方法：`PUT`
- 路径：`/api/admin/access/roles/{roleId}/permissions`
- 权限：`access_control:role_permissions`

请求体：

```json
{
  "permissionIds": [2, 3, 4]
}
```

字段说明：

| 字段            | 类型        | 必填 | 约束                          |
| --------------- | ----------- | ---- | ----------------------------- |
| `permissionIds` | `integer[]` | 是   | 前端权限树中勾选的权限节点 ID |

成功响应 `data`：同“查询角色权限配置树”。

补充说明：

- 更新不是增量修改
- 服务端会先删除该 `roleId` 下所有 `admin_role_permissions` 记录，再插入本次提交的 `permissionIds`
- 非 `root` 用户不能修改 `root` 角色的权限

## 11. 创建菜单

- 方法：`POST`
- 路径：`/api/admin/access/menus`
- 权限：`access_control:role_permissions`

请求体：

```json
{
  "name": "设备管理",
  "parentId": null
}
```

字段说明：

| 字段       | 类型              | 必填 | 约束          |
| ---------- | ----------------- | ---- | ------------- |
| `name`     | `string`          | 是   | 长度 `1..=64` |
| `parentId` | `integer \| null` | 否   | 父菜单 ID     |

成功响应 `data`：

```json
{
  "id": 1,
  "name": "设备管理",
  "parentId": null
}
```

## 12. 查询菜单列表

- 方法：`GET`
- 路径：`/api/admin/access/menus`
- 权限：`access_control:role_permissions`

成功响应 `data`：

```json
[
  {
    "id": 1,
    "name": "设备管理",
    "parentId": null
  }
]
```

## 13. 查询用户角色配置

- 方法：`GET`
- 路径：`/api/admin/account/admin-users/{userId}/roles`
- 权限：`accounts:admin_users`

路径参数：

| 参数     | 类型     | 必填 | 说明                              |
| -------- | -------- | ---- | --------------------------------- |
| `userId` | `string` | 是   | 目标用户 ID，业务层会按 UUID 解析 |

成功响应 `data`：

```json
[
  {
    "id": 1,
    "name": "系统管理员",
    "code": "system_admin",
    "checked": true
  },
  {
    "id": 2,
    "name": "客服",
    "code": "support",
    "checked": false
  }
]
```

补充说明：

- 返回所有可配置角色，并用 `checked` 标识目标用户是否拥有该角色
- 非 `root` 用户不会看到 `root` 角色

## 14. 更新用户角色配置

- 方法：`PUT`
- 路径：`/api/admin/account/admin-users/{userId}/roles`
- 权限：`accounts:admin_users`

请求体：

```json
{
  "roleIds": [1, 2]
}
```

字段说明：

| 字段      | 类型        | 必填 | 约束                        |
| --------- | ----------- | ---- | --------------------------- |
| `roleIds` | `integer[]` | 是   | 前端角色配置中勾选的角色 ID |

成功响应 `data`：同“查询用户角色配置”。

补充说明：

- 更新不是增量修改
- 服务端会先删除该 `userId` 下所有 `admin_user_roles` 记录，再插入本次提交的 `roleIds`
- 非 `root` 用户不能给别人分配 `root` 角色
- 非 `root` 用户不能修改带有 `root` 角色的后台用户

## 15. 查询当前用户权限

- 方法：`GET`
- 路径：`/api/admin/me/permissions`
- 权限：有效后台用户

成功响应 `data`：

```json
{
  "userId": "1b1f4e1d-5b4f-4d25-ae07-520f587f8d13",
  "roleCodes": ["system_admin"],
  "permissionCodes": ["accounts:admin_users", "access_control:roles"]
}
```

补充说明：

- 登录后前端可用此接口判断当前用户能力
- `root` 角色不受角色权限配置管理，`admin_role_permissions` 不需要保存 `root` 权限
- 如果当前用户拥有 `root` 角色，服务端按最高权限处理，并返回全部权限码

## 16. 查询当前用户菜单树

- 方法：`GET`
- 路径：`/api/admin/me/menus`
- 权限：有效后台用户

成功响应 `data`：

```json
[
  {
    "id": 1,
    "name": "Dashboard",
    "parentId": null,
    "children": []
  },
  {
    "id": 2,
    "name": "Accounts",
    "parentId": null,
    "children": [
      {
        "id": 3,
        "name": "Admin Users",
        "parentId": 2,
        "children": []
      },
      {
        "id": 4,
        "name": "App Users",
        "parentId": 2,
        "children": []
      }
    ]
  },
  {
    "id": 5,
    "name": "Access Control",
    "parentId": null,
    "children": [
      {
        "id": 6,
        "name": "Roles",
        "parentId": 5,
        "children": []
      },
      {
        "id": 7,
        "name": "Role Permissions",
        "parentId": 5,
        "children": []
      },
      {
        "id": 8,
        "name": "App Roles",
        "parentId": 5,
        "children": []
      },
      {
        "id": 9,
        "name": "App Role Permissions",
        "parentId": 5,
        "children": []
      }
    ]
  }
]
```

补充说明：

- 如果当前用户拥有 `root` 角色，返回全部菜单树
- 非 `root` 用户只返回有权限命中的菜单，以及这些菜单的祖先节点

## 17. 查询 App 用户列表

- 方法：`GET`
- 路径：`/api/admin/account/app-users`
- 权限：`accounts:app_users`

查询参数：

| 参数            | 类型               | 必填 | 默认值 | 说明                                                                                   |
| --------------- | ------------------ | ---- | ------ | -------------------------------------------------------------------------------------- |
| `page`          | `integer`          | 否   | `1`    | 页码，从 1 开始                                                                        |
| `pageSize`      | `integer`          | 否   | `20`   | 每页数量，最大 `100`                                                                   |
| `keyword`       | `string`           | 否   | -      | 关键字；匹配 `displayId` / `displayName` / `remark`，如果是合法 UUID 也会匹配 `userId` |
| `status`        | `string`           | 否   | -      | 用户状态：`enabled` / `disabled`，精确匹配                                             |
| `createdAtFrom` | `string(datetime)` | 否   | -      | 创建时间起点，RFC3339，例如 `2026-04-27T00:00:00Z`                                     |
| `createdAtTo`   | `string(datetime)` | 否   | -      | 创建时间终点，RFC3339                                                                  |
| `updatedAtFrom` | `string(datetime)` | 否   | -      | 更新时间起点，RFC3339                                                                  |
| `updatedAtTo`   | `string(datetime)` | 否   | -      | 更新时间终点，RFC3339                                                                  |
| `sortBy`        | `string`           | 否   | -      | 排序字段：`createdAt` / `updatedAt`；不传时默认按 `updatedAt` 排序                      |
| `sortOrder`     | `string`           | 否   | -      | 排序方向：`asc` / `desc`；不传时默认 `desc`                                             |

示例：

```http
GET /api/admin/account/app-users?page=1&pageSize=20&keyword=App%20User&status=enabled&createdAtFrom=2026-04-27T00:00:00Z&sortBy=createdAt&sortOrder=desc
```

服务端查询语义：

```sql
WHERE
  (
    display_id LIKE :keyword
    OR display_name LIKE :keyword
    OR remark LIKE :keyword
    OR user_id = :keyword
  )
  AND status = :status
  AND created_at >= :created_at_from
  AND created_at <= :created_at_to
  AND updated_at >= :updated_at_from
  AND updated_at <= :updated_at_to
ORDER BY updated_at DESC
-- 或按参数指定：created_at|updated_at ASC|DESC
```

成功响应 `data`：

```json
{
  "items": [
    {
      "userId": "1b1f4e1d-5b4f-4d25-ae07-520f587f8d13",
      "displayId": "zhangsan",
      "displayName": "张三",
      "remark": "测试 App 用户",
      "status": "enabled",
      "createdAt": "2026-04-27T00:00:00Z",
      "updatedAt": "2026-04-27T00:00:00Z",
      "roles": []
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

补充说明：

- App 用户只能通过普通客户端注册流程产生，后台不提供新增 App 用户接口
- HTTP JSON 字段统一使用 `camelCase`
- 返回对象会包含数据库时间字段：主数据对象返回 `createdAt`、`updatedAt`；授权关系表如果后续直接返回行数据，只返回 `createdAt`

## 18. 查询 App 用户统计

- 方法：`GET`
- 路径：`/api/admin/account/app-users/metrics`
- 权限：`accounts:app_users`

成功响应 `data`：

```json
{
  "total": 100,
  "enabled": 10,
  "disabled": 0,
  "multiRole": 0
}
```

字段说明：

| 字段        | 类型      | 说明                          |
| ----------- | --------- | ----------------------------- |
| `total`     | `integer` | App 用户总数                  |
| `enabled`   | `integer` | 启用账号数                    |
| `disabled`  | `integer` | 停用账号数                    |
| `multiRole` | `integer` | 同时挂载 2 个及以上角色账号数 |

## 19. 更新 App 用户

- 方法：`PATCH`
- 路径：`/api/admin/account/app-users/{userId}`
- 权限：`accounts:app_users`

请求体：

```json
{
  "remark": "新的备注",
  "status": "disabled"
}
```

成功响应 `data`：`AppUserResponse`。

## 20. 删除 App 用户

- 方法：`DELETE`
- 路径：`/api/admin/account/app-users/{userId}`
- 权限：`accounts:app_users`

成功响应 `data`：

```json
null
```

## 21. 查询 App 用户角色配置

- 方法：`GET`
- 路径：`/api/admin/account/app-users/{userId}/roles`
- 权限：`accounts:app_users`

成功响应 `data`：

```json
[
  {
    "id": 1,
    "name": "普通用户",
    "code": "user",
    "checked": true
  }
]
```

## 22. 更新 App 用户角色配置

- 方法：`PUT`
- 路径：`/api/admin/account/app-users/{userId}/roles`
- 权限：`accounts:app_users`

请求体：

```json
{
  "roleIds": [1, 2]
}
```

成功响应 `data`：同“查询 App 用户角色配置”。

## 23. 创建 App 角色

- 方法：`POST`
- 路径：`/api/admin/access/app-roles`
- 权限：`access_control:app_roles`

请求体：

```json
{
  "name": "普通用户",
  "code": "user"
}
```

字段说明：

| 字段   | 类型     | 必填 | 约束                          |
| ------ | -------- | ---- | ----------------------------- |
| `name` | `string` | 是   | 长度 `1..=64`                 |
| `code` | `string` | 是   | 长度 `1..=64`，且不能为保留值 `free` |

成功响应 `data`：

```json
{
  "id": 1,
  "name": "普通用户",
  "code": "user"
}
```

## 24. 查询 App 角色列表

- 方法：`GET`
- 路径：`/api/admin/access/app-roles`
- 权限：`access_control:app_roles`

成功响应 `data`：`RoleResponse[]`。

## 25. 删除 App 角色

- 方法：`DELETE`
- 路径：`/api/admin/access/app-roles/{roleId}`
- 权限：`access_control:app_roles`

成功响应 `data`：

```json
null
```

补充说明：

- `free` 角色是保留角色，不能删除

## 26. 查询 App 权限树

- 方法：`GET`
- 路径：`/api/admin/access/app-permissions`
- 权限：`access_control:app_role_permissions`

成功响应 `data`：

```json
[
  {
    "id": 1,
    "name": "个人中心",
    "kind": "group",
    "children": [
      {
        "id": 2,
        "name": "查看个人资料",
        "kind": "action",
        "children": []
      }
    ]
  }
]
```

## 27. 查询 App 角色权限配置

- 方法：`GET`
- 路径：`/api/admin/access/app-roles/{roleId}/permissions`
- 权限：`access_control:app_role_permissions`

成功响应 `data`：带 `checked` 字段的 App 权限树。

## 28. 更新 App 角色权限配置

- 方法：`PUT`
- 路径：`/api/admin/access/app-roles/{roleId}/permissions`
- 权限：`access_control:app_role_permissions`

请求体：

```json
{
  "permissionIds": [1, 2, 3]
}
```

成功响应 `data`：同“查询 App 角色权限配置”。
