# Post 8-127：上架/标题/详情文案搜索链路

## 状态

done

## 本次目标

把旧产品上架文案、标题生成、详情文案中依赖平台规则、标题趋势、搜索量、买家评价和购买顾虑的增强能力，迁成新产品框架中的可见 skill 链路。用户明确提出最新、实时、联网、公开来源、标题规则、搜索量、买家痛点或真实评价诉求时，先运行对应 native skill，再追加统一搜索 skill。

## 产品内核变化

- 上架 / 标题 / 详情文案不再把外部资料隐藏在单个 native 输出里，而是拆成“内容 native skill + 外部搜索 skill”两步。
- 普通上架文案、标题生成和详情文案仍只走 native skill，避免没有外部资料诉求时误触发搜索。
- 搜索继续遵守统一 search adapter 降级契约：provider 未配置、失败或无可信来源时，不伪造平台规则、热搜词、评价或购买顾虑。
- 商品上下文仍是显式选择才进入链路；未选择商品时，不从历史、预设商品、测试商品或默认商品推断商品上下文。

## 后端变化

- 修改 `backend/crates/agent/src/agent/worker/runner.rs`。
- 修改 `backend/crates/agent/src/roles/skill_catalog.rs`。
- 新增上架 / 标题 / 详情文案编排：
  - `ops_listing_copy -> search_platform_policy(policy_type=content)`
  - `ops_listing_copy -> search_market_info(topic=general)`
  - `ops_listing_copy -> search_product_reviews`
  - `web_title_generator -> search_platform_policy(policy_type=content)`
  - `web_title_generator -> search_market_info(topic=general)`
  - `web_product_description -> search_platform_policy(policy_type=content)`
  - `web_product_description -> search_market_info(topic=general)`
  - `web_product_description -> search_product_reviews`
- 运营岗位 role catalog 新增 `search_product_reviews` 权限，确保上架文案 + 买家评价搜索链路不会被运行时拒绝。
- 复用已有 SSE skill 事件、搜索 adapter、搜索证据 artifact 和内容 artifact 机制，没有新增 HTTP API 或数据库表。

## 前端 / 用户可感知变化

- 用户说“帮冷萃咖啡写上架文案，查抖音最新标题规则和违禁词，最多4条来源”时，过程区会看到 `ops_listing_copy` 和 `search_platform_policy` 两个步骤。
- 用户说“给冷萃咖啡生成商品标题，联网查最新搜索量、热搜词和高点击趋势”时，过程区会看到 `web_title_generator` 和 `search_market_info` 两个步骤。
- 用户说“写冷萃咖啡详情文案，联网查买家痛点、真实评价和购买顾虑”时，过程区会看到 `web_product_description` 和 `search_product_reviews` 两个步骤。
- 用户只说“写上架文案”“生成标题”“写详情文案”时，仍只看到 native 内容生成步骤。
- 如果用户没有手动选择商品，搜索链路不会绑定历史或演示商品；如果用户手动选择商品，搜索 input 会携带 `product_ref/product_name/product_category`。

## 文档同步

- `API_DOCS/sse.md`：补充运营上架文案、Web 标题 / 详情文案 native + 搜索双 skill 链路。
- `API_DOCS/app.md`：补充上架 / 标题 / 详情内容 artifact 与搜索证据 artifact 的并行沉淀关系。
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`：补充 Post 8-127 能力矩阵。
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`：记录本次开发。
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`：记录本次验收。

## 验收记录

- [x] 上架 / 标题 / 详情文案搜索链路按新产品 Agent worker 框架实现。
- [x] 普通上架文案 / 标题 / 详情文案不因没有外部资料诉求而触发搜索。
- [x] 平台规则查询使用已有 `search_platform_policy(policy_type=content)` skill。
- [x] 标题趋势 / 搜索量 / 详情页趋势使用已有 `search_market_info` skill。
- [x] 买家痛点 / 真实评价使用已有 `search_product_reviews` skill。
- [x] 运营岗位 role catalog 已允许 `search_product_reviews`。
- [x] 未显式选择商品时，不注入默认商品上下文。
- [x] 定向 agent 测试覆盖上架文案、Web 标题/详情、显式选品上下文、native-only 回归和 role catalog 权限。

## 后续缺口

- 真实搜索 provider 的配置、部署和来源质量优化仍属于外部搜索能力的后续事项。
- 前端可继续把 native + search 双 skill 链路展示为可折叠过程卡片和来源面板。
