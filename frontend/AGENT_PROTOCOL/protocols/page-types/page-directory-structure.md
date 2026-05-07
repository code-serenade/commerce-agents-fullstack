# 页面目录结构协议

本协议约束所有 app 页面如何组织目录。

它是页面协议的总原则，优先级高于具体页面类型协议。无论页面是否接 API、是否只是模板演示、是否当前很小，都应先按本协议判断文件落点。

---

## 1. 核心原则

页面不是一个文件。

一个页面默认必须是一个子目录：

```txt
<page>/
  index.tsx
```

`index.tsx` 只负责页面装配，不负责承载页面全部实现。

如果页面开始出现以下任一内容，应立即拆文件：

- 静态演示数据
- 多组卡片、表格、图表、列表 JSX
- 图表数据映射
- 表格列、行操作、排序过滤
- 表单字段、弹窗内容
- 大段业务枚举、状态 label、颜色映射
- 复杂 hook、memo、effect
- 一个 `components/*.tsx` 文件开始同时承载状态编排、事件映射、数据转换和 JSX
- `use-<page>-page.ts` 开始超过“页面编排”职责，沉淀纯算法、reducer 或大段 view model 映射

框架演示页也必须遵守本规则。演示代码会被 AI 学习，不能因为“不接 API”就把页面写成单文件样例。

---

## 2. `index.tsx` 的职责

`index.tsx` 是页面入口和装配层。

允许：

- 页面根容器
- 页面主要区块装配
- 调用一个页面级 hook
- 少量 props 连接

不允许长期保留：

- 数据常量
- 复杂 JSX 片段
- 图表、表格、表单的完整实现
- 请求、转换、过滤、排序、统计逻辑
- 页面局部组件定义

目标：

- 30-80 行优先
- 超过 120 行必须拆分
- 超过 160 行默认违反协议

---

## 3. 默认目录骨架

普通后台页面：

```txt
<page>/
  index.tsx
  constants.ts
  types.ts
  <page>-data.ts
  components/
```

列表页：

```txt
<page>/
  index.tsx
  constants.ts
  types.ts
  <page>-data.tsx
  metrics/
  table/
  dialogs/
```

控制台 / dashboard / 分析页：

```txt
<page>/
  index.tsx
  <page>-data.ts
  metrics/
    index.tsx
  charts/
    index.tsx
  components/
```

复杂交互 / workbench / agent / chat / editor 页：

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
    <feature>-view-model.ts
    <feature>-events.ts
  components/
    <feature-panel>.tsx
```

这类页面必须继续阅读：

- `AGENT_PROTOCOL/protocols/page-types/complex-interaction-page.md`

设置 / 授权 / 复杂配置页：

```txt
<page>/
  index.tsx
  constants.ts
  types.ts
  use-<page>-page.ts
  <page>-data.ts
  components/
  <domain>/
    logic.ts
```

---

## 4. 文件职责

### `<page>-data.ts`

页面数据入口。

可以放：

- 静态演示数据
- 静态演示数据
- API 查询 hook
- API -> 页面数据转换

不放：

- 页面 JSX
- 图表 JSX
- 表格列 JSX
- reducer、事件分发器或 view model 装配

### `metrics/`

摘要指标区。

只负责：

- 指标区组件
- `rows/data -> metric items`

### `charts/`

图表区。

只负责：

- 页面局部图表组合
- 图表组件的数据接线
- 图表标题、说明、容器

图表基础能力优先来自 `@workspace/ui-components/stable/chart`。

### `components/`

页面局部 UI 片段。

适合放：

- 页面标题
- 局部表格
- 风险面板
- 状态面板
- 页面私有说明块

如果组件被第二个页面复用，应重新判断是否上移到 `@workspace/app-kit`。

不适合放：

- API / React Query / mutation wiring
- reducer、状态机、事件分发器
- 大量 `useMemo` / `useEffect` 派生逻辑
- 业务算法、树转换、消息归并、权限计算
- 可复用的 DataTable、panel、dialog、tabs、toolbar、pagination 等通用模式

`components/` 文件应该偏 presentational：接收已经整理好的 props，渲染页面局部 UI。

### `domain/`

页面私有纯逻辑。

适合放：

- 纯函数
- reducer 之外可测试的状态转换
- tree / graph / event / message / permission 等业务数据计算
- 排序、过滤、分组、统计

不放：

- React hook
- JSX
- API import
- i18n hook

### `view-model/`

页面私有展示模型层。

适合放：

- API/domain 数据到组件 props 的转换
- 事件到展示状态的映射
- panel、thread、toolbar、detail drawer 的 view model
- 只服务当前页面的展示枚举、状态标签和可见性规则

不放：

- JSX
- API 请求
- 共享 UI primitive
- 可复用 app-kit 模式

---

## 5. 与其他协议的关系

AI 写页面时默认阅读顺序：

1. `AGENT_PROTOCOL/protocols/apps/index.md`
2. 当前 app 的 `AGENT_PROTOCOL/protocols/apps/<app>.md`
3. 本协议
4. `AGENT_PROTOCOL/protocols/page-types/page-index-usage.md`
5. 具体页面类型协议，例如 `shared-query-page`、`datatable-local-usage`、`metric-cards-usage`
6. 复杂交互页面继续读 `complex-interaction-page`

本协议解决“页面必须怎么拆目录”。

具体页面类型协议解决“某类页面内部模块怎么协作”。

---

## 6. 完成标准

页面完成时必须满足：

- 页面以目录形式存在。
- `index.tsx` 不承载数据常量。
- `index.tsx` 不承载大段 JSX 实现。
- 页面局部图表、表格、指标、状态面板各自有清晰落点。
- 页面级 hook 没有膨胀成业务逻辑仓库。
- `components/` 没有同时承载 view model、reducer、API 和 JSX。
- 演示页也按真实页面结构组织。
- 后续 AI 能从 `index.tsx` 看见页面结构，再进入子模块修改细节。
