# Post 8-118 Web/SEO 增长搜索链路

## 需求摘要

旧产品 Web 增长技能在 `web_seo_optimize` 和 `web_page_conversion` 内部直接调用 `_web_search`，把 SEO 算法动态、关键词竞争和行业转化率基准混入最终结果。新产品必须用自身 Agent runtime、skill autosuggest、SSE、search adapter 和 artifact 体系落地，不能把外部搜索隐藏在 native Web skill 内部。

## 后端设计

- 不新增数据库表，不新增 HTTP API。
- 不改 LLM client 初始化方式，继续由 `agent` crate runtime 持有。
- 修改 Agent worker autosuggest：
  - web 岗位先识别 native Web skill。
  - 普通 SEO、关键词研究、页面转化任务仍只触发 native skill。
  - 当用户明确表达“最新 / 实时 / 联网 / 搜索 / 公开来源 / 算法 / 规则 / 关键词竞争 / 搜索量 / 行业基准 / 转化率基准”等外部资料诉求时，追加一个搜索类 skill。
  - 搜索步骤通过现有 `skill_requested` / `skill_result` SSE 事件暴露，search adapter 不可用时沿用降级契约，不伪造来源。
- 商品上下文只来自本轮显式 `context.selected_summary`：
  - 用户已手动选择商品时，native skill 和 search skill 可携带 `product_ref/product_name/product_category`。
  - 未选择商品时，不从历史摘要或默认演示商品推断商品上下文。

## 链路映射

- `web_seo_optimize` + SEO 算法 / 流量规则 / 排名机制 -> `search_platform_policy`
- `web_seo_optimize` + 关键词竞争 / 搜索量 / 流量趋势 / 最新 SEO 资料 -> `search_market_info`
- `web_keyword_research` + 关键词竞争 / 搜索量 / CPC / 热度 / 联网资料 -> `search_market_info`
- `web_page_conversion` + 行业转化率基准 / CVR benchmark / 行业均值 -> `search_industry_benchmarks`

## 权限与鉴权

- 仍使用现有 Agent session 鉴权和 worker role。
- web 岗位已在 role catalog 中允许使用 `search_market_info`、`search_platform_policy`、`search_industry_benchmarks`，本轮不新增权限模型。

## API / SSE 影响

- 不新增 SSE 事件类型。
- 更新 `API_DOCS/sse.md`，说明 web 岗位可产生 native + search 双 skill 链路。
- 前端继续消费已有 `skill_requested`、`skill_result`、`workspace_artifact_created`。

## 验收

- 普通 Web 增长任务仍只触发 native skill。
- “SEO 优化 + 联网查最新搜索算法 / 关键词竞争”触发 `web_seo_optimize -> search_platform_policy/search_market_info`。
- “关键词研究 + 最新搜索量 / CPC / 竞争”触发 `web_keyword_research -> search_market_info`。
- “页面转化 + 行业转化率基准”触发 `web_page_conversion -> search_industry_benchmarks`。
- 未选择商品时不写空 `product_ref/product_name/product_category`。
- 运行 `cargo fmt --all`、相关 agent 测试、`cargo check`、`cargo clippy`。
