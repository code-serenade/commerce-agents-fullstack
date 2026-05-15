# Post 8-124：数据岗位外部趋势 / 基准搜索链路

## 状态

done

## 本次目标

把旧产品数据岗位里隐含的外部趋势、行业基准和市场资料检索能力，迁成新产品框架中的可见 skill 链路。用户明确提出最新、实时、联网、公开资料、行业基准或市场趋势诉求时，数据 native skill 先完成本地分析，再追加统一 `search_*` skill。

## 产品内核变化

- 数据岗位不再把外部资料需求隐藏进单个回答文本，而是拆成“数据分析 native skill + 外部搜索 skill”两步。
- 普通数据分析仍只走 native skill，避免没有实时资料诉求时误触发搜索。
- 外部搜索结果继续遵守统一 adapter 降级契约：provider 未配置、失败或无可信来源时，不伪造行业基准、趋势报告或市场资料。
- 商品上下文仍是显式选择才进入链路；未选择商品时，不从历史、预设商品、测试商品或默认商品推断商品上下文。

## 后端变化

- 修改 `backend/crates/agent/src/agent/worker/runner.rs`。
- 新增数据岗位外部搜索编排：
  - `data_channel_roi` / `data_attribution_analysis` -> `search_industry_benchmarks(metric=roi)`
  - `data_refund_decomposition` -> `search_industry_benchmarks(metric=refund)`
  - `data_trend_forecast` / `data_demand_forecast` / `data_multi_period_trend` / `data_seasonal_decompose` -> `search_trends`
  - `data_anomaly_diagnosis` -> `search_market_info(topic=news)`
  - `data_comprehensive_diagnosis` / `data_dashboard` -> `search_market_info(topic=general)`
- 复用已有 SSE skill 事件、搜索 adapter、搜索证据 artifact 和内容 artifact 机制，没有新增 HTTP API 或数据库表。
- `infer_data_search_days` 只读取明确时间窗口，避免把“最多4条来源”误解析成 `days=4`。

## 前端 / 用户可感知变化

- 用户说“做咖啡需求预测，并联网查最新市场趋势，最多4条来源”时，过程区会出现 `data_demand_forecast` 和 `search_trends` 两个步骤。
- 用户说“复盘渠道 ROI，并查行业 ROI 基准”时，过程区会出现 `data_channel_roi` 和 `search_industry_benchmarks(metric=roi)`。
- 用户说“做综合诊断，并查最新市场报告和行业动态”时，过程区会出现 `data_comprehensive_diagnosis` 和 `search_market_info(topic=general)`。
- 用户只做普通趋势预测、退款拆解、看板或综合诊断时，仍只看到 native 数据分析步骤。

## 文档同步

- `API_DOCS/sse.md`：补充数据岗位 native + search 双 skill 链路。
- `API_DOCS/app.md`：补充数据内容 artifact 与搜索证据 artifact 的并行沉淀关系。
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`：补充 Post 8-124 数据岗位能力矩阵。
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`：记录本次开发。
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`：记录本次验收。

## 验收记录

- [x] 数据岗位外部趋势 / 基准搜索链路按新产品 Agent worker 框架实现。
- [x] 普通数据 native skill 不因没有外部资料诉求而触发搜索。
- [x] 行业基准、趋势搜索和市场资料搜索使用已有 `search_*` skill。
- [x] 未显式选择商品时，不注入默认商品上下文。
- [x] 定向 agent 测试覆盖趋势、基准、市场资料和显式选品上下文。

## 后续缺口

- 真实外部搜索 provider 的配置、部署和质量优化仍属于外部搜索能力的后续事项。
- 前端可继续把 native + search 双 skill 链路展示为可折叠过程卡片和来源面板。
