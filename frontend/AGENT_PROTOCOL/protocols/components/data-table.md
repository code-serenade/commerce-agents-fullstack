# DataTable 协议

本协议用于约束 `@workspace/app-kit` 中 `DataTable` 的设计边界、真实 API 和 AI 默认接入方式。

当任务涉及以下任一内容时，必须阅读本文件，并以源码为准：

- 新增或修改 `DataTable` 本身
- 在 app 中接入 `DataTable`
- 调整列表页 query / row action / bulk action / insert action
- 调整 admin 列表页表格 header 布局
- 讨论 `DataTable` 是否应该支持某种能力

权威来源顺序：

1. `src/components/data-table/data-table.types.ts`
2. `src/components/data-table/data-table.tsx`
3. `src/components/data-table/data-table-header.tsx`
4. `src/components/data-table/data-table-dialogs.tsx`
5. 本文件

`showcases/guide/src/pages/DataTableGuidePage.tsx` 可能落后于源码。不要把 guide 页当成能力来源；如果 guide 与源码冲突，必须优先相信源码。

## 1. 宪章

`DataTable` 是 admin 场景的核心框架组件，不是自由拼装的通用玩具组件。

它的目标是：

- 提供稳定、受控、可复用的列表页框架
- 统一 admin 场景的查询、批量操作、行操作、分页、排序和时间筛选行为
- 允许合理配置，但禁止无边界外放内部结构

AI 必须默认把 `DataTable` 视为“框架和边界”，而不是“可随意定制的底层表格”。

## 2. 最小接入

最小闭环只需要：

```tsx
<DataTable<Row, Query>
  columns={columns}
  fetchData={fetchData}
  getRowId={(row) => row.id}
/>
```

必填项：

- `columns`：列定义
- `fetchData`：分页、查询、排序的数据入口
- `getRowId`：稳定行 ID

`fetchData` 收到：

- `page`
- `pageSize`
- `signal`
- `query`
- `sort`

必须返回：

- `items`
- `total`

默认规则：

- `page` 从 `1` 开始
- `initialPageSize` 默认 `15`
- `pageSizeOptions` 默认 `[10, 15, 30, 50]`
- `initialSort` 默认 `null`
- `fetchData` 必须处理 `AbortSignal`
- 查询、排序、页码变化都应通过 `fetchData` 收口

## 3. 列定义

`DataTableColumn<T>` 当前支持：

- `key`
- `header`
- `renderCell(row, rowIndex)`
- `width`
- `sticky`
- `sortable`

默认规则：

- `key` 必须稳定，并和排序字段保持一致
- `sortable: true` 后，点击表头会在 `asc -> desc -> none` 间切换
- `width` 可以是 number 或 CSS 字符串；固定列宽推荐用 number
- `sticky` 支持 `"left"` / `"right"`
- 也可以用 `stickyColumns={{ left, right }}` 按列位置批量冻结

AI 不应在业务页复制 `DataTable` 内部 sticky 计算逻辑。

## 4. Header 结构

`DataTable` header 固定分成两个大区域：

- 左侧：query 区
- 右侧：action 区

源码当前行为：

- query 区是横向滚动区域
- `search` 类型的 built-in query 会被提升为 leading search
- `query.tools` 默认显示 reset / refresh
- 其他 `query.builtInFields` 与 `query.fields` 会继续排列在 query 区
- action 区承载 `insert` / `bulkUpdate` / `bulkDelete` / `toolbarActions`

设计约束：

- 主搜索优先放 `query.builtInFields`
- 时间范围、状态、区域、负责人、类型、标签等筛选放 `query.fields`
- 不允许业务页自己拼一套独立 table header 来绕过框架
- 不允许新增万能 header render prop 来外放内部骨架

## 5. Query 规则

`query` 是查询区的唯一配置入口：

```tsx
query={{
  builtInFields,
  fields,
  audit: true,
  tools: true,
}}
```

### `query.builtInFields`

用于框架内建主查询。

源码支持类型：

- `search`
- `select`

默认规则：

- 主搜索使用 `type: "search"`
- built-in query 不允许承载 `date-range` / `scoped-date-range`
- 时间类查询必须走 `query.audit`
- built-in query 应保持视觉稳定，不承载零散业务筛选和时间特例

注意：当前源码里的 `search` 字段只支持 `key`、`label`、`description`、`disabled`、`placeholder`。不要使用 guide 旧示例里的 `fieldKey` / `fieldOptions`，除非先在源码类型和实现中补齐该能力。

### `query.fields`

用于业务补充筛选。

源码支持类型：

- `text`
- `search`
- `select`

默认规则：

- 附加筛选优先使用 `select`
- `text` 适合低优先级业务文本筛选
- 不要把主搜索类条件放到 `query.fields`
- `select` 默认允许清空

### Query 状态

`initialQuery` 会作为 query 初始值浅拷贝进内部 state。

修改任一 query 字段会：

- 将 `page` 重置为 `1`
- 触发 `fetchData`

`reset` 会把 query 恢复到 `initialQuery`。

## 6. 时间能力

`DataTable` 当前支持标准审计时间能力，但它不属于 `query.builtInFields` 或 `query.fields`。时间字段集合只能通过 `auditColumns` 配置，并且只允许以下字段：

- `createdAt`
- `updatedAt`

禁止业务通过自定义 query field 添加任意时间字段。不要为 `renewalAt`、`lastActiveAt`、`deletedAt` 等字段私自加时间 query。

`auditColumns` 是字段和列展示的单一事实源：

- 开启 `createdAt` 会显示创建时间列
- 开启 `updatedAt` 会显示更新时间列
- 同时开启两者时，表格显示两列
- 关闭某个字段时，它的列和对应 query 选项都应一起消失

`query.audit` 只控制是否显示时间 query 组件：

- `query.audit !== true` 不显示时间 query
- `query.audit === true` 时，query 组件自动跟随 `auditColumns`
- 如果 `auditColumns` 只有一个字段，query 显示单个日期范围
- 如果 `auditColumns` 有两个字段，query 显示字段切换 + 日期范围
- 时间字段文案、select 选项、placeholder、列头都由组件内部固定规则决定，不允许外部配置

### 单字段时间查询

适合查询单一时间字段。

```tsx
<DataTable
  auditColumns={["createdAt"]}
  query={{ audit: true }}
/>
```

对应 `Query`：

```ts
interface Query {
  createdAt?: DateRangeValue
}
```

`fetchData` 负责解释 `from` / `to` 的业务含义。组件不替业务拼服务端参数。

### 双字段时间查询

适合在 `createdAt` 和 `updatedAt` 之间切换。

```tsx
<DataTable
  auditColumns={["createdAt", "updatedAt"]}
  query={{ audit: true }}
/>
```

对应 `Query`：

```ts
interface Query {
  auditField: "createdAt" | "updatedAt"
  auditRange?: DateRangeValue
}
```

特殊行为：

- 双字段模式下，内部固定使用 `auditField` 和 `auditRange`
- 只切换内部时间字段选择且当前 range 为空时，源码会避免重复触发一次无意义 fetch
- 一旦 range 有值，所选时间字段和 range 都是有效查询条件

标准审计时间字段固定为：

- `createdAt`
- `updatedAt`

HTTP JSON 和业务 DTO 应继续使用 camelCase，不要引入 `created_at` / `updated_at`。

## 7. 审计时间列

如果页面需要展示标准审计时间字段，优先通过 `auditColumns` 启用。不要在每个页面重复手写列。

`auditColumns` 用于决定标准审计列集合，也决定 `query.audit` 可选择的时间字段。

`auditColumns` 支持：

- `true`：展示 `createdAt` 和 `updatedAt`
- `["createdAt"]`
- `["updatedAt"]`
- `{ columns, emptyText, formatDateTime }`

默认规则：

- 读取 row 上的 `createdAt` / `updatedAt`
- 支持 `Date`、可被 `Date` 解析的 string / number
- 无值或非法值显示 `auditEmptyText`，默认 `"-"`
- 默认格式跟随当前语言
- 列头固定由组件内部决定，不允许页面覆盖
- 如需业务格式，使用 `auditColumns.formatDateTime`

示例：

```tsx
<DataTable
  auditColumns={{
    columns: ["createdAt", "updatedAt"],
    formatDateTime: (value) => formatDateTime(value),
  }}
/>
```

## 8. Action 规则

顶部 action 区只承载框架级主操作：

- `insert`
- `bulkUpdate`
- `bulkDelete`

顺序固定：

1. `insert`
2. `bulkUpdate`
3. `bulkDelete`

默认规则：

- 三类操作都支持 `false` 关闭
- 顶部主按钮默认是 icon-only + tooltip
- `bulkUpdate` / `bulkDelete` 需要选中行才可用
- 当开启 `bulkUpdate` 或 `bulkDelete` 且没有显式关闭 `selection` 时，选择列会自动开启

### `toolbarActions`

`toolbarActions` 只是保留插槽，不应用于替代内建主操作。

AI 禁止把大块业务操作直接塞进 `toolbarActions`，除非用户明确要求且不破坏整体框架。

## 9. Selection / Bulk 规则

`selection` 是一等能力，不应被当成零散附加逻辑。

`selection` 支持：

- `false`
- `{}`
- `columnWidth`
- `selectedRowKeys`
- `onSelectedRowKeysChange(keys, rows)`

默认规则：

- 显式 `selection={false}` 会关闭选择列，即使配置了 bulk action
- 未传 `selection` 且开启 `bulkDelete` 或 `bulkUpdate` 时，自动开启选择列
- 受控选择由 `selectedRowKeys` + `onSelectedRowKeysChange` 接管
- `selectedRows` 只来自当前页 rows，跨页选择时业务要自行确认语义

`bulkDelete.onDelete` 收到：

- `clearSelection`
- `selectedRowKeys`
- `selectedRows`

`bulkUpdate.onSubmit` 收到：

- `clearSelection`
- `selectedRowKeys`
- `selectedRows`
- `fieldKey`
- `value`

`bulkUpdate.fields` 当前支持：

- `text`
- `select`

## 10. Dialog 规则

`DataTable` 内建 dialog 用于统一核心交互：

- `insert`
- row `edit`
- row `delete`
- `bulkUpdate`
- `bulkDelete`

`insert` 支持：

- `label`
- `disabled`
- `title`
- `description`
- `cancelLabel`
- `confirmLabel`
- `renderContent({ close })`
- `onConfirm`

row `edit` 支持：

- `label`
- `title(row, rowIndex)`
- `description(row, rowIndex)`
- `cancelLabel`
- `confirmLabel`
- `renderContent({ row, rowIndex, close })`
- `onConfirm(row, rowIndex)`

row `delete` 支持：

- `label`
- `title(row, rowIndex)`
- `description(row, rowIndex)`
- `cancelLabel`
- `confirmLabel`
- `onConfirm(row, rowIndex)`

默认规则：

- 弹窗结构优先统一，不要每个页面各搞一套
- 表单内容可以通过 `renderContent` 注入
- 提交成功后组件会触发 refresh
- 文案必须可 i18n，优先使用 `localeText` 或业务翻译结果

## 11. 行级操作

`rowActions` 支持：

- `false`
- `header`
- `columnWidth`
- `sticky`
- `moreLabel`
- `edit`
- `delete`
- `moreItems`

默认规则：

- `edit` / `delete` 显式配置时显示
- `moreItems` 只有非空时才显示更多菜单
- action 列宽度默认按可见按钮数量自动计算
- `sticky` 支持 `"left"` / `"right"` / `false`
- 行级操作图标与批量/顶部操作图标应保持语义区分

`moreItems` 支持：

- `key`
- `label` 或 `(row, rowIndex) => label`
- `variant`
- `disabled` 或 `(row, rowIndex) => boolean`
- `onClick(row, rowIndex)`

## 12. 状态、空态和错误态

基础文案 props：

- `emptyText`
- `errorText`
- `loadingText`

自定义渲染 props：

- `renderEmpty`
- `renderError(error, retry)`
- `renderLoading`

`localeText` 可覆盖：

- empty / error / loading
- refresh / reset
- insert / actions / edit / delete / more
- delete dialog
- bulk delete / bulk update
- sort 文案
- audit columns 文案

默认规则：

- 简单文案优先用 `localeText`
- 需要完整布局时才用 `renderEmpty` / `renderError` / `renderLoading`
- `renderError` 必须暴露 retry 操作

## 13. 展示密度和尺寸

当前支持：

- `stripedRows`
- `compactColumns`
- `compactRows`
- `fillWidth`
- `height`
- `caption`

默认规则：

- `stripedRows` 默认 `true`
- `compactColumns` 默认 `false`
- `compactRows` 默认 `false`
- `fillWidth` 默认 `true`
- `height` 可以是 number 或 CSS 字符串
- 宽表应优先设置列宽和冻结列，而不是让内容自然撑爆布局

## 14. 响应式规则

AI 修改 `DataTable` 时，必须同时考虑：

- desktop
- mobile

默认要求：

- query 区优先保留横向空间
- action 区按按钮数量收缩
- 顶部主操作在小屏可竖排
- header 不应因为局部 flex 改动而意外断裂

如果某次改动会影响 header 行内结构，必须优先检查：

- leading search 是否仍然稳定
- reset / refresh 是否仍在 query 区
- action 区是否仍然垂直居中
- query 区横向滚动是否可用

## 15. AI 默认决策规则

当 AI 被要求“新增列表页”或“在 admin 中使用表格”时，默认应：

1. 优先使用 `DataTable`
2. 定义清晰的 `Row` 和 `Query`
3. 主查询放入 `query.builtInFields`
4. 业务补充筛选放入 `query.fields`
5. 标准时间字段只能通过 `auditColumns` 启用，字段只允许 `createdAt` / `updatedAt`
6. 时间 query 只能通过 `query.audit` 开关显示，并自动跟随 `auditColumns`
7. 行级操作优先使用 `rowActions`
8. 批量操作优先使用 `bulkUpdate` / `bulkDelete`
9. 新增操作优先使用 `insert`

AI 不应默认：

- 自己在 app 页面里重新手写一套 table header
- 复制 `DataTable` 内部实现到业务页
- 为单页面特例破坏 `DataTable` 统一行为
- 使用 guide 旧代码里但源码类型不存在的 props

## 16. 禁止事项

- 禁止把 `DataTable` 改成任意布局容器。
- 禁止为了满足少数页面，把 header 完全外放给业务自定义。
- 禁止新增“万能透传型 props”来逃避框架设计。
- 禁止把业务特例直接写死进组件核心。
- 禁止在 app 中绕过 `DataTable` 自己复制 bulk action / row action / query header 逻辑。
- 禁止把 guide 页旧示例当成当前能力文档。

## 17. 推荐接入方式

推荐：

- `query.builtInFields`：主搜索等非时间内建查询
- `query.fields`：状态、区域、分类、负责人等附加筛选
- `auditColumns`：标准 `createdAt` / `updatedAt` 字段集合，也是审计列展示的事实源
- `query.audit`：只控制是否显示标准时间查询组件
- `insert`：新增
- `bulkUpdate` / `bulkDelete`：批量操作
- `rowActions.edit` / `rowActions.delete`：单行标准操作
- `rowActions.moreItems`：复制、跳转、打开业务自定义弹窗等扩展动作

不推荐：

- 用 `toolbarActions` 搭一整套自己的顶部操作栏
- 把主搜索拆成多个分离组件塞进 `query.fields`
- 在多个页面重复手写审计时间列
- 通过大量页面级 patch 来修 `DataTable` 的视觉差异

## 18. 完成标准

涉及 `DataTable` 的任务，只有同时满足以下条件才算完成：

1. 不破坏 query / action 两区结构
2. 不引入新的无边界透传
3. 不使用源码不存在的 props
4. 时间查询和审计时间列按源码能力接入
5. 至少在一个真实 app 页面完成验证
6. `pnpm -C packages/app-kit typecheck` 通过

如果只是视觉调整，也必须检查：

- desktop header 是否稳定
- mobile header 是否仍可用
- 关闭部分 action 后区域是否自动收缩
- 宽表、冻结列、紧凑行列是否仍可用
