# DataTable 本地数据使用协议

本文件只给 AI 看。

它约束的是：

- app 页面如何在“本地数据模式”下接入 `DataTable`
- `fetchData` 在本地数据模式下应该怎么写
- 列定义、查询字段、行操作应如何拆分

它不负责：

- `DataTable` 组件内部实现细节
- 服务端查询型页面的 `DataTable` 用法

若页面协议明确说明当前页面属于“本地数据模式”，AI 应继续阅读本文件。

---

## 1. 默认定位

这里的 `DataTable` 是：

- 页面主数据已一次性拿到
- 表格只对本地数据做加工

所以这里的 `fetchData` 本质上不是“请求服务端”，而是：

- 读取本地 rows
- filter
- sort
- paginate

---

## 2. fetchData 规则

在本地数据模式下，`fetchData` 默认应：

1. 从统一 query cache 中读取 rows
2. 对 rows 做本地过滤
3. 对过滤结果做本地排序
4. 对排序结果做本地分页
5. 返回 `items + total`

推荐：

```tsx
const fetchData = async ({ page, pageSize, query, sort }) => {
  const rows = await queryClient.ensureQueryData({ queryKey, queryFn })
  const filtered = filterRows(rows, query)
  const sorted = sortRows(filtered, sort)
  return paginateRows(sorted, page, pageSize)
}
```

禁止：

- 在本地数据页面里把 `fetchData` 写成新的服务端查询
- 让 `DataTable` 再单独请求一份和页面主数据相同的接口

---

## 3. 文件拆分规则

`DataTable` 相关逻辑默认拆成：

- `table/logic.ts`
  - filter
  - sort
  - paginate
- `table/index.ts`
  - 收口页面级 `DataTable` 配置
- `table/columns.tsx`
  - columns
- `table/query-fields.ts`
  - query.builtInFields
  - query.fields
- `table/status.ts`
  - 状态映射
- `table/sort.ts`
  - 排序字段映射
- `table/row-actions.ts`
  - rowActions

不要把这些全部写在主页面。

命名规则：

- 如果只是收口配置，不要滥用 `use-*` 命名
- 只有文件内部真的持有 hook 行为时，才使用 `use-<page>-table.ts`
- 当页面已经拆出表格入口与纯逻辑两个文件时，模板页优先收敛为 `table/` 目录

模板升级规则：

- 模板页不要再单独保留页面根目录 `config/`
- 表格相关配置统一收进 `table/`

---

## 4. 查询字段规则

默认规则：

- 主搜索放 `query.builtInFields`
- 附加筛选放 `query.fields`
- 列表页的 query 变化只影响本地视图，不默认触发新请求

如果 query 变化必须请求服务端，那说明页面不属于本协议。

---

## 5. 行操作规则

`rowActions` 默认收口到独立文件：

- `table/row-actions.ts`

目的：

- 主文件保持干净
- 页面后续扩展 edit/delete/moreItems 时有稳定落点

---

## 6. 本地数据模式与服务端查询模式的区别

本地数据模式：

- 一次请求拿全量 rows
- 表格 query 本地处理
- `MetricCards` / `DataTable` 共用数据源

服务端查询模式：

- query 变化要重新请求服务端
- 表格可能拥有独立接口参数
- 页面可能还需要单独 summary 接口

AI 不能混用两种模式。

---

## 7. 禁止事项

- 禁止在本地数据页里让 `DataTable` 重新请求同一份全量接口
- 禁止把 filter / sort / paginate 直接堆在 `index.tsx`
- 禁止把 columns / query.fields / rowActions 长期塞在页面主文件
- 禁止把服务端查询型逻辑混进本地数据型表格页
- 禁止让 `data.tsx` 通过 `table/index.ts` 间接依赖表格纯逻辑

---

## 8. 完成标准

本地数据模式下的 `DataTable` 接入只有在以下条件同时满足时才算完成：

1. `fetchData` 只做本地数据加工
2. 表格不再单独重复请求页面主数据
3. 列、查询字段、行操作拆分清晰
4. 页面主文件没有堆积表格细节
