# Post 8-104：搜索证据 Artifact 商品上下文沉淀

## 需求摘要

继续按“旧产品能力 = 后端内核能力来源、Kimi Agent 集群 = 前端体验参考、新产品架构 = 唯一落地框架”的路线迁移。上一切片已让 `search_*` skill 在显式选品场景下把 `product_ref/product_name/product_category` 放入 `output.query`。本切片把这些字段继续沉淀到 `search_evidence` 工作区交付物的 `metadata` 和 `preview`，让刷新后的工作区、历史任务和前端来源面板仍能识别这条 live 搜索证据是否绑定本轮选中商品。

## 后端设计

- 不新增数据库表，不修改 migration。
- 不新增 HTTP API。
- 继续复用 `WorkspaceApi::record_agent_search_evidence`。
- 只在搜索结果满足已有 live 条件时创建 `artifactType=search_evidence`：
  - `skill_id` 以 `search_` 开头。
  - `status=success`。
  - `execution_mode=adapter`。
  - `output.status=live`。
  - `output.sources` 非空。
- 从 `output.query.product_ref`、`output.query.product_name`、`output.query.product_category` 读取可选商品上下文。
- 如果存在商品上下文，则写入 artifact metadata：
  - `productContext.productRef`
  - `productContext.productName`
  - `productContext.productCategory`
  - 顶层冗余 `productRef/productName/productCategory`，方便前端列表快速读取。
- 如果不存在商品上下文，不写空字段，不从 query 字符串、历史摘要或工作区上下文推断商品。

## 鉴权与用户隔离

本切片不新增鉴权逻辑。`record_agent_search_evidence` 仍按当前登录用户校验 workspace/run 归属。商品上下文已经由 SSE 主链路在 service/web-server 层完成权限校验并注入 Agent metadata；本切片只沉淀 search skill 输出中的显式字段。

## LLM 与外部系统

本切片不新增 LLM provider，也不改变外部搜索 provider 调用。未配置、失败、无来源、degraded 或 deferred 的搜索仍不会创建搜索证据 artifact，也不会伪造 live 来源。

## API/SSE 文档

同步更新根目录 `API_DOCS/app.md` 与 `API_DOCS/sse.md`，说明 `search_evidence` artifact metadata 可带商品上下文。

## 验证计划

- `cargo fmt --all`
- `cargo test -p service search_evidence_artifact_requires_live_sources -- --nocapture`
- `cargo test -p service search_evidence_artifact_preserves_product_context -- --nocapture`
- `cargo test -p service search_evidence_artifact_ignores_degraded_output -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p service`
- `cargo clippy -p service`
- `git diff --check`
