# Post 8-71 商品 UUID 引用进入 Skill 契约

状态：done

## 背景

新产品商品 ID 是 UUID 字符串，SSE 上下文使用 `productIds/contextRefs`。旧产品迁来的部分 skill 仍保留 `product_id: integer` 字段，如果把 UUID 硬塞进去会造成类型错配，也会影响后续 service adapter、artifact 来源和前端来源面板对齐。

## 本次改动

- 保留旧 `product_id`，作为旧整数参数兼容字段。
- 新增 `product_ref: string`，表示新产品商品引用，格式为 `product:<uuid>`。
- 新增 `product_refs: string[]`，用于直播脚本等多商品场景。
- worker 从 `context.selected_summary` 解析选中商品后，自动写入 `product_ref/product_refs`。
- autosuggest input 会清理 `null` 和空数组，未选商品时不会留下空的商品引用字段。
- creative / web / design 的 data readiness 识别 `product_ref/product_refs`，显式选品时标记为 `user_provided`。
- ops 上架文案和执行计划 schema 支持 `product_ref`。

## 涉及能力

- `creative_video_script`
- `creative_seeding_copy`
- `creative_live_script`
- `creative_product_article`
- `web_seo_optimize`
- `web_title_generator`
- `web_keyword_research`
- `web_product_description`
- `design_main_image`
- `design_detail_page`
- `design_color_scheme`
- `design_material_spec`
- `design_campaign_poster`
- `ops_listing_copy`
- `ops_execution_plan`

## 用户可感知变化

用户手动选择商品后，后端不仅会把商品名称和卖点传给 skill，还会把可追踪的商品引用 `product:<uuid>` 传进去。后续交付物、来源面板、商品详情跳转和 service adapter 可以基于这个引用继续衔接。

用户没有选择商品时，skill input 不会出现 `product_ref/product_refs`，仍按未指定商品的普通链路处理。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不让 agent crate 直连数据库读取商品。
- `product_id: integer` 暂时保留，后续可逐步迁为 `product_ref`。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo fmt --all --check`
- [x] `cargo test -p agent selected_product_context -- --nocapture`
- [x] `cargo test -p agent product_ref -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
