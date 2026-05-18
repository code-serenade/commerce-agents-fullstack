# Post 8-126：创意直播/软文趋势搜索链路

## 状态

done

## 本次目标

把旧产品创意能力里隐含的直播间玩法、转化节奏、爆款文章、公众号热门风格等外部趋势检索能力，迁成新产品框架中的可见 skill 链路。用户明确提出最新、实时、联网、公开来源或热门趋势诉求时，创意 native skill 先生成直播话术或产品软文，再追加统一 `search_trends`。

## 产品内核变化

- 创意岗位不再把直播/软文趋势需求隐藏进单个 native 输出，而是拆成“创意内容 native skill + 外部趋势搜索 skill”两步。
- 普通直播话术和普通产品软文仍只走 native skill，避免没有外部资料诉求时误触发搜索。
- 搜索继续遵守统一 search adapter 降级契约：provider 未配置、失败或无可信来源时，不伪造热门直播间、爆款文章或公开来源。
- 商品上下文仍是显式选择才进入链路；未选择商品时，不从历史、预设商品、测试商品或默认商品推断商品上下文。

## 后端变化

- 修改 `backend/crates/agent/src/agent/worker/runner.rs`。
- 新增创意直播/软文编排：
  - `creative_live_script -> search_trends`
  - `creative_product_article -> search_trends`
- 调整创意 native 意图优先级：明确“产品软文 / 文章”时不被“爆款 / 热门”误路由成 `creative_trend_catch`。
- 补强创意搜索主题抽取：支持 `帮X写...` 和 `写一篇X产品软文...` 两类常见表达。
- 修正直播时长推断：`最多4条来源` 只作为搜索来源数量，不再误当成 4 小时直播。
- 复用已有 SSE skill 事件、搜索 adapter、搜索证据 artifact 和内容 artifact 机制，没有新增 HTTP API 或数据库表。

## 前端 / 用户可感知变化

- 用户说“写一套抖音直播带货话术，联网查最新直播间玩法和转化节奏，最多4条来源”时，过程区会看到 `creative_live_script` 和 `search_trends` 两个步骤。
- 用户说“写一篇冷萃咖啡产品软文，查最新公众号爆款文章和热门内容风格”时，过程区会看到 `creative_product_article` 和 `search_trends` 两个步骤。
- 用户只说“写直播带货开场话术”或“写一篇产品软文”时，仍只看到创意 native 内容生成步骤。
- 如果用户没有手动选择商品，搜索链路不会绑定历史或演示商品；如果用户手动选择商品，搜索 input 会携带 `product_ref/product_name/product_category`。

## 文档同步

- `API_DOCS/sse.md`：补充创意直播/软文 native + 趋势搜索双 skill 链路。
- `API_DOCS/app.md`：补充创意内容 artifact 与趋势搜索证据 artifact 的并行沉淀关系。
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`：补充 Post 8-126 创意能力矩阵。
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`：记录本次开发。
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`：记录本次验收。

## 验收记录

- [x] 创意直播/软文趋势搜索链路按新产品 Agent worker 框架实现。
- [x] 普通直播话术 / 产品软文不因没有外部趋势诉求而触发搜索。
- [x] 趋势查询使用已有 `search_trends` skill。
- [x] 未显式选择商品时，不注入默认商品上下文。
- [x] “最多4条来源”等搜索来源数量不会污染直播时长。
- [x] 定向 agent 测试覆盖直播/软文链路、显式选品上下文、native-only 回归和 creative/search 回归。

## 后续缺口

- 真实内容趋势 provider 的配置、部署和质量优化仍属于外部搜索能力的后续事项。
- 前端可继续把 native + search 双 skill 链路展示为可折叠过程卡片和来源面板。
