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
- 页面子模块职责能从文件名看出来。
- 后续 AI 能通过 `index.tsx` 先理解页面结构，再进入对应模块修改细节。
