# Post 8-17：竞品分析读取 API

## 状态

done

## 本轮定位

旧产品迁移主体计划已完成 `12/12`。本轮是主体计划后的第 17 个补强项：把旧产品“竞品监控 / 竞品分析 / 竞品对比”的读取能力迁到新产品后端聚合 API。

## 产品内核变化

- 竞品能力从“散列表 CRUD”推进到“可直接消费的结构化分析结果”。
- 后端可以按当前商品聚合竞品价格、评分、平台分布和近期变化。
- 返回 `agentContext`，后续可直接注入 Agent 问答、竞品分析 skill、定价策略 skill。

## 后端变化

- 新增设计文档：`temp/DEVELOPMENT_DOCS/backend/product_competitor_analysis_post_8_17.md`。
- Service 层新增 `ProductCompetitorApi::competitor_analysis`。
- Web-server 新增 `GET /api/app/products/{productId}/competitors/analysis`。
- API 文档同步 `API_DOCS/app.md`。
- 本轮不新增表，不新增 migration，不调用 LLM，不触发外部搜索。

## 前端变化

- `apps/web/src/api/commerce.ts` 新增 `getProductCompetitorAnalysisApi`。
- 同步新增竞品分析响应类型。
- 本轮不新增页面入口，只同步真实后端 API 契约。

## 用户可感知变化

- 后续前端接入后，用户在商品详情的竞品区可以看到竞品分析摘要，而不只是竞品列表。
- 竞品价格是否高于/低于均价、有哪些近期价格变化、不同平台竞品分布，可以由后端直接给出。
- Agent 问答后续可读取 `agentContext`，回答竞品和定价问题时有结构化依据。

## 后续缺口

- 还未实现自动搜索竞品。
- 还未实现立即监控竞品并自动写入变化记录。
- 还未把分析结果自动沉淀为 workspace artifact。
