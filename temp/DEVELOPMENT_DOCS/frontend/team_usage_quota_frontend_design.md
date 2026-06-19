# Account Usage Quota Frontend Design

## 需求摘要

本阶段只实现账号级问答次数限流展示与后台管理，不做界面体验重设计、角色个性化和付费。

前端需要完成：

1. Web 注册页继续把学校、参赛队伍名提交到用户资料，作为统计字段。
2. Web 对话页展示当前账号的问答额度、已用次数、剩余次数和状态提示。
3. Admin 后台账号用量页查看每个账号的问答次数、剩余额度、Token 统计、学校和队伍。
4. Admin 后台允许有权限的管理员修改单个账号问答上限、启停状态和备注。
5. Token 只展示统计，不作为限流配置。

## 页面与路由

### Web 注册页

- 注册资料字段：
  - `schoolName`
  - `teamName`
- 说明：
  - 两个字段用于后台按学校和队伍统计，不作为额度主体。
  - 真正额度主体是账号 `userId`。

### Web 对话页

- API：`GET /api/app/usage/quota`
- 展示：
  - 当前账号前 `N` 次 AI 问答可使用
  - 已用 `X` 次
  - 剩余 `Y` 次
  - 当前账号 ID
  - 学校/队伍统计归属
  - 停用或用完时提示联系赛事或平台管理员
- 不展示 token。

### Admin 账号用量页

- 路由：`/usage/team-usage`
- 页面目录：`frontend/apps/admin/src/pages/protected/usage/team-usage/`
- 导航：
  - 分组：用量管理
  - 菜单：账号用量
  - 权限码：`usage:team_usage`
- 说明：
  - 路由和目录保留 `team-usage` 是历史兼容命名。
  - 页面展示和业务行为按账号用量处理。

页面结构：

- `index.tsx`：装配 `MetricCards`、`DataTable`
- `team-usage-data.tsx`：服务端分页请求、API 响应到表格行映射、指标卡数据
- `metrics/index.tsx`：从当前页快照生成摘要卡片
- `table/`：列、查询字段、状态映射、行操作、表格配置
- `dialogs/edit-team-quota-dialog.tsx`：账号额度编辑内容

## API 映射

### 列表

- 前端方法：`listTeamUsagesApi`
- 后端接口：`GET /api/admin/usage/accounts`
- DataTable 参数映射：
  - `page` -> `page`
  - `pageSize` -> `pageSize`
  - `query.keyword` -> `keyword`
  - `query.status` -> `status`
  - sort:
    - `display_id` -> `displayId`
    - `question_used` -> `questionUsed`
    - `question_remaining` -> `questionRemaining`
    - `lastUsedAt` -> `lastUsedAt`
    - `updatedAt` -> `updatedAt`

返回行映射：

- `id`：账号额度记录 ID，用于编辑接口 path。
- `userId`：真实限流主体。
- `displayId` / `displayName`：后台可读账号信息。
- `schoolName` / `teamName`：统计字段。
- `questionLimit` / `questionUsed` / `questionRemaining`：限流相关次数。
- `tokenUsed`：统计信息。

### 更新账号额度

- 前端方法：`updateTeamUsageQuotaApi`
- 后端接口：`PATCH /api/admin/usage/accounts/{usageId}/quota`
- 请求字段：
  - `questionLimit`
  - `status`
  - `remark`

## 登录态和权限

- Admin 页面复用既有后台登录态。
- 路由由 Admin protected layout 保护。
- 菜单可见性使用本地导航配置中的 `permissionCode: "usage:team_usage"`。
- 服务端仍是最终权限校验点。
- 普通学生账号没有后台入口，不能查看或修改任何账号额度。

## 异常与空状态

- DataTable 继续使用 `@workspace/app-kit` 的内建 loading/error/empty 状态。
- 额度编辑表单：
  - 数值字段按非负整数处理。
  - 最终范围仍由后端校验：`0..=1000000`。
- 用户端 SSE 错误提示：
  - `-12068`：当前账号 AI 问答次数已用完，请联系赛事或平台管理员增加额度。
  - `-12069`：当前账号 AI 问答已暂停，请联系管理员。

## 不做内容

- 不做 UI 全量重设计
- 不做 AI 角色人格化/深刻性调整
- 不做付费套餐、支付或订阅
