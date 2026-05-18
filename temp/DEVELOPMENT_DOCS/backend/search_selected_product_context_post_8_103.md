# Post 8-103：搜索 skill 显式选品上下文

## 需求摘要

继续按“旧产品能力 = 后端内核能力来源、Kimi Agent 集群 = 前端体验参考、新产品架构 = 唯一落地框架”的路线迁移。当前切片补齐外部搜索类 skill 与本轮显式选中商品的衔接，让“搜这个商品的竞品 / 口碑 / 趋势 / 平台规则”等问法能够使用用户手动勾选的商品名称、类目和平台构造搜索请求。

## 后端设计

- 不新增数据库表，不修改 migration。
- 不新增 HTTP API。
- 继续复用 Agent SSE 的 `skill_requested` / `skill_result` 和现有 `search_evidence` artifact 链路。
- 搜索 skill 可接受可选商品上下文：
  - `product_ref`：新产品商品引用，格式为 `product:{uuid}`。
  - `product_name`：本轮显式选中的商品名。
  - `product_category`：本轮显式选中的商品类目。
- 搜索输出的 `query` 对象回显上述可选商品上下文，便于前端区分“指定商品搜索”和“普通搜索”。

## Worker 设计

- `autosuggest_search` 接收 `Task`，只从 `context.selected_summary` 读取本轮显式选中商品。
- 当用户使用“这个商品 / 本品 / 该产品 / 所选商品 / 当前商品”等指代词时，搜索 query 使用选中商品名和类目。
- 搜索平台优先使用用户文本显式平台；如果文本没有平台且本轮选中商品有平台，则使用选中商品平台。
- 未选商品时不写 `product_ref`、`product_name` 或 `product_category`，也不从历史摘要、RAG 摘要或工作区摘要推断默认商品。

## 鉴权与用户隔离

本切片不新增鉴权逻辑。商品上下文仍由 web-server/service 层按当前登录用户校验后注入 `context.selected_summary`。agent crate 只消费已校验的 runtime metadata，不直连数据库，不跨用户读取商品资料。

## LLM 与外部系统

本切片不新增 LLM provider，也不改变外部搜索 provider 调用。搜索 adapter 未配置、请求失败或无来源时仍返回 `deferred/degraded`，不会伪造联网结果。

## 验证计划

- `cargo fmt --all`
- `cargo test -p agent selected_product_context_feeds_search_queries -- --nocapture`
- `cargo test -p agent search_without_selection_has_no_empty_product_context -- --nocapture`
- `cargo test -p agent search_skill_echoes_product_context_in_degraded_output -- --nocapture`
- `cargo test -p agent autosuggests_search_skills_from_external_search_intent -- --nocapture`
- `cargo test -p agent autosuggest_search_preserves_native_analysis_without_external_intent -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
