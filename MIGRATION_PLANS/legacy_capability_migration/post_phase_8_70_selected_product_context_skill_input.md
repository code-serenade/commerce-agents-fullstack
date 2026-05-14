# Post 8-70 显式选品上下文进入 Skill 输入

状态：done

## 背景

前端已经能把用户手动勾选的商品以 `metadata.context.productIds` / `metadata.context.contextRefs` 发送给 SSE。后端 web-server 也已经只在显式选择存在时生成 `context.selected_summary`。但 worker 自动补偿商品型 skill input 时，部分分支仍主要使用本轮问题文本，无法稳定把选中商品名称、平台、类目和卖点传入 skill。

这会导致两个体验问题：

- 用户已选商品时，skill 不一定围绕该商品生成结构化产物。
- 用户未选商品时，历史对话或测试样例中的商品容易被误解成默认指定商品。

## 本次改动

- 新增 `SelectedProductContext` 解析逻辑，只读取 `MessageContext.metadata["context.selected_summary"]`。
- 解析 `Product source product:<uuid> | name=... | platform=... | category=... | brand=... | description=...`。
- 解析 `Knowledge source product_knowledge:<uuid> | product=<uuid> | content=...`，并把知识内容作为卖点写入对应商品。
- `creative_video_script`、`creative_seeding_copy`、`creative_product_article`、`creative_live_script` 自动补偿时优先使用显式选品。
- `ops_listing_copy`、`ops_promo_planning`、`ops_channel_strategy`、`ops_assortment_planning` 自动补偿时优先使用显式选品。
- `web_keyword_research`、`web_title_generator`、`web_product_description`、`web_store_design`、`web_page_conversion` 自动补偿时优先使用显式选品。
- `design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster` 自动补偿时优先使用显式选品。

## 明确边界

- 不从 `conversation.history_summary` 推断选中商品。
- 不从 `context.summary`、`context.combined_summary`、`rag.summary`、`workspaceContext.summary` 推断选中商品。
- 不注入“千乐咖啡”或任何测试/历史商品作为默认商品。
- 当前商品 ID 是 UUID，而部分旧 skill 的 `product_id` schema 仍是 integer，因此本次不写 `product_id`，避免类型错配。
- 不新增 API、不新增数据库表、不改前端。

## 用户可感知变化

用户在输入区手动选择某个商品后，再让系统生成短视频脚本、种草文案、上架文案、标题、详情页文案、主图/详情页设计方案时，后端 skill input 会真实携带该商品名称、平台、类目和卖点。

如果用户没有选择商品，同样的问题会按“未指定商品”的普通问答或普通 skill 路径处理，不会把之前对话里提到的商品、工作区绑定商品或测试商品自动带进来。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo fmt --all --check`
- [x] `cargo test -p agent selected_product_context -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_ops_listing_copy_features -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
