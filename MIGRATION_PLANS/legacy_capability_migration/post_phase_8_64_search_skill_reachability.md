# Post 8-64：搜索类 skill 自然触达补强

状态：done

## 本次目标

- 延续外部搜索 adapter 契约，让用户不需要手写 `search_*` 工具调用。
- 用户说“搜趋势 / 查竞品公开资料 / 看平台最新规则 / 找行业基准 / 搜用户评价”时，后端能自动进入对应搜索 skill。
- 坚持真实边界：没有搜索 provider 时返回 deferred/degraded，不伪造实时来源。

## 本次修改

- 后端 agent：worker autosuggest 新增搜索类 skill 触达逻辑。
- 后端 agent：覆盖 `search_trends`、`search_competitor`、`search_market_info`、`search_product_reviews`、`search_platform_policy`、`search_industry_benchmarks`。
- 后端 agent：保留本地 native skill 优先级，普通竞品分析、趋势预测、关键词研究不会被搜索 skill 抢走。
- 后端 agent：支持 `_needs_fresh_data/bypass_cache` 语义，用户说“实时/最新/联网/不要缓存”时进入搜索查询参数。
- 后端角色目录：补齐对应角色可用搜索 skill 权限，避免自动触发后被权限拦截。
- API/SSE 和迁移文档同步。

## 用户可感知变化

- 用户可以直接说“联网搜一下咖啡饮品最近趋势”，系统会进入 `search_trends`。
- 用户可以说“查抖音最新平台规则”，系统会进入 `search_platform_policy`。
- 用户可以说“搜冷萃咖啡用户差评和口碑”，系统会进入 `search_product_reviews`。
- 如果外部搜索没有配置，系统会明确展示未接入/降级状态，不会编造来源。

## 当前验证

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/search_skill_reachability_post_8_64.md`
- [x] Agent 单测：趋势、竞品、平台政策、评价、行业基准自然触达
- [x] Agent 单测：不覆盖普通竞品分析、趋势预测、关键词研究

## 后续缺口

- 真实搜索 provider 配置和线上稳定性仍需环境侧完成。
- 前端来源面板后续应更清楚展示 live/degraded/deferred 三种状态。
