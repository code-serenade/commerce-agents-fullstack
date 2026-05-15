# Post 8-119 运营岗位实时搜索链路

## 目标

把旧产品运营类 skill 里依赖外部搜索的能力，迁移为新产品架构下可见、可审计、可降级的 skill 链路。

旧产品参考：

```text
D:\github\commerce-agents崩了之后重新做\server\src\skills\ops.py
```

## 修改范围

- `backend/crates/agent/src/agent/worker/runner.rs`
- `API_DOCS/sse.md`
- `temp/DEVELOPMENT_DOCS/backend/ops_search_chain_post_8_119.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`

## 核心变化

- 普通运营任务仍只调用 native ops skill。
- 带“最新 / 实时 / 联网 / 搜索 / 公开资料 / 市场定价 / 竞品价格区间 / 平台活动规则 / 大促规则 / 渠道趋势 / 行业 ROI 基准 / 热卖趋势”等诉求的运营任务，会追加搜索 skill。
- 搜索不再隐藏在运营 skill 输出里，而是通过独立 `skill_requested` / `skill_result` 出现在 SSE 过程里。
- 搜索 provider 不可用时继续返回 adapter_required/degraded，不伪造实时来源。
- 未显式选商品时，不写 `product_ref/product_name/product_category`。

## 链路

```text
ops_pricing_strategy -> search_competitor(focus=price)
ops_smart_pricing -> search_competitor(focus=price)
ops_promo_planning -> search_platform_policy(policy_type=promotion)
ops_promo_planning -> search_market_info(topic=news)
ops_channel_strategy -> search_industry_benchmarks(metric=roi)
ops_channel_strategy -> search_market_info(topic=general)
ops_assortment_planning -> search_trends
```

## 用户可感知变化

- 用户要求“做智能定价并联网查最新竞品价格区间”时，过程区能看到智能定价和竞品搜索两个步骤。
- 用户要求“做促销活动并查最新大促规则/报名规则”时，过程区能看到促销策划和平台政策搜索两个步骤。
- 用户要求“做渠道策略并查行业 ROI 基准”时，过程区能看到渠道策略和行业基准搜索两个步骤。
- 用户要求“做货盘规划并查热卖趋势”时，过程区能看到选品规划和趋势搜索两个步骤。
- 搜索不可用时会显示明确降级，不会把无来源内容包装成实时市场情报。

## 验收命令

```text
cargo fmt --all
cargo test -p agent ops
cargo test -p agent search
cargo test -p agent keeps_roles_from_requesting_unowned_skills
cargo check -p agent -p web-server
cargo clippy -p agent -p web-server
git diff --check
```
