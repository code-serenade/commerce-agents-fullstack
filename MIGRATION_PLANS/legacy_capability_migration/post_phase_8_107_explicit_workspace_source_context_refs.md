# Post 8-107 显式工作区来源上下文引用

## 状态

done

## 目标

把旧产品“用户手动选择历史资料继续追问”的能力继续迁入新产品后端上下文体系。

上一轮已经支持显式选择历史 `workspace_artifact`。本轮补齐：

- 会话附件
- 商品资产
- 商品素材
- 工作区交付物

## 本轮修改

- 后端 service：`WorkspaceApi::context_summary` 支持显式 `conversation_file`、`product_asset`、`product_material` 引用。
- 后端 service：`WorkspaceApi::context_pack` 支持显式 `conversation_file`、`product_asset`、`product_material` 引用。
- 后端 service：新增统一显式来源解析结构 `ExplicitWorkspaceContextRefs`。
- 后端 service：新增当前用户归属、active 状态和所属资源校验。
- API：同步 `API_DOCS/app.md` 与 `API_DOCS/sse.md`。
- 文档：新增后端设计归档。

## 显式引用格式

- `conversation_file:{uuid}` / `file:{uuid}`
- `product_asset:{uuid}` / `asset:{uuid}`
- `product_material:{uuid}` / `material:{uuid}`
- `workspace_artifact:{uuid}` / `artifact:{uuid}`

## 用户可感知变化

后续前端选择某个历史附件、商品素材、商品资产或搜索证据后，继续提问时 Agent 可以把这份真实资料作为上下文。

如果用户没有手动选择资料，普通问答不会自动混入历史附件、历史商品素材、历史交付物、默认商品或测试商品资料。

## 边界

- 本轮不新增数据库表。
- 本轮不读取 object storage，不做 OCR、PDF、图片或视频解析。
- 显式商品资产 / 商品素材不会反向把所属商品当成本轮“已选商品”；商品知识和商品型 skill 输入仍必须通过 `productIds` 或 `product:{uuid}` 显式选择。
- 本轮不改前端，只补后端真实能力和 API 契约。

## 验证

- `cargo test -p service explicit_workspace_context_refs -- --nocapture`
- `cargo test -p service context_pack_search_evidence_uses_readable_summary -- --nocapture`
- `cargo check -p service -p web-server`
- `cargo clippy -p service -p web-server`
- `cargo fmt --all --check`
- `git diff --check`
