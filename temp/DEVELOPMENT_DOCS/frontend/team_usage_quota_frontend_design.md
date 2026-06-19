# Team Usage Quota Frontend Design

## 需求摘要

本阶段只实现队伍注册信息与限流管理，不做界面体验重设计、角色个性化和付费。

前端需要完成：

1. Web 注册页把学校、参赛队伍名作为必填字段提交给既有注册资料缓存流程。
2. Admin 后台新增队伍用量管理页，查看每个学校/队伍的问答次数、剩余额度、成员数量和状态。
3. Admin 后台允许有权限的管理员修改队伍问答上限、Token 上限、启停状态和备注。

## 页面与路由

### Web 注册页

- 文件：`frontend/apps/web/src/pages/auth/register/use-register-page.ts`
- 变更：
  - `schoolName` 设置 `required: true`
  - `teamName` 设置 `required: true`
  - 中英文文案移除“可选”

### Admin 队伍用量页

- 路由：`/usage/team-usage`
- 页面目录：`frontend/apps/admin/src/pages/protected/usage/team-usage/`
- 导航：
  - 分组：用量管理
  - 菜单：队伍用量
  - 权限码：`usage:team_usage`

页面结构：

- `index.tsx`：装配 `MetricCards`、`DataTable`、成员弹窗
- `team-usage-data.tsx`：服务端分页请求、API 响应到表格行映射、指标卡数据
- `metrics/index.tsx`：从当前页快照生成摘要卡片
- `table/`：列、查询字段、状态映射、行操作、表格配置
- `dialogs/edit-team-quota-dialog.tsx`：额度编辑内容
- `dialogs/team-members-dialog.tsx`：队伍成员查看

## API 映射

### 列表

- 前端方法：`listTeamUsagesApi`
- 后端接口：`GET /api/admin/usage/teams`
- DataTable 参数映射：
  - `page` -> `page`
  - `pageSize` -> `pageSize`
  - `query.keyword` -> `keyword`
  - `query.status` -> `status`
  - sort:
    - `member_count` -> `memberCount`
    - `question_used` -> `questionUsed`
    - `question_remaining` -> `questionRemaining`
    - `lastUsedAt` -> `lastUsedAt`
    - `updatedAt` -> `updatedAt`

### 更新额度

- 前端方法：`updateTeamUsageQuotaApi`
- 后端接口：`PATCH /api/admin/usage/teams/{teamId}/quota`
- 请求字段：
  - `questionLimit`
  - `tokenLimit`
  - `status`
  - `remark`

### 查看成员

- 前端方法：`listTeamUsageMembersApi`
- 后端接口：`GET /api/admin/usage/teams/{teamId}/members`
- 触发时机：用户点击行操作“查看成员”后请求，列表加载不做成员详情 N+1 请求。

## 登录态和权限

- Admin 页面复用既有后台登录态。
- 路由由 Admin protected layout 保护。
- 菜单可见性使用本地导航配置中的 `permissionCode: "usage:team_usage"`。
- 服务端仍是最终权限校验点；前端不自行判断是否有业务管理权限。

## 异常与空状态

- DataTable 继续使用 `@workspace/app-kit` 的内建 loading/error/empty 状态。
- 成员弹窗：
  - loading：显示成员加载中
  - empty：显示暂无成员
- 额度编辑表单：
  - 前端将空 Token 上限提交为 `null`
  - 数值字段按非负整数处理；最终范围仍由后端校验

## 不做内容

- 不做 UI 全量重设计
- 不做 AI 角色人格化/深刻性调整
- 不做付费套餐、支付或订阅
