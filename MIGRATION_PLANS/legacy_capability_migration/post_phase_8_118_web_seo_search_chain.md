# Post 8-118 Web/SEO 增长搜索链路

## 目标

把旧产品 Web/SEO 类 skill 内部隐藏的 `_web_search` 能力，迁移为新产品架构下可见、可审计、可降级的 skill 链路。

旧产品参考：

```text
D:\github\commerce-agents崩了之后重新做\server\src\skills\web.py
```

## 修改范围

- `backend/crates/agent/src/agent/worker/runner.rs`
- `API_DOCS/sse.md`
- `temp/DEVELOPMENT_DOCS/backend/web_seo_search_chain_post_8_118.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`

## 核心变化

- 普通 SEO、关键词研究、页面转化任务仍只调用 native Web skill。
- 带“最新 / 实时 / 联网 / 搜索 / 公开来源 / 搜索算法 / 流量规则 / 关键词竞争 / 搜索量 / CPC / 行业转化率基准”等诉求的 Web 任务，会追加搜索 skill。
- 搜索不再隐藏在 Web skill 输出里，而是通过独立 `skill_requested` / `skill_result` 出现在 SSE 过程里。
- 搜索 provider 不可用时继续返回 adapter_required/degraded，不伪造实时来源。
- 未显式选商品时，不写 `product_ref/product_name/product_category`。

## 链路

```text
web_seo_optimize -> search_platform_policy(policy_type=algorithm)
web_seo_optimize -> search_market_info
web_keyword_research -> search_market_info
web_page_conversion -> search_industry_benchmarks(metric=conversion)
```

## 用户可感知变化

- 用户要求“做 SEO 优化并联网查最新搜索算法/流量规则”时，过程区能看到 SEO 优化和平台算法规则搜索两个步骤。
- 用户要求“关键词研究并查最新搜索量/CPC/竞争热度”时，过程区能看到关键词矩阵和市场资料搜索两个步骤。
- 用户要求“页面转化分析并查行业转化率基准”时，过程区能看到页面转化分析和行业基准搜索两个步骤。
- 搜索不可用时会显示明确降级，不会把无来源内容包装成实时 SEO 或行业基准资料。

## 验收命令

```text
cargo fmt --all
cargo test -p agent web
cargo test -p agent search
cargo test -p agent keeps_roles_from_requesting_unowned_skills
cargo check -p agent -p web-server
cargo clippy -p agent -p web-server
git diff --check
```
