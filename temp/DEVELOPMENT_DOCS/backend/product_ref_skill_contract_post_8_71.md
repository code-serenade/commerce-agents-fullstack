# Post 8-71 商品 UUID 引用进入 Skill 契约

## 业务理解

新产品的商品 ID 是 UUID 字符串，前端和 web-server 都以 `productIds/contextRefs` 传递商品引用。但部分旧产品迁来的 skill schema 仍使用 `product_id: integer`，这会导致后端不能把新产品真实商品 UUID 安全写入 skill input。

本切片目标是建立兼容契约：

- 保留旧 `product_id`，兼容旧参数和旧测试。
- 新增 `product_ref` / `product_refs`，用于新产品 UUID 或 `product:<uuid>` 引用。
- worker 自动补偿商品型 skill input 时，在用户显式选品后写入 `product_ref` 或 `product_refs`。
- skill 输出和 data readiness 能识别 `product_ref`，避免把显式 UUID 商品上下文误标为默认假设。

## 数据库设计

本切片不新增表、不修改 migration。

## 接口设计

本切片不新增 HTTP API。

SSE 请求仍使用：

- `metadata.context.productIds`
- `metadata.context.contextRefs`

Agent runtime 内部 skill input 新增兼容字段：

- `product_ref: string`
- `product_refs: string[]`

## 鉴权与用户隔离

商品归属校验仍发生在 web-server / service 层构建 `context.selected_summary` 时。agent worker 只消费已校验后的 `context.selected_summary`，不直连数据库，也不自行扩大商品可见范围。

## 行为规则

1. 只有存在 `context.selected_summary` 时才写入 `product_ref/product_refs`。
2. `product_ref` 使用 `product:<uuid>` 格式，便于前端、artifact、source panel 和后续 service adapter 对齐。
3. 不把 UUID 写进旧 `product_id: integer` 字段。
4. 未选择商品时不写 `product_ref/product_refs`。
5. 旧 `product_id` 参数继续可用，但只作为旧整数参数兼容。

## 本次涉及 Skill

- creative：`creative_video_script`、`creative_seeding_copy`、`creative_live_script`、`creative_product_article`
- web：`web_seo_optimize`、`web_title_generator`、`web_keyword_research`、`web_product_description`
- design：`design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster`
- ops：`ops_listing_copy`、`ops_execution_plan`

## 验收

- 显式选品后，worker autosuggest input 包含 `product_ref` 或 `product_refs`。
- creative / web / design data readiness 把 `product_ref/product_refs` 视为用户提供的商品上下文。
- 旧 `product_id` integer 兼容不被破坏。
- 未选商品时不写 `product_ref/product_refs`。
