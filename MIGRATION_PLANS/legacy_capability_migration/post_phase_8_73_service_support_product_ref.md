# Post 8-73 客服支持类 Skill 商品引用契约补齐

状态：done

## 背景

客服工单、FAQ、升级处理和退换货处理都可能围绕用户本轮手动选中的商品展开。Post 8-72 已让 `service_query_product` 支持 `product_ref` 查询，本阶段继续把其它客服支持类 skill 的商品关联字段补齐到新产品商品引用契约。

## 本次改动

- `service_ticket_handler` schema 新增 `product_ref`，输出到 `内部工单字段.product_ref`。
- `service_faq_playbook` schema 新增 `product_ref`，输出 `商品引用`。
- `service_escalation_flow` schema 新增 `product_ref`，输出 `商品引用`。
- `service_return_handler` schema 新增 `product_ref`，输出 `商品引用`。
- worker 在显式选品后给客服工单、FAQ、升级、退换货自动补偿 `product_ref`。
- worker 未选品时通过 input compact 清理空商品引用，不留下空字段。
- 旧 `product_id` 继续作为旧整数关联字段兼容。

## 涉及能力

- `service_ticket_handler`
- `service_faq_playbook`
- `service_escalation_flow`
- `service_return_handler`

## 用户可感知变化

用户手动选择商品后，再让客服岗位生成工单话术、FAQ、主管升级流程或退换货处理方案时，后端结果会保留这次选中商品的 `product:<uuid>` 引用。

用户没有选择商品时，这些客服能力仍可按普通客服问题运行，但不会暗中绑定历史商品、默认商品或测试商品。

## 边界

- 本阶段不新增数据库表。
- 本阶段不新增 HTTP API。
- 本阶段不让 agent crate 直连数据库读取商品。
- `product_ref` 只作为本轮显式商品关联记录，不代表已查询实时平台商品库。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent service -- --nocapture`
- [x] `cargo test -p agent service_support -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_service_support_product_refs -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`
