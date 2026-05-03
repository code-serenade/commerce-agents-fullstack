# apps/admin 协议

`apps/admin` 是商用后台应用，不是组件实验场，也不是共享层的临时垃圾桶。

它的目标是复用框架层能力，稳定地搭建后台页面，让 AI 在后台开发时优先做页面装配、权限编排和业务流程，而不是在页面里重新发明一套后台 UI 系统。

## 1. 宪章

- `admin` 是真实商用 app。
- `admin` 负责后台路由、页面、布局、状态编排、权限编排与业务流程。
- `admin` 不是共享 UI 沉淀层。
- `admin` 承载标准后台基础能力，尤其是账号、角色、权限和菜单可见性管理。
- 非程序员指挥 AI 时，`admin` 必须尽量表现为“按协议装配”，而不是“自由发挥搭页面”。

## 2. admin 在整个框架中的位置

- 后台页面、后台路由、后台权限流程：`apps/admin`
- 共享后台页面、壳层与复合装配：`@workspace/app-kit`
- 稳定基础交互组件：`@workspace/ui-components`
- primitive 原语能力：`@workspace/ui-core`
- 通用服务、请求、query、i18n：`@workspace/services`

AI 在 `admin` 中写代码时，默认任务应理解为：

- 组合已有协议
- 接入后台业务
- 保持后台结构统一

而不是：

- 重新定义组件协议
- 在页面里下沉一层共享体系
- 用一次性代码覆盖框架边界

## 3. 默认消费顺序

AI 在 `admin` 中写页面时，默认顺序必须是：

1. 先找 `@workspace/app-kit`
2. 再找 `@workspace/ui-components`
3. 再找 `@workspace/services`
4. 最后才写 `admin` 本地实现

不得默认从 `ui-core` 直接起手写页面。

如果某个需求需要大量直接操作 primitive，AI 必须先重新判断该能力是否应该进入 `ui-components` 或 `@workspace/app-kit`。

## 4. 本地目录职责

- `api/`：`admin` 应用侧 API 聚合，只负责后台应用消费入口，不重写共享请求基础设施。
- `navigation/`：后台菜单配置、菜单可见性判断、路由激活匹配。
- `shell/`：后台应用壳层装配，如 sidebar、top bar、账户区、权限加载后的导航跳转。
- `layouts/`：页面布局与路由容器。
- `pages/`：后台页面。
- `routes/`：后台路由定义、懒加载和受保护页面入口。
- `store/`：跨页面状态，如鉴权、权限、用户信息恢复。
- `test/`：应用级测试。

若某段代码不属于上述任一职责，AI 必须重新判断落点。

## 5. 适合放在 admin 本地的内容

- 只服务于 `admin` 的页面组件
- 只服务于 `admin` 的局部布局
- 与后台业务流程强绑定的页面逻辑
- 只服务于后台权限模型的页面装配
- 只服务于后台菜单结构的本地映射

以下情况默认不应继续放在 `admin`：

- 第二个 app 明显也会复用
- 多个后台页面正在重复同一套结构
- 页面已经在定义一个“通用后台模式”
- 需要给 AI 提供统一参考，而不是某一页私有实现

出现这些信号时，应优先上移到 `packages/*`。

## 6. 后台页面默认策略

### 列表页

- 列表页优先使用 `@workspace/app-kit` 的 `DataTable`。
- 不要在页面里重写一套 query header、bulk action、row action、分页排序骨架。
- 如 `DataTable` 缺能力，应先判断是否属于稳定后台模式；若是，优先补到 `app`，不要先在 `admin` 私搭旁路。

#### DataTable 的定位

`admin` 中的 `DataTable` 不是演示组件，也不是越做越重的万能表格。

它是后台列表页的统一控制协议，用来让 AI 和业务页面按同一种结构组织：

- 列定义
- 查询字段
- 分页
- 排序
- 行操作
- 批量操作开关
- 新增入口
- loading / empty / error
- 表格高度和滚动结构

页面只负责提供业务数据、业务字段、业务动作和少量页面状态，不应复制 `DataTable` 内部 header、分页、selection、row action 或 dialog 骨架。

#### DataTable 的两种合法数据模式

`admin` 目前只允许两种 DataTable 数据模式。AI 必须先判断模式，再写代码。

**模式 A：一次请求全量数据，由前端管理**

适用场景：

- 数据量明确较小或中等
- 页面需要 `MetricCards` / `DataTable` 共用同一份 rows
- 服务端接口只提供列表全量，或当前后台页面更适合本地查询

规则：

- 页面 data hook 先通过 React Query 拉取全量 rows。
- `DataTable.fetchData` 必须从 query cache 读取 rows。
- 查询、排序、分页在前端完成。
- `query` 变化不应重新请求服务端。
- 新增、编辑、删除成功后，统一 invalidate/refetch 对应 query key。

当前参考页面：

- `accounts/admin-users`
- `access/roles`
- `access/app-roles`

标准形态：

```tsx
const fetchData = async ({ page, pageSize, query, sort, signal }) => {
  void signal
  const rows = await queryClient.ensureQueryData({ queryKey, queryFn })
  const filtered = filterRows(rows, query)
  const sorted = sortRows(filtered, sort)
  return paginateRows(sorted, page, pageSize)
}
```

**模式 B：分页请求完全交给服务端控制**

适用场景：

- 数据量可能很大
- 服务端已经提供分页、筛选、排序参数
- 页面不能或不应该一次性拉取全部 rows
- summary / metrics 可以由独立接口提供

规则：

- `DataTable.fetchData` 直接把 `page`、`pageSize`、`query`、`sort` 映射成接口参数。
- 服务端返回当前页 `items` 和全量 `total`。
- 前端不再对当前页做二次全量过滤或全量分页。
- query 字段必须和服务端查询协议一一映射，不能写只有本地才生效的筛选。
- 新增、编辑、删除成功后，刷新列表所依赖的接口或用 `key` 触发 DataTable 重新拉取；metrics 走自己的 query key。

当前参考页面：

- `accounts/app-users`

标准形态：

```tsx
const fetchData = async ({ page, pageSize, query, sort, signal }) => {
  void signal
  const params = buildListParams(page, pageSize, query, sort)
  const response = await listRowsApi(params)

  return {
    items: response.items.map(mapRow),
    total: response.total,
  }
}
```

#### DataTable 文件拆分规则

列表页必须优先保持以下结构：

- `index.tsx`
  - 只装配页面主结构、`MetricCards`、`DataTable`、页面级 dialog 状态
  - 不直接 import `./table/columns`、`./table/query-fields`、`./table/logic`
- `<page>-data.tsx`
  - 负责 React Query、接口请求、数据映射、`fetchData`
  - 本地模式可以直接 import `./table/logic` 和 `./table/sort`
  - 不依赖 `./table` 入口，避免 data 层反向依赖表格装配层
- `table/index.ts`
  - 收口传给 `DataTable` 的配置
  - 可以组合 columns、query fields、row actions、默认 page size、开关
- `table/columns.tsx`
  - 只定义列
- `table/query-fields.ts`
  - 只定义 `query.builtInFields` 和 `query.fields`
- `table/logic.ts`
  - 只放本地 filter / sort / paginate 等纯逻辑
  - 服务端模式若没有本地处理，不要为了凑结构硬写复杂 logic
- `table/row-actions.ts`
  - 只定义行操作
- `table/status.ts` / `table/sort.ts`
  - 状态映射、排序字段映射等稳定纯逻辑

#### DataTable 禁止事项

- 禁止把本地全量模式和服务端分页模式混在一个 `fetchData` 里。
- 禁止在服务端分页模式下先请求一页，再做本地全量分页假象。
- 禁止在本地全量模式下让 `DataTable` 和 `MetricCards` 各请求一份相同列表数据。
- 禁止把 filter / sort / paginate 堆在页面 `index.tsx`。
- 禁止为了单页特殊布局重写 DataTable header、tail、row action 或分页。
- 禁止把演示页代码作为生产页面模板；真实页面必须以 `admin-users`、`app-users`、`roles`、`app-roles` 等精心设计过的页面为参考。

### 导航与壳层

- 后台导航优先使用已有 sidebar / top bar / app shell 体系。
- 菜单显隐应由权限数据和本地菜单映射共同决定，不要把权限判断散落到每个页面按钮里。
- 后台默认入口跳转应在壳层或导航逻辑统一处理，不要让多个页面各自写一遍“无权限时去哪”。

### 路由与认证布局

`admin` 是后台系统，未登录用户不应看到后台壳层、菜单或任何业务信息。

默认规则：

- 未登录状态只挂载认证路由，例如 login。
- login 使用 `@workspace/app-kit/login` 的共享登录视图/表单协议；`admin` 本地只保留后台登录后的业务流程。
- `admin` 登录成功后恢复的是后台用户权限和后台菜单访问能力，不能复用或触发 `web` 的 app user 初始化流程。
- 已登录并完成权限恢复后才挂载后台 app layout / shell 与 protected routes。
- `admin` 可以使用 `isAuthenticated ? protectedRoutes : publicRoutes` 的整棵路由切换模式。
- 不要为了和 `web` 保持形式一致，把后台 app shell 暴露给未登录用户。
- 公开可浏览页面不应默认放在 `admin`；如果确实需要公开后台入口，必须单独评估其布局和信息暴露边界。

### 表单与详情页

- 简单表单与基础交互优先使用 `ui-components`。
- 表单 schema、默认值转换、接口提交映射应集中组织，不要散落在 JSX 里。
- 详情页、设置页、配置页可以有后台本地结构，但不要长出可复用组件协议后仍留在页面里。

### 权限页

- 权限管理页可以表达后台专有流程，但权限树、菜单码、按钮码、API 码的规则必须清晰且可追踪。
- 页面可以承载权限业务语义，但不应私自创造第二套通用树控件协议。

## 7. AI 默认判断规则

AI 在 `admin` 动手前，至少先回答下面 5 个问题：

1. 这是后台业务页面，还是共享后台模式？
2. 现有 `app` / `ui-components` 是否已经有合法入口？
3. 这段实现是一次性页面逻辑，还是第二个页面也会复用？
4. 这次改动会不会把共享问题留在 `admin` 本地？
5. 改完后，其他 AI 再看这段代码时，能不能学到正确边界？

如果第 2 个问题答案是“有”，优先复用。

如果第 3 个问题答案是“会复用”，优先上移。

如果第 4 个问题答案是“会”，应停止继续在 `admin` 内扩写。

## 8. 何时必须上移到 packages

以下需求默认不应继续留在 `admin`：

- 第二个列表页也要用同一套 header / query / action 结构
- 第二个 app 也会使用同一套后台壳层片段
- 为了多个页面新增了统一的页面级 UI 组合模式
- 页面里出现稳定可复用的权限选择、筛选、摘要、壳层装配模式
- 你已经开始写“让别的页面以后也能用”的 API

上移判断规则：

- primitive 行为：去 `ui-core`
- 简单稳定组件：去 `ui-components`
- 后台复合装配：去 `app`
- 通用服务能力：去 `services`

## 9. 禁止事项

- 禁止引入第二套第三方 UI 设计系统作为主组件层
- 禁止在页面里直接散落 primitive 写法
- 禁止把共享问题留在 `admin` 本地临时修补后不回收
- 禁止把 showcase 风格代码整块搬进生产后台
- 禁止直接复制 `app` 或 `ui-components` 内部实现到页面
- 禁止因为赶进度绕过 `api/`、`navigation/`、`store/` 等本地职责边界
- 禁止把权限码、菜单码、路由匹配规则散落在多个无关文件中重复维护
- 禁止在 `navigation/menu-config.tsx` 中省略稳定 `id`，或用 `label`、`href`、`#` 之类占位值充当导航身份标识

## 10. admin 对整个框架的意义

- `admin` 是商用主战场。
- `admin` 中沉淀出来的稳定后台模式，应优先回收到 `app`。
- `admin` 页面写法本身也应成为 AI 继续开发后台基础能力的合法参考。
- `admin` 的价值不是证明 AI 能把页面拼出来，而是证明 AI 能按协议稳定开发后台。

## 11. AI 交付标准

只有同时满足以下条件，`admin` 改动才算完成：

- 分层判断正确。
- 优先复用了现有共享层能力。
- 没有在页面中复制共享组件内部结构。
- 没有把应上移的后台模式继续堆在本地。
- 路由、导航、权限、状态各自放在正确职责目录。
- 新代码能作为后续 AI 继续开发后台的合法参考。

## 12. 门禁

涉及 `apps/admin` 的改动至少通过：

- `pnpm -C apps/admin lint`
- `pnpm -C apps/admin test`
- `pnpm -C apps/admin rules`
- `pnpm -C apps/admin build`
