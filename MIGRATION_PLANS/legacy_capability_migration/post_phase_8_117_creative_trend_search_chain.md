# Post 8-117 创意内容趋势搜索链路

## 目标

把旧产品创意类 skill 内部隐藏的 `_web_search` 能力，迁移为新产品架构下可见、可审计、可降级的 skill 链路。

旧产品参考：

```text
D:\github\commerce-agents崩了之后重新做\server\src\skills\creative.py
```

## 修改范围

- `backend/crates/agent/src/agent/worker/runner.rs`
- `backend/crates/agent/src/roles/skill_catalog.rs`
- `API_DOCS/sse.md`
- `temp/DEVELOPMENT_DOCS/backend/creative_trend_search_chain_post_8_117.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`

## 核心变化

- 普通创意任务仍只调用 native creative skill。
- 带“最新 / 实时 / 联网 / 搜索 / 爆款 / 热门风格 / 热门话题 / 营销节点”等诉求的创意任务，会追加搜索 skill。
- 搜索不再隐藏在创意 skill 输出里，而是通过独立 `skill_requested` / `skill_result` 出现在 SSE 过程里。
- 搜索 provider 不可用时继续返回 adapter_required/degraded，不伪造实时来源。
- 未显式选商品时，不写 `product_ref/product_name/product_category`。

## 链路

```text
creative_video_script -> search_trends
creative_seeding_copy -> search_trends
creative_trend_catch -> search_trends
creative_content_calendar -> search_market_info
```

## 用户可感知变化

- 用户要求“写脚本并联网查最新爆款趋势”时，过程区能看到创意生成和外部趋势搜索两个步骤。
- 用户要求“排内容日历并查最新营销节点”时，过程区能看到内容排期和市场信息搜索两个步骤。
- 搜索不可用时会显示明确降级，不会把无来源内容包装成实时热点。

## 验收命令

```text
cargo fmt --all
cargo test -p agent creative
cargo test -p agent search
cargo test -p agent keeps_roles_from_requesting_unowned_skills
cargo check -p agent -p web-server
cargo clippy -p agent -p web-server
git diff --check
```
