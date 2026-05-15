# Post 8-105：搜索证据进入上下文内容包的可读摘要

状态：done

## 目标

让 `workspace_context_pack` 读取 `search_evidence` 工作区交付物时，注入 Agent 的不是完整原始 JSON，而是更适合后续问答复用的可读搜索证据摘要。

## 修改范围

- 后端 service：`context_pack_artifact_candidate` 针对 `artifactType=search_evidence` 且 `metadata.kind=agent_search_evidence` 的交付物启用专用摘要。
- 后端 service：新增搜索证据上下文摘要渲染，输出 `contentKind=search_evidence_summary`。
- 摘要文本包含 query、显式商品上下文、evidence summary、confidence、source count、domains 和 Top sources。
- JSON 无法解析或没有可读字段时回退到 `preview`。
- API/SSE 文档：补充 `workspace_context_pack` 的 `search_evidence_summary` 语义。

## 边界

- 不新增数据库表。
- 不新增 HTTP API。
- 不调用外部搜索 provider。
- 不创建新的 live 搜索。
- 不从 query 文本、历史对话或工作区摘要推断商品，只读取搜索证据里已有的显式商品上下文。

## 用户可感知变化

- 后续用户在同一工作区继续追问时，Agent 复用的是“搜索证据摘要”，不是难读的完整 JSON。
- 前端后续展示工作区内容包时，可以用 `contentKind=search_evidence_summary` 区分“真实搜索证据摘要”和普通交付物正文。
- 商品绑定搜索证据继续保留商品名、类目和 `product:<uuid>`，普通搜索证据仍不绑定商品。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p service context_pack_search_evidence_uses_readable_summary -- --nocapture`
- [x] `cargo test -p service context_pack_search_evidence_falls_back_to_preview_when_json_invalid -- --nocapture`

- [x] `cargo fmt --all --check`
- [x] `cargo check -p service`
- [x] `cargo clippy -p service`
- [x] `git diff --check`
