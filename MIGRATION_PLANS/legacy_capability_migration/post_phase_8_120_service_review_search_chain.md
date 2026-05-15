# Post 8-120 客服评价/口碑搜索链路

## 目标

把旧产品客服类 skill 中依赖外部评价、口碑、差评信息的能力，迁移为新产品架构下可见、可审计、可降级的 skill 链路。

旧产品参考：

```text
D:\github\commerce-agents崩了之后重新做\server\src\skills\customer_service.py
```

## 修改范围

- `backend/crates/agent/src/agent/worker/runner.rs`
- `API_DOCS/sse.md`
- `temp/DEVELOPMENT_DOCS/backend/service_review_search_chain_post_8_120.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`

## 核心变化

- 普通客服工单、售后、情绪分析、满意度驱动因素任务仍只调用 native service skill。
- 带“真实评价 / 商品评价 / 买家评价 / 口碑 / 差评 / 吐槽 / 真实反馈 / 联网 / 搜索 / 最新”等诉求的客服任务，会追加 `search_product_reviews`。
- 搜索不再隐藏在客服 native skill 输出里，而是通过独立 `skill_requested` / `skill_result` 出现在 SSE 过程里。
- `search_product_reviews` 会解析用户自然语言里的时间窗口和来源数量，例如“最近30天、最多5条来源”会落到 `days=30`、`max_results=5`。
- 搜索 provider 不可用时继续返回 adapter_required/degraded，不伪造实时评价来源。
- 未显式选商品时，不写 `product_ref/product_name/product_category`。

## 链路

```text
service_sentiment_analyzer -> search_product_reviews
service_ticket_handler -> search_product_reviews
service_return_handler -> search_product_reviews
service_nps_driver_analysis -> search_product_reviews
```

## 用户可感知变化

- 用户要求“分析差评和投诉情绪，并联网搜真实用户评价”时，过程区能看到情绪分析和评价搜索两个步骤。
- 用户要求“处理退款售后，并搜真实差评/口碑”时，过程区能看到退换货处理和评价搜索两个步骤。
- 用户明确说“最近30天、最多5条来源”时，后端能区分时间窗口和来源数量，不会把 30 天误当成来源条数。
- 搜索不可用时会显示明确降级，不会把无来源内容包装成真实平台评价。

## 验收命令

```text
cargo fmt --all
cargo test -p agent service
cargo test -p agent search
cargo test -p agent keeps_roles_from_requesting_unowned_skills
cargo check -p agent -p web-server
cargo clippy -p agent -p web-server
git diff --check
```
