# Post 8-105：搜索证据进入上下文内容包的可读摘要

## 需求摘要

继续按“旧产品能力 = 后端内核能力来源、Kimi Agent 集群 = 前端体验参考、新产品架构 = 唯一落地框架”的路线迁移。前几轮已经让 `search_*` skill 在显式选品场景下保留商品上下文，并把 live 搜索结果沉淀为 `search_evidence` 工作区交付物。本轮解决后续复用问题：`workspace_context_pack` 读取工作区交付物时，不应把 `search_evidence.content` 中完整 JSON 原样注入 Agent，而应生成可读、可控的搜索证据摘要。

## 后端设计

- 不新增数据库表，不修改 migration。
- 不新增 HTTP API。
- 继续复用 `WorkspaceApi::context_pack`。
- 对普通 artifact 保持现有规则：优先读取 `content`，没有正文时读取 `preview`。
- 对 `artifactType=search_evidence` 且 `metadata.kind=agent_search_evidence` 的 artifact：
  - 优先解析 `content` 中完整搜索输出 JSON。
  - 生成 `contentKind=search_evidence_summary`。
  - 文本包含：
    - query / planned query
    - 显式商品上下文（如果存在）
    - evidence summary / llm_context
    - confidence / source count / domains
    - Top sources 的 title / domain / url / snippet
  - 如果 JSON 解析失败或没有可读字段，则回退到 `preview`。
- 不从搜索 query 或历史摘要推断商品；只使用 artifact metadata 或 search output query 中已有的显式商品上下文字段。

## 鉴权与用户隔离

本切片不新增鉴权逻辑。`context_pack` 仍按当前登录用户校验 workspace、conversation、product 和 run 归属。搜索证据 artifact 仍只来自当前用户可访问的工作区交付物。

## LLM 与外部系统

本切片不新增 LLM 调用，不访问外部搜索 provider，也不创建新的 live 搜索。它只把已经沉淀的真实 live 搜索证据整理成更适合 Agent 后续复用的文本。

## API/SSE 文档

同步更新根目录 `API_DOCS/app.md` 与 `API_DOCS/sse.md`，说明 `workspace_context_pack` 对 `search_evidence` 的 `contentKind=search_evidence_summary` 语义。

## 验证计划

- `cargo fmt --all`
- `cargo test -p service context_pack_search_evidence_uses_readable_summary -- --nocapture`
- `cargo test -p service context_pack_search_evidence_falls_back_to_preview_when_json_invalid -- --nocapture`
- `cargo fmt --all --check`
- `cargo check -p service`
- `cargo clippy -p service`
- `git diff --check`

