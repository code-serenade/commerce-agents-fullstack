# Post 8-72 客服商品查询 product_ref 契约

状态：done

## 背景

客服 / 售后岗位经常需要在回答客户前确认“本轮选中的商品资料”。之前 `service_query_product` 已经只读取显式商品上下文，但输入筛选主要还是旧 `product_id` 和商品名。Post 8-71 建立了 `product_ref=product:{uuid}` 后，需要让客服商品查询也使用同一契约。

## 本次改动

- `service_query_product` schema 新增 `product_ref`。
- `service_query_product` 查询输出中的 `query` 回显 `product_ref`。
- 显式上下文匹配优先级调整为：`product_ref` -> 旧 `product_id` -> `product_name`。
- `product_ref=product:{uuid}` 可匹配 `context.refs`、`context.product_ids`、`sources_json.product_id` 和 `selected_summary`。
- 客服 worker autosuggest 新增“查询商品 / 商品信息 / 商品资料 / 引用商品”等自然触发。
- 已显式选品时，客服商品查询 autosuggest input 自动带入 `product_ref` 和商品名。
- 未选商品时，input 会被清理，不留下空 `product_ref/product_name`。

## 涉及能力

- `service_query_product`

## 用户可感知变化

用户在客服 / 售后岗位问“查一下这个引用商品资料，客服回复客户前确认商品信息”时，后端能进入商品上下文查询 skill。

如果用户已经在输入区手动勾选商品，skill 会按 `product:<uuid>` 精确使用这次选中的商品资料；如果用户没有选择商品，系统仍会走缺显式上下文的降级路径，不会拿历史商品或默认商品来回答。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不让 agent crate 直连数据库读取商品。
- 旧 `product_id` 继续兼容旧参数。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent service_query_product -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_service_query_product_ref -- --nocapture`
- [x] `cargo test -p agent service_query_product_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
