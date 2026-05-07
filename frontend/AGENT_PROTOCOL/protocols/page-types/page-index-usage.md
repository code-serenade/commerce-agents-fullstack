# 页面 index 使用协议

本协议约束所有 app 页面目录中的 `index.tsx`。

它不区分页面是否使用 `DataTable`，也不区分页面是列表、配置、授权、表单还是详情。只要是页面入口文件，就必须优先遵守本协议。

---

## 1. 定位

`index.tsx` 是页面装配入口，不是页面实现仓库。

它的职责是让人和 AI 一眼看懂：

- 这个页面由哪些模块组成
- 页面级 hook 如何连接这些模块
- 主要交互入口在哪里

它不负责解释：

- 数据怎么请求
- 树怎么转换
- 表格怎么过滤排序
- 弹窗怎么提交
- 表单怎么校验
- 复杂 UI 细节怎么排布

一句话：

> `index.tsx` 负责“看见页面结构”，其他文件负责“解释页面怎么工作”。

---

## 2. 允许出现在 index.tsx 的内容

优先允许：

- 页面根布局容器
- 页面标题组件
- 页面主要区域组件装配
- 一个页面级 hook，例如 `useXxxPage`
- 少量纯粹的 props 连接
- 简单空状态装配

可以接受的轻量代码：

- `const page = useXxxPage()`
- `<PageHeading />`
- `<MainPanel {...page.mainPanel} />`
- `<EmptyState />`

---

## 3. 不允许长期留在 index.tsx 的内容

以下内容默认必须拆出去：

- React Query / mutation 细节
- API import 和请求函数调用
- `filter / sort / paginate`
- 树结构转换、flatten、checked 收集
- 表格列定义
- 表格 row actions
- 表单字段 JSX
- 弹窗内容 JSX
- 复杂 `useMemo` / `useEffect` 状态同步
- 大段重复卡片、表格、列表、树节点渲染
- 业务枚举 label map

如果 `index.tsx` 开始出现这些内容，应立即拆到对应文件。

---

## 4. 推荐结构

### DataTable 列表页

继续遵守：

- `AGENT_PROTOCOL/protocols/page-types/shared-query-page.md`
- `AGENT_PROTOCOL/protocols/page-types/datatable-local-usage.md`

典型结构：

```txt
<page>/
  index.tsx
  types.ts
  constants.ts
  <page>-data.tsx
  metrics/
  table/
  dialogs/
```

### 复杂交互页

例如 agent、chat、workbench、editor、builder、canvas、flow、console 等页面：

```txt
<page>/
  index.tsx
  types.ts
  constants.ts
  use-<page>-page.ts
  <page>-data.ts
  domain/
    logic.ts
  view-model/
    panel-view-model.ts
    event-handlers.ts
  components/
    page-header.tsx
    primary-panel.tsx
    activity-panel.tsx
    detail-panel.tsx
```

说明：

- `index.tsx` 只拼接 `<PageHeader />`、`<PrimaryPanel />`、`<ActivityPanel />` 等主模块。
- `use-<page>-page.ts` 只调用数据入口、维护页面级状态、组合 view model、暴露 handlers。
- `<page>-data.ts` 只处理 API/query/mutation 或演示数据入口。
- `domain/logic.ts` 放纯函数和可测试业务计算。
- `view-model/` 放事件、数据到组件 props 的转换。
- `components/` 只渲染已经准备好的 props，不重新计算业务模型。

复杂交互页必须继续遵守：

- `AGENT_PROTOCOL/protocols/page-types/complex-interaction-page.md`

### 非 DataTable 配置页

例如授权、菜单配置、设置页：

```txt
<page>/
  index.tsx
  types.ts
  constants.ts
  use-<page>-page.ts
  <page>-data.ts
  components/
  <domain>/
    logic.ts
```

说明：

- `use-<page>-page.ts`：页面状态、派生 view model、事件 handler。
- `<page>-data.ts`：请求、mutation、query key 消费。
- `components/`：页面局部 UI 片段。
- `<domain>/logic.ts`：树、映射、统计、纯函数逻辑。

---

## 5. index.tsx 大小建议

默认目标：

- 30-80 行优先
- 超过 120 行必须重新评估
- 超过 160 行默认视为违反协议

注意：把 160 行以上的内容从 `index.tsx` 挪到一个巨大的 `components/<panel>.tsx` 或 `use-<page>-page.ts` 不算完成拆分。拆分的目标是职责边界清楚，而不是只移动代码。

例外：

- 页面本身极简单，没有拆分价值
- 临时迁移中间态，但最终提交前应收敛

---

## 6. 命名规则

- 页面级 hook 用 `use<PascalPageName>Page`
- 请求文件优先用 `<page>-data.ts`
- 纯逻辑放 `logic.ts`
- 页面 UI 组件放 `components/`
- 不要把可复用后台模式藏在页面本地；若第二个页面复用，应考虑上移到 `@workspace/app-kit`

---

## 7. 完成标准

页面完成时，至少满足：

- `index.tsx` 不直接 import API。
- `index.tsx` 不包含复杂数据转换。
- `index.tsx` 不包含大段表格、树、弹窗、表单 JSX。
- 页面级状态和派生值集中在 hook 中。
- 大型展示模型、事件映射和纯逻辑继续拆到 `view-model/` 与 `domain/`。
- 页面组件文件只接收 props 并渲染局部 UI，不混合 API、reducer 和复杂派生逻辑。
- 页面子模块职责能从文件名看出来。
- 后续 AI 能通过 `index.tsx` 先理解页面结构，再进入对应模块修改细节。
