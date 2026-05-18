# Post 8-102：运营执行计划 product_ref 契约

## 需求摘要

继续按“旧产品能力 = 后端内核能力来源、Kimi Agent 集群 = 前端体验参考、新产品架构 = 唯一落地框架”的路线迁移。当前切片补齐 `ops_execution_plan` 的商品引用链路，让运营执行计划能够区分“本轮显式选中商品”和“未指定商品的普通运营计划”。

## 后端设计

- 不新增数据库表，不修改 migration。
- 不新增 HTTP API。
- 继续复用 Agent SSE skill 事件和工作区 artifact 沉淀链路。
- `ops_execution_plan` 入参继续支持：
  - `goal`：运营目标。
  - `stage`：运营阶段。
  - `product_ref`：新产品商品引用，格式为 `product:{uuid}`，只来自本轮显式选中商品。
  - `product_id`：旧整数兼容字段，保持不变。
- skill 成功输出补充 `商品引用`：
  - 有 `product_ref` 时回显该引用。
  - 没有 `product_ref` 时为 `null`。

## Worker 设计

- `ops` 岗位兜底执行计划自动补偿改为结构化 input 构造函数。
- 只有 `context.selected_summary` 中存在本轮显式选中商品时，才写入 `product_ref`。
- 未选商品时不写空字符串、不写 `null`，避免前端和后续 artifact 误判为已指定商品。

## 鉴权与用户隔离

本切片不新增鉴权逻辑。商品上下文仍由 web-server/service 层按当前登录用户校验后注入 `context.selected_summary`。agent crate 只消费已校验的 runtime metadata，不直连数据库，不跨用户读取商品资料。

## LLM 与外部系统

本切片不新增 LLM provider、不读取淘宝、抖音、千川、ERP、WMS 或实时供应链数据。执行计划仍是 native deterministic skill；真实外部数据和平台执行仍由后续 search/adapter/service 能力处理。

## 验证计划

- `cargo fmt --all`
- `cargo test -p agent ops_execution_plan -- --nocapture`
- `cargo test -p agent selected_product_context_feeds_execution_plan_ref -- --nocapture`
- `cargo test -p agent execution_plan_without_selection_has_no_empty_product_ref -- --nocapture`
- `cargo test -p agent autosuggests_ops_execution_plan_without_llm_tool_json -- --nocapture`
- `cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p agent`
- `cargo clippy -p agent`
- `git diff --check`
