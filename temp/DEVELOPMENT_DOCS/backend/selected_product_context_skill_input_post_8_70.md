# Post 8-70 显式选品上下文进入 Skill 输入

## 业务理解

用户在前端手动勾选商品后，后端应把该商品资料作为本轮指定商品上下文传给电商 skill。用户没有手动选择商品时，问答和 skill 应走未指定商品的普通链路，不能从历史对话、RAG 摘要、工作区摘要或测试样例中自动注入默认商品。

## 数据库设计

本切片不新增表、不修改 migration。商品资料仍来自既有商品/知识库上下文解析链路。

## 现有上下文来源

- 前端发送 `context.productIds` 和 `context.contextRefs`。
- `web-server` 只在显式选择存在时调用商品上下文服务。
- 后端把显式选品结果写入 `MessageContext.metadata["context.selected_summary"]`。
- `agent` worker 只把 `context.selected_summary` 视为“本轮手动选择的商品上下文”。

## 本次开发范围

1. 在 `agent` worker 中解析 `context.selected_summary` 的商品行和知识行。
2. 内容、上架、详情、设计等商品类 skill 自动补偿输入时，优先使用显式选中的商品名称、平台、类目、卖点。
3. 没有 `context.selected_summary` 时保持原普通链路，不从 `conversation.history_summary`、`context.summary`、`rag.summary` 或工作区摘要中推断指定商品。

## 接口与鉴权

本切片不新增 HTTP API。鉴权仍沿用现有 SSE 会话和商品上下文服务，商品上下文只由当前登录用户可访问的数据解析生成。

## 输入输出变化

### 输入

`MessageContext.metadata["context.selected_summary"]` 示例：

```text
Product source product:<uuid> | name=<name> | platform=<platform> | category=<category> | brand=<brand> | price=<price> <currency> | status=<status> | description=<description>
Knowledge source product_knowledge:<uuid> | product=<uuid> | title=<title> | type=<type> | source=<source> | confidence=<n> | content=<content>
```

### 输出

worker 自动生成的 skill input 会在显式选品时携带：

- `product_name`
- `platform`
- `category` / `product_type`
- `features` / `selling_points`

具体字段按各 skill 已有 schema 写入，不新增不受支持的字段。

## 边界

- 只解析 `context.selected_summary`。
- `context.summary` 仅作为对话上下文，不作为指定商品来源。
- 商品 ID 当前为 UUID 字符串，而部分旧 skill 的 `product_id` schema 是 integer，因此本切片不把 UUID 写入 `product_id`，避免类型错配。
- 外部平台真实数据、实时搜索和平台写操作不在本切片处理。

## 验收

1. 手动选择商品后，短视频脚本、上架文案、详情文案、详情页设计等 skill 使用选中商品。
2. 未选择商品时，即使历史摘要中出现某个商品，也不会把它当成本轮指定商品。
3. `cargo test -p agent` 的相关选品 autosuggest 测试通过。
4. `cargo check -p agent`、`cargo clippy -p agent` 通过。
