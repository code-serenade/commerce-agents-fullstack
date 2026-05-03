# MetricCards 使用协议

本文件只给 AI 看。

它约束的是：

- app 页面如何接入 `MetricCards`
- `MetricCards` 在页面中的职责边界
- 什么时候应该自定义 `card`

它不负责：

- `MetricCards` 组件内部实现细节
- 页面整体数据模式

若页面协议中提到需要使用 `MetricCards`，AI 应继续阅读本文件。

---

## 1. 默认定位

`MetricCards` 是页面摘要区组件。

默认职责：

- 展示页面主数据的摘要指标
- 消费页面已有数据
- 不主动拥有自己的独立请求策略

默认不负责：

- 自己单独请求一份页面主数据
- 承载复杂业务交互
- 替代详情面板或图表面板

---

## 2. 默认数据来源

`MetricCards` 默认应消费页面主数据派生出的 `items`。

推荐：

1. 页面先拿到主数据 `rows`
2. 单独在 `metrics` 文件中把 `rows` 转成 `MetricCards items`
3. 页面主文件只负责渲染

推荐模式：

```tsx
const rows = query.data ?? []
const metricCards = buildXxxMetricCards(rows)

return <MetricCards items={metricCards} />
```

禁止：

- 页面为了卡片区再额外请求一次同一份主数据
- 直接在页面 JSX 里堆复杂统计表达式

---

## 3. 文件落点规则

如果页面使用 `MetricCards`，模板页默认应有独立的 `metrics/` 目录：

- `metrics/index.tsx`

它只负责：

- `rows -> MetricCards items`

普通小页面可暂时保留单文件 `<page>-metrics.tsx`，但模板页优先目录化。

不要把卡片映射逻辑长期放在：

- `index.tsx`
- `data.tsx`
- `config.tsx`

---

## 4. 默认样式使用规则

AI 默认先使用 `MetricCards` 的默认样式。

只有在以下情况才考虑自定义 `card`：

- 页面已有非常明确的卡片视觉规范
- 多个页面需要同一种特殊摘要卡样式
- 默认样式无法表达页面核心信息层级

若只是单页轻微差异，不要优先重写整个卡片渲染。

---

## 5. item 组织规则

`MetricCardsItem` 默认应只承载页面摘要信息，不承载复杂业务状态机。

优先使用：

- `label`
- `value`
- `tail`
- `icon`
- `variant`

如果页面摘要信息超过这个层级，应优先重新判断是否还是 `MetricCards` 的适用场景。

---

## 6. 与页面数据模式的关系

`MetricCards` 本身不决定页面请求策略。

请求策略由页面协议决定。

例如：

- `shared-query-page`：卡片消费一次查询后的共享数据
- `server-query-page`：卡片可能消费 summary 接口

AI 不应反过来用 `MetricCards` 去决定页面数据流。

---

## 7. 禁止事项

- 禁止卡片区单独重复拉主数据
- 禁止在页面主文件堆复杂卡片统计逻辑
- 禁止因为单页特例就复制 `MetricCards` 内部结构
- 禁止把 `MetricCards` 当作任意布局容器

---

## 8. 完成标准

`MetricCards` 接入只有在以下条件同时满足时才算完成：

1. 数据来源清晰
2. 卡片映射逻辑不在主文件堆积
3. 没有重复请求同一份主数据
4. 默认优先复用现有组件能力
