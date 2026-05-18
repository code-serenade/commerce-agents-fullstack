# Post 8-104：搜索证据 Artifact 商品上下文沉淀

状态：done

## 目标

上一轮 `Post 8-103` 已经让 `search_*` skill 在显式选品场景下把 `product_ref/product_name/product_category` 放入 `output.query`。本轮继续把这些字段沉淀到 `search_evidence` 工作区交付物，避免前端刷新、查看历史任务或查看来源面板时丢失“这条搜索证据是否绑定本轮选中商品”的上下文。

## 修改范围

- 后端 service：`build_search_evidence_artifact` 读取 `output.query.product_ref/product_name/product_category`。
- 后端 service：当商品上下文存在时，写入 `metadata.productContext.productRef/productName/productCategory`。
- 后端 service：同时冗余写入顶层 `metadata.productRef/productName/productCategory`，便于前端列表和历史面板快速读取。
- 后端 service：搜索证据 `preview` 增加 `Product:` 行，让用户打开交付物前即可区分指定商品搜索和普通搜索。
- API 文档：同步 `API_DOCS/app.md` 与 `API_DOCS/sse.md`。
- 后端设计文档：新增 `temp/DEVELOPMENT_DOCS/backend/search_evidence_product_context_post_8_104.md`。

## 边界

- 不新增数据库表。
- 不新增 HTTP API。
- 不改变外部搜索 provider 调用。
- 只在已有 live 搜索证据创建条件满足时沉淀 artifact。
- 未显式选商品时不写空字段，也不从 query 文本、历史对话、工作区摘要或预设商品推断商品。
- adapter 未配置、provider 失败、无可信来源、`deferred/degraded` 状态仍不会创建假搜索证据。

## 用户可感知变化

- 用户手动勾选商品后，再触发外部搜索类 skill，后续工作区交付物会保留该商品的 `product:<uuid>`、商品名和类目。
- 用户查看历史搜索证据时，可以区分“指定商品搜索证据”和“普通市场搜索证据”。
- 用户未选择商品时，搜索证据不会暗中绑定千乐咖啡、历史商品、预设商品或测试商品。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p service search_evidence_artifact_requires_live_sources -- --nocapture`
- [x] `cargo test -p service search_evidence_artifact_preserves_product_context -- --nocapture`
- [x] `cargo test -p service search_evidence_artifact_ignores_degraded_output -- --nocapture`

- [x] `cargo fmt --all --check`
- [x] `cargo check -p service`
- [x] `cargo clippy -p service`
- [x] `git diff --check`
