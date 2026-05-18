# Post 8-74 营销活动草稿 product_ref 契约

状态：done

## 背景

`coordination_create_campaign` 会把 Agent 输出沉淀为营销活动草稿。活动草稿需要能追踪本轮显式选中的商品，但旧契约只有 `product_id`，不利于统一新产品 `product_ref=product:{uuid}` 商品引用。

## 本次改动

- `coordination_create_campaign` schema 新增 `product_ref`。
- 输出 `campaign_creation.product_ref` 和 `normalized_inputs.product_ref`。
- 未显式传入 `product_ref` 时，从 `context.refs` 的第一个 `product:{uuid}` 推断。
- 旧 `product_id` 继续保留；未传 `product_id` 时优先从 `product_ref` 派生 UUID 字符串，再回退 `context.product_ids`。
- 本阶段不触碰平台改价、改库存、同步商品等审批型平台写操作。

## 涉及能力

- `coordination_create_campaign`

## 用户可感知变化

用户手动选择商品后，让 Agent 把活动方案保存为营销活动草稿时，草稿输出会保留 `product:<uuid>` 商品引用。后续活动管理、工作区交付物和来源面板可以基于该引用继续对齐商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不让 agent crate 直连数据库。
- 本阶段不执行平台写操作，也不创建审批。
- 旧 `product_id` 仍用于兼容现有 campaign draft 持久化链路。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent coordination_create_campaign -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
