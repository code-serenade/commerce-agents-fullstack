# Post 8-17：商品竞品分析读取 API 后端设计

## 需求摘要

旧产品具备竞品档案、竞品变化监控、竞品分析与策略建议能力。新产品当前已经有商品竞品档案 CRUD 和变化记录 CRUD，但缺少一个稳定的后端聚合分析入口，前端和 Agent 只能读取散列表，无法直接消费“竞品价格位置、平台分布、近期变化、风险信号”等结构化结果。

本阶段实现“竞品分析读取 API”。它不调用 LLM，不触发外部搜索，不新增表，只基于新产品已有数据表做真实聚合分析。

## 数据表设计

不新增表，不新增 migration。

复用已有表：

```text
products
product_competitors
competitor_changes
```

核心字段：

```text
products.price / currency / platform / category / brand
product_competitors.price / rating / monthly_sales / platform / notes / search_results / metadata
competitor_changes.change_type / old_value / new_value / summary / source_type / detected_at
```

## 权限设计

- 接口必须登录，使用 App JWT。
- `user_id` 来自 JWT，按 UUID 字符串处理。
- 只能分析当前用户自己的商品。
- 已归档商品不可分析。
- 竞品和变化记录必须按当前商品、当前用户过滤。

## HTTP 接口

```text
GET /api/app/products/{productId}/competitors/analysis
```

查询参数：

```text
status      可选，active | archived，不传默认 active
platform    可选，只分析某平台竞品
changeLimit 可选，1-500，不传默认 100
```

成功响应包含：

- 商品基准信息
- 竞品数量、价格/评分/销量字段覆盖情况
- 竞品价格带、均价、自身价格相对位置
- 平台分布
- Top 竞品快照
- 近期变化
- signals / warnings
- 可给 Agent 注入的 `agentContext`

## 错误码

复用已有错误码：

- `APP_USER_NOT_FOUND`
- `APP_USER_DISABLED`
- `PRODUCT_NOT_FOUND`
- `PRODUCT_FORBIDDEN`
- `PRODUCT_COMPETITOR_REQUEST_INVALID`

## 与 Agent 上下文关系

本接口返回的 `agentContext` 是稳定结构化上下文，可在后续 Agent 问答、竞品分析 skill、运营策略 skill、定价策略 skill 中直接引用。

本阶段不自动写 workspace artifact，不自动触发搜索；后续可基于该 API 再扩展“分析结果沉淀为交付物”和“外部搜索补全竞品数据”。

## 开发顺序

1. Service DTO 增加竞品分析请求与响应结构。
2. `ProductCompetitorApi` 增加 `competitor_analysis`。
3. Web-server 增加 query DTO、handler 和 route。
4. 更新 `API_DOCS/app.md`。
5. 前端 `apps/web/src/api/commerce.ts` 同步类型和 API 函数。
6. 更新迁移记录与验收记录。
