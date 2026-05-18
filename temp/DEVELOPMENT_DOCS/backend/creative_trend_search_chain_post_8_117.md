# Post 8-117 创意内容趋势搜索链路

## 需求摘要

旧产品的创意类 skill 会在 `creative_video_script`、`creative_seeding_copy`、`creative_content_calendar`、`creative_trend_catch` 内部直接 `_web_search`，把“爆款风格、热门话题、营销节点、最新趋势”混入最终结果。新产品不能隐藏这类外部搜索能力，应按现有 Agent runtime、skill autosuggest、SSE、search adapter 和工作区 artifact 体系显式暴露。

## 后端设计

- 不新增数据库表，不新增 HTTP API。
- 不改 LLM client 初始化方式，继续由 `agent` crate runtime 持有。
- 修改 Agent worker autosuggest：
  - creative 岗位先识别 native 创意 skill。
  - 当用户同时表达“联网 / 最新 / 实时 / 搜索 / 热点 / 爆款 / 营销节点”等实时资料诉求时，追加一个搜索类 skill。
  - 搜索步骤通过现有 `skill_requested` / `skill_result` SSE 事件暴露，搜索 provider 不可用时沿用 `adapter_required/degraded`，不伪造来源。
- 商品上下文只来自本轮显式 `context.selected_summary`：
  - 用户已手动选择商品时，native skill 和 search skill 可携带 `product_ref/product_name/product_category`。
  - 未选择商品时，不从历史摘要或默认演示商品推断商品上下文。

## 链路映射

- `creative_video_script` + 实时爆款 / 最新内容趋势诉求 -> `search_trends`
- `creative_seeding_copy` + 热门风格 / 标签趋势诉求 -> `search_trends`
- `creative_content_calendar` + 最新营销节点 / 热门话题诉求 -> `search_market_info`
- `creative_trend_catch` + 最新趋势 / 热点诉求 -> `search_trends`

## 权限与鉴权

- 仍使用现有 Agent session 鉴权和 worker role。
- creative 岗位继续可使用自身 creative 类 skill。
- 本轮补充 creative 对 `search_market_info` 的白名单权限，仅用于内容排期的营销节点 / 市场公开资料搜索。

## API / SSE 影响

- 不新增 SSE 事件类型。
- 更新 `API_DOCS/sse.md`，说明 creative 岗位可产生 native + search 双 skill 链路。
- 前端继续消费已有 `skill_requested`、`skill_result`、`workspace_artifact_created`。

## 验收

- 普通创意任务仍只触发 native skill。
- “联网查最新爆款趋势再写脚本”触发 `creative_video_script -> search_trends`。
- “内容排期 + 最新营销节点 / 热门话题”触发 `creative_content_calendar -> search_market_info`。
- 未选择商品时不写空 `product_ref/product_name/product_category`。
- 运行 `cargo fmt --all`、相关 agent 测试、`cargo check`、`cargo clippy`。
