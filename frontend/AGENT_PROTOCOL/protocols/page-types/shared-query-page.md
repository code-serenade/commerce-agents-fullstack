# 共享查询页面协议

本协议只约束一类页面：

- 页面主数据只查询一次
- 多个组件共享同一份主数据源
- 页面内部各模块从同一份数据衍生各自视图

它经常表现为：

- 顶部摘要卡片
- 主体表格或列表
- 辅助统计、筛选、详情摘要、局部面板

但本协议的核心不是某两个组件的组合，而是：

- **单次全量查询**
- **共享查询结果**
- **多个组件共同消费**

这是一个**单独页面类型协议**，不是 app 总协议，也不是组件内部协议。

当需求满足以下任一条件时，AI 必须阅读本文件：

- 新增一个“单次查询、多组件共享数据”的页面
- 重构现有列表页为标准模板页
- 讨论这类页面应该如何拆文件

---

## 1. 定位

这是 app 层的**页面装配协议**。

它解决的问题不是：

- `DataTable` 内部怎么实现
- `MetricCards` 内部怎么实现

它解决的是：

- 页面应该怎么组织文件
- 页面主文件应该保留什么
- 查询、卡片、表格、弹窗应该如何分层
- 如何把“全量一次请求 + 本地表格加工”写成稳定模板

组件接入细则应继续阅读：

- [MetricCards 使用协议](./metric-cards-usage.md)
- [DataTable 本地数据使用协议](./datatable-local-usage.md)

这类页面**不是普通服务端查询型页面**。

它的本质是：

- 后端一次返回页面所需全量数据
- 前端把该数据转换成 `rows`
- 页面上的多个组件共用这一份主数据
- 各模块只对这份数据做本地加工

---

## 2. 适用范围

适用于：

- `apps/admin`
- `apps/web`
- 未来任何 app 中，采用“摘要卡片 + 表格列表”的页面

典型页面：

- 账号列表
- 用户列表
- 订单列表
- 权限列表
- 配置中心列表
- 资源管理列表
- 一个请求驱动卡片、表格、侧边摘要的页面

不适用于：

- 纯详情页
- 纯表单页
- 纯 dashboard 页
- 强可视化分析页

也不适用于：

- 数据量很大，必须服务端分页的页面
- 搜索条件必须实时发服务端查询的页面
- 依赖复杂后端检索条件的页面

如果页面属于以上情况，不应套用本协议，而应单独建立“服务端查询型页面协议”。

---

## 3. 标准目录结构

这类页面默认采用以下结构：

```txt
<page>/
  index.tsx
  types.ts
  constants.ts
  <page>-data.tsx
  metrics/
    index.tsx
  table/
    index.ts
    logic.ts
    columns.tsx
    query-fields.ts
    status.ts
    sort.ts
    row-actions.ts
  dialogs/
    create-<entity>-dialog.tsx
    edit-<entity>-dialog.tsx
    delete-<entity>-dialog.tsx
```

如果页面带有任何弹窗型交互，优先直接建立 `dialogs/` 目录。

不要为了“暂时只有一个弹窗”把目录骨架做成特例。

如果页面要作为 AI 模板长期参考，`metrics` 也优先目录化，不要默认保留单文件：

- `metrics/index.tsx`

后续若指标区复杂化，再继续在 `metrics/` 下扩展即可。

模板页中，所有 `DataTable` 相关结构优先统一收敛到 `table/` 目录，不再额外保留页面根目录 `config/` 或单独的 `<page>-row-actions.ts`。

---

## 4. 每个文件的职责

### `index.tsx`

职责：

- 页面壳层
- 装配 `MetricCards`
- 装配 `DataTable`
- 连接 page hook / dialog config

禁止：

- 直接写查询逻辑
- 直接写 filter / sort / paginate
- 直接堆大量列定义
- 直接写弹窗表单内容
- 直接写插入/编辑提交逻辑

目标：

- 主文件控制在 30-80 行优先
- 让后续 AI 一眼能看懂页面结构

### `types.ts`

职责：

- 页面行类型
- 查询结构类型
- 页面内部状态联合类型

禁止：

- 常量
- JSX
- 请求逻辑

### `constants.ts`

职责：

- query key
- 默认 query
- page size options
- 页面固定常量

规则：

- 不要把 query key 散落在多个文件里
- 不要把默认 query 写死在页面 JSX 里

### `<page>-data.tsx`

职责：

- React Query
- API -> row 数据转换
- 页面数据加载
- query cache 读取
- 暴露供 `DataTable` 使用的本地 `fetchData`

不要负责：

- 列定义
- 行操作配置
- 弹窗 JSX
- 通过 `<page>-table` 文件转出口再间接依赖表格纯逻辑

### `metrics/index.tsx`

职责：

- `rows -> MetricCards items`

规则：

- 只做摘要映射
- 不直接发请求

模板规则：

- 模板页优先直接使用 `metrics/` 目录
- 不要因为“当前只有 MetricCards”就把模板页长期定格为单文件 metrics

### `table/logic.ts`

职责：

- filter
- sort
- paginate

规则：

- 必须尽量纯函数化
- 不要耦合 React hook
- 这一层默认处理 `DataTable` 的本地查询行为
- `data.tsx` 应直接依赖这一层，不要通过页面装配文件中转

### `table/index.ts`

职责：

- 收口 DataTable 需要的页面级配置

通常包含：

- `columns`
- `getRowId`
- `initialQuery`
- `query`
- `rowActions`
- `pageSizeOptions`

目标：

- 让页面主文件只拿 `table` 配置即可

模板目录规则：

- 模板页中，`table/index.ts` 负责收口表格入口配置
- 不要把成熟模板页长期维持在页面根目录分散的表格文件

### `table/columns.tsx`

职责：

- `DataTable columns`
- 列级展示表达

### `table/query-fields.ts`

职责：

- `query.builtInFields`
- `query.fields`

### `table/status.ts`

职责：

- 表格域内的状态映射
- 状态标签与状态枚举互转

### `table/sort.ts`

职责：

- 排序值映射
- 排序比较规则

### `table/row-actions.ts`

职责：

- `rowActions` 配置
- 所有单行操作定义

### `dialogs/*.tsx`

职责：

- 单个弹窗 UI
- 单个弹窗状态
- 单个弹窗提交逻辑
- 单个弹窗成功后的刷新逻辑

规则：

- 一个弹窗一个文件
- 不允许多个弹窗长期堆在主文件

---

## 5. 推荐开发顺序

AI 在新建这类页面时，默认按如下顺序：

1. `types.ts`
2. `constants.ts`
3. `<page>-data.tsx`
4. `metrics/index.tsx`
5. `table/logic.ts`
6. `table/columns.tsx`
7. `table/query-fields.ts`
8. `table/status.ts`
9. `table/sort.ts`
10. `table/row-actions.ts`
11. `table/index.ts`
12. `dialogs/*.tsx`
13. `index.tsx`

不要先把所有逻辑写进 `index.tsx` 再回头拆。

---

## 6. 多组件共享规则

这类页面最重要的规则是：

- 页面主数据只维护一份
- 不同组件从这份数据派生视图
- 组件之间不要各自再拉一遍同类数据

推荐模式：

- `useQuery` 获取一次主数据
- `metrics` 负责把主数据转成摘要卡片
- `DataTable` 或列表区负责把主数据转成分页视图
- 侧边统计、二级摘要、局部计数，也都从同一份主数据派生

禁止：

- 卡片一份请求
- 表格一份请求
- 辅助面板再一份请求
- 同页多个模块分别拉同一个接口

---

## 7. 请求策略规则

默认规则：

- 顶部统计卡片和表格必须共用同一份主数据
- 页面主数据默认只请求一次
- 页面优先避免 N+1 请求
- 列表页所需聚合数据优先推动服务端一次返回
- `DataTable` 的 query 变化默认不重新请求服务端

推荐：

- 一个 query key 作为页面主数据源
- 一个 `loadRows` 负责获取全量数据
- 一个 `fetchData` 负责本地表格加工
- 新增/编辑/删除后统一 `invalidateQueries`

不推荐：

- 卡片单独拉一次
- 表格再拉一次同一个接口
- 每行再拉一次关联接口
- 在本协议页面里实现服务端分页

推荐的数据流：

1. `useQuery` 拉取一次全量数据
2. 转成页面 `rows`
3. `MetricCards` 直接消费 `rows`
4. `DataTable.fetchData` 从 cache 拿 `rows`
5. 在前端执行 `filter / sort / paginate`

示意：

```tsx
const rowsQuery = useQuery({
  queryKey,
  queryFn: loadRows,
})

const metricCards = buildMetricCards(rowsQuery.data ?? [])

const fetchData = async ({ page, pageSize, query, sort }) => {
  const rows = await queryClient.ensureQueryData({ queryKey, queryFn: loadRows })
  const filtered = filterRows(rows, query)
  const sorted = sortRows(filtered, sort)
  return paginateRows(sorted, page, pageSize)
}
```

---

## 8. 何时不该使用本协议

出现以下任一信号时，不应继续使用本协议：

- 列表数据量已经明显过大
- 前端全量加载成本太高
- 条件查询必须依赖服务端检索
- 服务端已经提供了标准分页查询接口
- query 变化后必须重新命中后端才能得到正确结果

此时应切换到另一种页面类型：

- 服务端查询型 DataTable 页面

不要强行把这类页面继续塞进“本地数据页协议”。

---

## 9. 弹窗规则

只要有以下操作，就应优先拆成独立弹窗文件：

- 新增
- 编辑
- 删除确认
- 分配角色
- 查看详情

弹窗文件应负责：

- UI
- 状态
- 提交
- 刷新

不要把这些逻辑散落在主页面。

---

## 10. 禁止事项

- 禁止把整个页面长期堆在 `index.tsx`
- 禁止把卡片统计逻辑写死在 JSX 里
- 禁止把 columns / query.fields / rowActions 全堆在主文件
- 禁止把 filter / sort / paginate 与 React 状态强耦合
- 禁止把多个 dialog 长期塞在一个文件里
- 禁止把这类页面写成一次性实现，却希望以后复制复用
- 禁止误把本协议页面写成服务端查询型表格页
- 禁止让 `MetricCards` 与 `DataTable` 各自维护一份独立数据源

---

## 11. AI 默认判断规则

当 AI 接到一个新页面任务时，应先判断：

1. 这个页面是不是“单次查询 + 多组件共享数据”的类型？
2. 数据是否适合一次性全量加载？
3. 如果适合，是否应直接按本协议落目录？
4. 这个页面以后是否会成为别的页面模板？

只要答案偏向“是”，默认直接按本协议写。

---

## 12. 成熟模板标准

一个成熟的共享查询页面，至少应满足：

- 主文件只做拼装
- `types / constants / data / metrics / config / table logic / row actions / dialogs` 分层清晰
- 请求与缓存统一
- 全量数据只维护一份主数据源
- 表格 query 变化不触发服务端重复请求
- 弹窗独立
- 新页面可以直接复制这个目录骨架

---

## 13. 完成标准

这类页面改动只有同时满足以下条件才算完成：

1. 页面主文件没有堆积大量业务细节
2. `MetricCards` 和 `DataTable` 按协议装配
3. 查询、排序、筛选、分页职责清楚
4. 弹窗没有继续留在主文件
5. 主数据只请求一次且被卡片和表格共享
6. 页面结构能作为后续 AI 的合法模板
7. 对应 app 构建通过
